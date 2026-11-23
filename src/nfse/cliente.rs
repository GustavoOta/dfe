//! Cliente da SEFIN Nacional: assinatura, `POST /nfse`, `GET /dps/{id}` e `GET /nfse/{chave}`.
//!
//! Contrato: Manual dos Contribuintes da API do Emissor Público Nacional (v1.2) e Manual
//! Integrado do SN NFS-e (§6.1). Os nomes dos campos JSON (`dpsXmlGZipB64`, `chaveAcesso`,
//! `nfseXmlGZipB64`, `erros` com `codigo`/`Codigo`…) vêm do Swagger, que recusa acesso
//! automatizado; foram conferidos em três clientes abertos que falam com a SEFIN. A mensagem de
//! erro é lida nas duas grafias.
//!
//! mTLS com o A1, lido do arquivo a cada operação. O cliente HTTP nasce de
//! `interno::connection::WebService` (cadeia ICP-Brasil fora do Windows, timeout de 30 s).

use super::assinatura::{self, DpsAssinada};
use super::{Ambiente, DpsGerada};
use crate::error::{DfeError, Result};
use crate::interno::cert::Cert;
use crate::interno::connection::WebService;
use base64::{engine::general_purpose::STANDARD, Engine};
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression;
use serde::Deserialize;
use std::fmt;
use std::io::Read;

/// Teto do XML descompactado de uma resposta (a NFS-e tem poucos KB).
const LIMITE_XML: u64 = 8 * 1024 * 1024;

fn base(ambiente: Ambiente) -> &'static str {
    match ambiente {
        Ambiente::Producao => "https://sefin.nfse.gov.br/SefinNacional",
        Ambiente::ProducaoRestrita => "https://sefin.producaorestrita.nfse.gov.br/SefinNacional",
    }
}

/// Mensagem da SEFIN (item de `erros` ou de `alertas`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mensagem {
    pub codigo: Option<String>,
    pub descricao: Option<String>,
    pub complemento: Option<String>,
}

impl fmt::Display for Mensagem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let partes: Vec<&str> = [&self.codigo, &self.descricao, &self.complemento]
            .into_iter()
            .filter_map(|p| p.as_deref())
            .filter(|p| !p.is_empty())
            .collect();
        f.write_str(&partes.join(" - "))
    }
}

/// NFS-e gerada pela SEFIN a partir da DPS.
#[derive(Clone, Debug)]
pub struct NfseAutorizada {
    pub id_dps: String,
    pub chave_acesso: String,
    /// XML da NFS-e (descompactado), com a DPS e a assinatura da SEFIN.
    pub xml: String,
    pub alertas: Vec<Mensagem>,
    pub data_hora_processamento: Option<String>,
}

/// DPS recusada pelas regras de negócio: nenhuma NFS-e foi gerada.
#[derive(Clone, Debug)]
pub struct Rejeicao {
    pub id_dps: String,
    pub erros: Vec<Mensagem>,
    pub alertas: Vec<Mensagem>,
}

/// Resposta conclusiva do `POST /nfse`.
#[derive(Clone, Debug)]
pub enum ResultadoEmissao {
    Autorizada(NfseAutorizada),
    Rejeitada(Rejeicao),
}

/// Falha no envio da DPS.
#[derive(Clone, Debug)]
pub enum ErroEnvio {
    /// A DPS **não chegou** à SEFIN: certificado ilegível, ambiente trocado, falha ao abrir a
    /// conexão. Pode ser reenviada.
    Local(DfeError),
    /// A DPS pode ter sido processada (timeout, conexão caída depois do envio, HTTP 5xx,
    /// resposta ilegível). **Não reenviar às cegas:** consultar por
    /// [`NfseClient::consultar_dps`] com este `id_dps`; só a ausência confirmada libera o
    /// reenvio do mesmo XML.
    Indeterminado {
        id_dps: String,
        status: Option<u16>,
        motivo: String,
    },
}

impl fmt::Display for ErroEnvio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroEnvio::Local(e) => write!(f, "DPS não enviada: {e}"),
            ErroEnvio::Indeterminado {
                id_dps,
                status,
                motivo,
            } => match status {
                Some(s) => write!(
                    f,
                    "resultado indeterminado da DPS {id_dps} (HTTP {s}): {motivo}"
                ),
                None => write!(f, "resultado indeterminado da DPS {id_dps}: {motivo}"),
            },
        }
    }
}

impl std::error::Error for ErroEnvio {}

impl From<DfeError> for ErroEnvio {
    fn from(e: DfeError) -> Self {
        ErroEnvio::Local(e)
    }
}

/// Resultado do `GET /dps/{id}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConsultaDps {
    /// Existe NFS-e gerada a partir da DPS.
    Encontrada { chave_acesso: String },
    /// A SEFIN respondeu que não há NFS-e para a DPS (HTTP 404).
    NaoEncontrada,
}

/// Resultado do `GET /nfse/{chave}`.
#[derive(Clone, Debug)]
pub struct NfseConsultada {
    pub chave_acesso: String,
    pub xml: String,
}

/// Cliente da SEFIN Nacional, com ambiente fixo e o A1 do prestador.
///
/// ```no_run
/// # async fn exemplo(dps: dfe::nfse::DpsGerada) -> Result<(), Box<dyn std::error::Error>> {
/// use dfe::nfse::{Ambiente, NfseClient, ResultadoEmissao};
///
/// let cliente = NfseClient::builder()
///     .ambiente(Ambiente::ProducaoRestrita)
///     .cert("C:/certificados/empresa.pfx", "senha")
///     .build()?;
/// let assinada = cliente.assinar(&dps).await?;
/// // Persistir assinada.id() e assinada.xml() antes do envio.
/// match cliente.transmitir(&assinada).await? {
///     ResultadoEmissao::Autorizada(nfse) => println!("chave {}", nfse.chave_acesso),
///     ResultadoEmissao::Rejeitada(r) => println!("{:?}", r.erros),
/// }
/// # Ok(()) }
/// ```
pub struct NfseClient {
    ambiente: Ambiente,
    cert_path: String,
    cert_pass: String,
    #[cfg(test)]
    falso: Option<std::sync::Arc<testes::Falso>>,
}

#[derive(Default)]
pub struct NfseClientBuilder {
    ambiente: Option<Ambiente>,
    cert: Option<(String, String)>,
}

impl NfseClientBuilder {
    pub fn ambiente(mut self, ambiente: Ambiente) -> Self {
        self.ambiente = Some(ambiente);
        self
    }

    /// Caminho do `.pfx` (A1) e senha. O arquivo é lido a cada operação.
    pub fn cert(mut self, caminho: impl Into<String>, senha: impl Into<String>) -> Self {
        self.cert = Some((caminho.into(), senha.into()));
        self
    }

    pub fn build(self) -> Result<NfseClient> {
        let ambiente = self
            .ambiente
            .ok_or_else(|| DfeError::Configuracao("NFS-e: ambiente não informado".into()))?;
        let (cert_path, cert_pass) = self
            .cert
            .filter(|(caminho, _)| !caminho.trim().is_empty())
            .ok_or_else(|| DfeError::Configuracao("NFS-e: certificado A1 não informado".into()))?;
        Ok(NfseClient {
            ambiente,
            cert_path,
            cert_pass,
            #[cfg(test)]
            falso: None,
        })
    }
}

impl NfseClient {
    pub fn builder() -> NfseClientBuilder {
        NfseClientBuilder::default()
    }

    pub fn ambiente(&self) -> Ambiente {
        self.ambiente
    }

    /// Assina a DPS e confere no XSD do ambiente. Recusa DPS de outro ambiente.
    pub async fn assinar(&self, dps: &DpsGerada) -> Result<DpsAssinada> {
        self.conferir_ambiente(dps.ambiente())?;
        assinatura::assinar(dps, &self.cert_path, &self.cert_pass).await
    }

    /// `POST /nfse`: envia a DPS (GZip + Base64) e devolve a NFS-e ou a rejeição.
    pub async fn transmitir(
        &self,
        dps: &DpsAssinada,
    ) -> std::result::Result<ResultadoEmissao, ErroEnvio> {
        self.conferir_ambiente(dps.ambiente())?;
        let corpo = serde_json::json!({ "dpsXmlGZipB64": compactar(dps.xml())? }).to_string();
        let url = format!("{}/nfse", base(self.ambiente));
        match self.requisitar(Requisicao::post(url, corpo)).await {
            Ok(resposta) => classificar_emissao(dps.id(), resposta),
            Err(Falha::NaoEnviada(e)) => Err(ErroEnvio::Local(e)),
            Err(Falha::Indeterminada(motivo)) => Err(ErroEnvio::Indeterminado {
                id_dps: dps.id().to_string(),
                status: None,
                motivo,
            }),
        }
    }

    /// `GET /dps/{id}`: chave da NFS-e gerada a partir da DPS. Exige que o certificado seja de
    /// um ator da nota (prestador, tomador ou intermediário).
    pub async fn consultar_dps(&self, id_dps: &str) -> Result<ConsultaDps> {
        if id_dps.len() != 45
            || !id_dps.starts_with("DPS")
            || !id_dps[3..].bytes().all(|c| c.is_ascii_digit())
        {
            return Err(DfeError::Validacao(format!(
                "NFS-e: Id de DPS inválido: {id_dps}"
            )));
        }
        let url = format!("{}/dps/{id_dps}", base(self.ambiente));
        let resposta = self
            .requisitar(Requisicao::get(url))
            .await
            .map_err(Falha::em_erro)?;
        match resposta.status {
            404 => Ok(ConsultaDps::NaoEncontrada),
            200..=299 => {
                let json: RespostaJson = ler_json(&resposta)?;
                let chave_acesso = json
                    .chave_acesso
                    .filter(|c| !c.is_empty())
                    .ok_or_else(|| inesperada(&resposta))?;
                Ok(ConsultaDps::Encontrada { chave_acesso })
            }
            _ => Err(inesperada(&resposta)),
        }
    }

    /// `GET /nfse/{chave}`: XML da NFS-e.
    pub async fn consultar_nfse(&self, chave_acesso: &str) -> Result<NfseConsultada> {
        if chave_acesso.len() != 50
            || !chave_acesso
                .bytes()
                .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
        {
            return Err(DfeError::Validacao(format!(
                "NFS-e: chave de acesso inválida: {chave_acesso}"
            )));
        }
        let url = format!("{}/nfse/{chave_acesso}", base(self.ambiente));
        let resposta = self
            .requisitar(Requisicao::get(url))
            .await
            .map_err(Falha::em_erro)?;
        if !(200..=299).contains(&resposta.status) {
            return Err(inesperada(&resposta));
        }
        let json: RespostaJson = ler_json(&resposta)?;
        let xml = json
            .nfse_xml_gzip_b64
            .as_deref()
            .ok_or_else(|| inesperada(&resposta))
            .and_then(descompactar)?;
        Ok(NfseConsultada {
            chave_acesso: json
                .chave_acesso
                .unwrap_or_else(|| chave_acesso.to_string()),
            xml,
        })
    }

    fn conferir_ambiente(&self, ambiente: Ambiente) -> Result<()> {
        if ambiente != self.ambiente {
            return Err(DfeError::Validacao(format!(
                "NFS-e: DPS de {ambiente:?} num cliente de {:?}",
                self.ambiente
            )));
        }
        Ok(())
    }

    async fn requisitar(&self, req: Requisicao) -> std::result::Result<RespostaHttp, Falha> {
        #[cfg(test)]
        if let Some(falso) = &self.falso {
            return falso.responder(req);
        }
        mtls(&self.cert_path, &self.cert_pass, req).await
    }
}

#[derive(Debug)]
pub(super) struct Requisicao {
    pub(super) post: bool,
    pub(super) url: String,
    pub(super) corpo: Option<String>,
}

impl Requisicao {
    fn get(url: String) -> Self {
        Self {
            post: false,
            url,
            corpo: None,
        }
    }

    fn post(url: String, corpo: String) -> Self {
        Self {
            post: true,
            url,
            corpo: Some(corpo),
        }
    }
}

#[derive(Debug)]
pub(super) struct RespostaHttp {
    pub(super) status: u16,
    pub(super) corpo: String,
}

#[derive(Debug)]
pub(super) enum Falha {
    /// Nada saiu da máquina (certificado, conexão recusada, TLS).
    NaoEnviada(DfeError),
    /// A requisição pode ter chegado (timeout, conexão caída, corpo ilegível).
    Indeterminada(String),
}

impl Falha {
    fn em_erro(self) -> DfeError {
        match self {
            Falha::NaoEnviada(e) => e,
            Falha::Indeterminada(m) => DfeError::Webservice(format!("NFS-e: {m}")),
        }
    }
}

async fn mtls(
    cert_path: &str,
    cert_pass: &str,
    req: Requisicao,
) -> std::result::Result<RespostaHttp, Falha> {
    let cert = Cert::from_pfx(cert_path, cert_pass).map_err(Falha::NaoEnviada)?;
    let cliente = WebService::client(cert.identity).map_err(Falha::NaoEnviada)?;
    let pedido = match req.corpo {
        Some(corpo) if req.post => cliente
            .post(&req.url)
            .header("Content-Type", "application/json")
            .body(corpo),
        _ => cliente.get(&req.url),
    };
    let pedido = pedido.header("Accept", "application/json");
    let resposta = pedido.send().await.map_err(|e| {
        if e.is_connect() {
            Falha::NaoEnviada(DfeError::Webservice(format!(
                "NFS-e: falha ao conectar: {e}"
            )))
        } else {
            Falha::Indeterminada(format!("falha durante a requisição: {e}"))
        }
    })?;
    let status = resposta.status().as_u16();
    let corpo = resposta
        .text()
        .await
        .map_err(|e| Falha::Indeterminada(format!("falha ao ler a resposta: {e}")))?;
    Ok(RespostaHttp { status, corpo })
}

/// Campos das respostas da SEFIN (emissão e consultas).
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RespostaJson {
    #[serde(alias = "idDPS")]
    id_dps: Option<String>,
    chave_acesso: Option<String>,
    #[serde(rename = "nfseXmlGZipB64")]
    nfse_xml_gzip_b64: Option<String>,
    data_hora_processamento: Option<String>,
    #[serde(default)]
    alertas: Option<Vec<MensagemJson>>,
    #[serde(default)]
    erros: Option<Vec<MensagemJson>>,
    #[serde(default)]
    erro: Option<MensagemJson>,
}

#[derive(Deserialize)]
struct MensagemJson {
    #[serde(alias = "Codigo")]
    codigo: Option<serde_json::Value>,
    #[serde(alias = "Descricao")]
    descricao: Option<serde_json::Value>,
    #[serde(alias = "Complemento")]
    complemento: Option<serde_json::Value>,
}

impl From<MensagemJson> for Mensagem {
    fn from(m: MensagemJson) -> Self {
        let texto = |v: Option<serde_json::Value>| match v {
            None | Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(s)) => Some(s),
            Some(outro) => Some(outro.to_string()),
        };
        Mensagem {
            codigo: texto(m.codigo),
            descricao: texto(m.descricao),
            complemento: texto(m.complemento),
        }
    }
}

fn mensagens(lista: Option<Vec<MensagemJson>>) -> Vec<Mensagem> {
    lista
        .unwrap_or_default()
        .into_iter()
        .map(Mensagem::from)
        .collect()
}

fn classificar_emissao(
    id_dps: &str,
    resposta: RespostaHttp,
) -> std::result::Result<ResultadoEmissao, ErroEnvio> {
    let indeterminado = |motivo: String| ErroEnvio::Indeterminado {
        id_dps: id_dps.to_string(),
        status: Some(resposta.status),
        motivo,
    };
    let json: RespostaJson = serde_json::from_str(&resposta.corpo).map_err(|_| {
        indeterminado(format!(
            "resposta fora do formato: {}",
            trecho(&resposta.corpo)
        ))
    })?;

    let sucesso = (200..=299).contains(&resposta.status);
    if sucesso {
        if let (Some(chave), Some(b64)) = (&json.chave_acesso, &json.nfse_xml_gzip_b64) {
            let xml = descompactar(b64)
                .map_err(|e| indeterminado(format!("NFS-e {chave} gerada, XML ilegível: {e}")))?;
            return Ok(ResultadoEmissao::Autorizada(NfseAutorizada {
                id_dps: json.id_dps.unwrap_or_else(|| id_dps.to_string()),
                chave_acesso: chave.clone(),
                xml,
                alertas: mensagens(json.alertas),
                data_hora_processamento: json.data_hora_processamento,
            }));
        }
    }

    let mut erros = mensagens(json.erros);
    erros.extend(json.erro.map(Mensagem::from));
    if (sucesso || (400..=499).contains(&resposta.status)) && !erros.is_empty() {
        return Ok(ResultadoEmissao::Rejeitada(Rejeicao {
            id_dps: json.id_dps.unwrap_or_else(|| id_dps.to_string()),
            erros,
            alertas: mensagens(json.alertas),
        }));
    }
    Err(indeterminado(format!(
        "resposta sem NFS-e nem erros: {}",
        trecho(&resposta.corpo)
    )))
}

fn ler_json(resposta: &RespostaHttp) -> Result<RespostaJson> {
    serde_json::from_str(&resposta.corpo).map_err(|_| inesperada(resposta))
}

fn inesperada(resposta: &RespostaHttp) -> DfeError {
    let json: RespostaJson = serde_json::from_str(&resposta.corpo).unwrap_or_default();
    let mut erros = mensagens(json.erros);
    erros.extend(json.erro.map(Mensagem::from));
    let detalhe = if erros.is_empty() {
        trecho(&resposta.corpo)
    } else {
        erros
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ")
    };
    DfeError::Webservice(format!("NFS-e: HTTP {}: {detalhe}", resposta.status))
}

fn trecho(texto: &str) -> String {
    texto.chars().take(300).collect()
}

pub(super) fn compactar(xml: &str) -> Result<String> {
    let mut gz = Vec::new();
    GzEncoder::new(xml.as_bytes(), Compression::default())
        .read_to_end(&mut gz)
        .map_err(|e| DfeError::Xml(format!("NFS-e: falha ao compactar: {e}")))?;
    Ok(STANDARD.encode(gz))
}

pub(super) fn descompactar(b64: &str) -> Result<String> {
    let bytes = STANDARD
        .decode(b64.trim())
        .map_err(|e| DfeError::Xml(format!("NFS-e: Base64 inválido: {e}")))?;
    let mut xml = Vec::new();
    GzDecoder::new(bytes.as_slice())
        .take(LIMITE_XML + 1)
        .read_to_end(&mut xml)
        .map_err(|e| DfeError::Xml(format!("NFS-e: GZip inválido: {e}")))?;
    if xml.len() as u64 > LIMITE_XML {
        return Err(DfeError::Xml(
            "NFS-e: XML descompactado acima do limite".into(),
        ));
    }
    String::from_utf8(xml).map_err(|_| DfeError::Xml("NFS-e: XML fora de UTF-8".into()))
}

#[cfg(test)]
pub(super) mod testes {
    use super::*;
    use std::sync::Mutex;

    /// Transporte simulado: devolve as respostas em ordem e guarda as requisições.
    pub(in crate::nfse) struct Falso {
        respostas: Mutex<Vec<std::result::Result<RespostaHttp, Falha>>>,
        pub(in crate::nfse) pedidos: Mutex<Vec<Requisicao>>,
    }

    impl Falso {
        pub(in crate::nfse) fn responder(
            &self,
            req: Requisicao,
        ) -> std::result::Result<RespostaHttp, Falha> {
            self.pedidos.lock().unwrap().push(req);
            self.respostas.lock().unwrap().remove(0)
        }
    }

    pub(in crate::nfse) fn http(
        status: u16,
        corpo: &str,
    ) -> std::result::Result<RespostaHttp, Falha> {
        Ok(RespostaHttp {
            status,
            corpo: corpo.to_string(),
        })
    }

    pub(in crate::nfse) fn falha(f: Falha) -> std::result::Result<RespostaHttp, Falha> {
        Err(f)
    }

    impl NfseClient {
        pub(in crate::nfse) fn com_respostas(
            ambiente: Ambiente,
            cert_path: &str,
            cert_pass: &str,
            respostas: Vec<std::result::Result<RespostaHttp, Falha>>,
        ) -> (Self, std::sync::Arc<Falso>) {
            let falso = std::sync::Arc::new(Falso {
                respostas: Mutex::new(respostas),
                pedidos: Mutex::new(Vec::new()),
            });
            let cliente = NfseClient {
                ambiente,
                cert_path: cert_path.to_string(),
                cert_pass: cert_pass.to_string(),
                falso: Some(falso.clone()),
            };
            (cliente, falso)
        }
    }
}
