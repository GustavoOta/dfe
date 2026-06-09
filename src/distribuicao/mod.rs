use base64::Engine;
use flate2::read::GzDecoder;
use serde::{Deserialize, Deserializer, Serialize};
use std::io::Read;
use std::path::PathBuf;

mod service;
#[cfg(test)]
mod test;

/// Resposta da consulta de distribuição de DF-e ao Ambiente Nacional.
#[derive(Debug, Serialize, Deserialize)]
pub struct DistribuicaoResposta {
    /// Ambiente: `"1"` = Produção · `"2"` = Homologação.
    #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    /// Versão do aplicativo da SEFAZ.
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    /// Código de status (`"137"` = documentos localizados; `"138"` = nenhum documento).
    #[serde(rename = "cStat")]
    pub c_stat: String,
    /// Descrição do status.
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
    /// Data e hora da resposta.
    #[serde(rename = "dhResp")]
    pub dh_resp: String,
    /// Último NSU consultado.
    #[serde(rename = "ultNSU", default)]
    pub ult_nsu: String,
    /// NSU máximo disponível para o CNPJ.
    #[serde(rename = "maxNSU", default)]
    pub max_nsu: String,
    /// Documentos retornados (descomprimidos automaticamente de `docZip`).
    #[serde(
        rename = "loteDistDFeInt",
        default,
        deserialize_with = "deserialize_lote_dist_dfe_int"
    )]
    pub lote_dist_dfe_int: Option<Vec<LoteDistDFeInt>>,
}

/// Um documento DF-e retornado na distribuição.
#[derive(Debug, Serialize)]
pub struct LoteDistDFeInt {
    /// Número Sequencial Único do documento.
    pub nsu: String,
    /// Schema XML do documento (ex.: `"resNFe_v1.01.xsd"`).
    pub schema: String,
    /// Metadados da NF-e, quando `schema == "resNFe_v1.01.xsd"`.
    pub content: Option<ResNFe>,
    /// XML descomprimido do documento (base64 + GZIP decodificados automaticamente).
    pub content_xml: Option<String>,
    /// Conteúdo bruto (base64) retornado pela SEFAZ.
    pub content_raw: String,
}

/// Metadados de um resumo de NF-e (`resNFe`) retornados na distribuição.
#[derive(Debug, Serialize, Deserialize)]
pub struct ResNFe {
    /// Chave de acesso da NF-e (44 dígitos).
    #[serde(rename = "chNFe")]
    pub ch_nfe: String,
    /// CNPJ do emitente.
    #[serde(rename = "CNPJ")]
    pub cnpj: String,
    /// Razão social do emitente.
    #[serde(rename = "xNome")]
    pub x_nome: String,
    /// Inscrição Estadual do emitente.
    #[serde(rename = "IE")]
    pub ie: String,
    /// Data de emissão.
    #[serde(rename = "dhEmi")]
    pub dh_emi: String,
    /// Tipo de operação: `"0"` = Entrada · `"1"` = Saída.
    #[serde(rename = "tpNF")]
    pub tp_nf: String,
    /// Valor total da NF-e.
    #[serde(rename = "vNF")]
    pub v_nf: String,
    /// Digest SHA-1 do XML assinado.
    #[serde(rename = "digVal")]
    pub dig_val: String,
    /// Data e hora do recebimento pela SEFAZ.
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: String,
    /// Número do protocolo de autorização.
    #[serde(rename = "nProt")]
    pub n_prot: String,
    /// Situação da NF-e na SEFAZ.
    #[serde(rename = "cSitNFe")]
    pub c_sit_nfe: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoteDistDFeIntRaw {
    #[serde(rename = "@NSU")]
    nsu: String,
    #[serde(rename = "@schema")]
    schema: String,
    #[serde(rename = "$text")]
    content_raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoteDistDFeIntContainer {
    #[serde(rename = "docZip", default)]
    doc_zip: Vec<LoteDistDFeInt>,
}

fn deserialize_lote_dist_dfe_int<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<LoteDistDFeInt>>, D::Error>
where
    D: Deserializer<'de>,
{
    let container = Option::<LoteDistDFeIntContainer>::deserialize(deserializer)?;
    Ok(container
        .map(|value| value.doc_zip)
        .filter(|docs| !docs.is_empty()))
}

fn decode_doczip_content(encoded: &str) -> Result<String, String> {
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .map_err(|e| e.to_string())?;

    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut decompressed = String::new();
    decoder
        .read_to_string(&mut decompressed)
        .map_err(|e| e.to_string())?;

    Ok(decompressed)
}

fn parse_xml_to_res_nfe(xml: &str) -> Result<ResNFe, String> {
    quick_xml::de::from_str::<ResNFe>(xml).map_err(|e| e.to_string())
}

impl<'de> Deserialize<'de> for LoteDistDFeInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = LoteDistDFeIntRaw::deserialize(deserializer)?;
        let content_xml = decode_doczip_content(&raw.content_raw).ok();
        let content = content_xml
            .as_deref()
            .and_then(|xml| parse_xml_to_res_nfe(xml).ok());

        Ok(Self {
            nsu: raw.nsu,
            schema: raw.schema,
            content,
            content_xml,
            content_raw: raw.content_raw,
        })
    }
}

/// Destino do arquivo `flag.json` gerado após cada consulta de distribuição.
#[derive(Clone)]
pub enum FlagDir {
    /// Usa o caminho padrão: `{dir_do_executável}/distribuicao-logs/flag`.
    Default,
    /// Usa um diretório customizado fornecido pelo chamador.
    Custom(PathBuf),
}

/// Builder para consulta de documentos fiscais no Ambiente Nacional (AN).
/// Alias público: [`Distribuicao`].
pub struct Consulta {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// CNPJ do destinatário (14 dígitos).
    pub cnpj: String,
    /// Código IBGE da UF (ex.: `35` para SP).
    pub uf: u8,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub ambiente: u8,
    /// Diretório do `flag.json`; `None` desativa o mecanismo de flag.
    pub flag_dir: Option<FlagDir>,
}

/// Builder para consulta de documentos a partir de um NSU específico.
/// Alias público: [`DistribuicaoNSU`].
pub struct ConsultaNSU {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// CNPJ do destinatário (14 dígitos).
    pub cnpj: String,
    /// Código IBGE da UF.
    pub uf: u8,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub ambiente: u8,
    /// NSU a partir do qual consultar (15 dígitos).
    pub nsu: String,
    /// Diretório do `flag.json`; `None` desativa o mecanismo de flag.
    pub flag_dir: Option<FlagDir>,
}

/// Builder para consulta de um documento pela chave de acesso.
/// Alias público: [`DistribuicaoChaveAcesso`].
pub struct ConsultaChaveAcesso {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// CNPJ do destinatário (14 dígitos).
    pub cnpj: String,
    /// Código IBGE da UF.
    pub uf: u8,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub ambiente: u8,
    /// Chave de acesso da NF-e (44 dígitos).
    pub chave_acesso: String,
    /// Diretório do `flag.json`; `None` desativa o mecanismo de flag.
    pub flag_dir: Option<FlagDir>,
}

/// Manifestação **Ciência da Operação** (evento `210210`).
pub struct CienciaOperacao {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// CNPJ do destinatário.
    pub cnpj: String,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub ambiente: u8,
    /// Chave de acesso da NF-e (44 dígitos).
    pub chave_acesso: String,
}

/// Manifestação **Confirmação da Operação** (evento `210200`).
pub struct ConfirmacaoOperacao {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// CNPJ do destinatário.
    pub cnpj: String,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub ambiente: u8,
    /// Chave de acesso da NF-e (44 dígitos).
    pub chave_acesso: String,
}

/// Manifestação **Desconhecimento da Operação** (evento `210220`).
pub struct DesconhecimentoOperacao {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// CNPJ do destinatário.
    pub cnpj: String,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub ambiente: u8,
    /// Chave de acesso da NF-e (44 dígitos).
    pub chave_acesso: String,
}

/// Manifestação **Operação Não Realizada** (evento `210240`).
pub struct OperacaoNaoRealizada {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// CNPJ do destinatário.
    pub cnpj: String,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub ambiente: u8,
    /// Chave de acesso da NF-e (44 dígitos).
    pub chave_acesso: String,
    /// Justificativa (mínimo 15 caracteres).
    pub justificativa: String,
}

/// Resposta das operações de manifestação do destinatário.
pub type ManifestacaoResposta = crate::tipos::manifestacao::Response;

/// Consulta todos os documentos disponíveis (a partir do último NSU).
pub type Distribuicao = Consulta;
/// Consulta documentos a partir de um NSU específico.
pub type DistribuicaoNSU = ConsultaNSU;
/// Consulta um documento pela chave de acesso.
pub type DistribuicaoChaveAcesso = ConsultaChaveAcesso;

impl Consulta {
    /// Cria um builder vazio para consulta de distribuição.
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            uf: 0,
            ambiente: 0,
            flag_dir: None,
        }
    }

    /// Caminho do certificado A1 (`.pfx`).
    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    /// Senha do certificado.
    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    /// CNPJ do destinatário (14 dígitos, sem formatação).
    pub fn cnpj(mut self, cnpj: &str) -> Self {
        self.cnpj = cnpj.to_string();
        self
    }

    /// Código IBGE da UF (ex.: `35` para SP).
    pub fn uf(mut self, uf: u8) -> Self {
        self.uf = uf;
        self
    }

    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub fn ambiente(mut self, ambiente: u8) -> Self {
        self.ambiente = ambiente;
        self
    }

    /// Ativa o mecanismo de flag com o caminho padrão (`{exe}/distribuicao-logs/flag`).
    pub fn check_flag(mut self) -> Self {
        self.flag_dir = Some(FlagDir::Default);
        self
    }

    /// Ativa o mecanismo de flag com um diretório customizado.
    pub fn check_flag_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.flag_dir = Some(FlagDir::Custom(path.into()));
        self
    }

    /// Envia a consulta ao Ambiente Nacional e retorna [`DistribuicaoResposta`].
    pub async fn send(self) -> Result<DistribuicaoResposta, String> {
        if self.cert_path.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_path".to_string());
        }
        if self.cert_pass.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_pass".to_string());
        }
        if self.cnpj.trim().is_empty() {
            return Err("Campo obrigatório não informado: cnpj".to_string());
        }
        if self.uf == 0 {
            return Err("Campo obrigatório não informado: uf".to_string());
        }
        if self.ambiente == 0 {
            return Err("Campo obrigatório não informado: ambiente".to_string());
        }

        self.executar_consulta().await
    }
}

impl ConsultaNSU {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            uf: 0,
            ambiente: 0,
            nsu: String::new(),
            flag_dir: None,
        }
    }

    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    pub fn cnpj(mut self, cnpj: &str) -> Self {
        self.cnpj = cnpj.to_string();
        self
    }

    pub fn uf(mut self, uf: u8) -> Self {
        self.uf = uf;
        self
    }

    pub fn ambiente(mut self, ambiente: u8) -> Self {
        self.ambiente = ambiente;
        self
    }

    pub fn nsu(mut self, nsu: &str) -> Self {
        self.nsu = nsu.to_string();
        self
    }

    pub fn check_flag(mut self) -> Self {
        self.flag_dir = Some(FlagDir::Default);
        self
    }

    pub fn check_flag_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.flag_dir = Some(FlagDir::Custom(path.into()));
        self
    }

    pub async fn send(self) -> Result<DistribuicaoResposta, String> {
        if self.cert_path.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_path".to_string());
        }
        if self.cert_pass.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_pass".to_string());
        }
        if self.cnpj.trim().is_empty() {
            return Err("Campo obrigatório não informado: cnpj".to_string());
        }
        if self.uf == 0 {
            return Err("Campo obrigatório não informado: uf".to_string());
        }
        if self.ambiente == 0 {
            return Err("Campo obrigatório não informado: ambiente".to_string());
        }
        if self.nsu.trim().is_empty() {
            return Err("Campo obrigatório não informado: nsu".to_string());
        }

        self.executar_consulta_nsu().await
    }
}

impl ConsultaChaveAcesso {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            uf: 0,
            ambiente: 0,
            chave_acesso: String::new(),
            flag_dir: None,
        }
    }

    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    pub fn cnpj(mut self, cnpj: &str) -> Self {
        self.cnpj = cnpj.to_string();
        self
    }

    pub fn uf(mut self, uf: u8) -> Self {
        self.uf = uf;
        self
    }

    pub fn ambiente(mut self, ambiente: u8) -> Self {
        self.ambiente = ambiente;
        self
    }

    pub fn chave_acesso(mut self, chave_acesso: &str) -> Self {
        self.chave_acesso = chave_acesso.to_string();
        self
    }

    pub fn check_flag(mut self) -> Self {
        self.flag_dir = Some(FlagDir::Default);
        self
    }

    pub fn check_flag_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.flag_dir = Some(FlagDir::Custom(path.into()));
        self
    }

    pub async fn send(self) -> Result<DistribuicaoResposta, String> {
        if self.cert_path.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_path".to_string());
        }
        if self.cert_pass.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_pass".to_string());
        }
        if self.cnpj.trim().is_empty() {
            return Err("Campo obrigatório não informado: cnpj".to_string());
        }
        if self.uf == 0 {
            return Err("Campo obrigatório não informado: uf".to_string());
        }
        if self.ambiente == 0 {
            return Err("Campo obrigatório não informado: ambiente".to_string());
        }
        if self.chave_acesso.trim().is_empty() {
            return Err("Campo obrigatório não informado: chave_acesso".to_string());
        }

        self.executar_consulta_chave_acesso().await
    }
}

impl CienciaOperacao {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            ambiente: 0,
            chave_acesso: String::new(),
        }
    }

    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    pub fn cnpj(mut self, cnpj: &str) -> Self {
        self.cnpj = cnpj.to_string();
        self
    }

    pub fn ambiente(mut self, ambiente: u8) -> Self {
        self.ambiente = ambiente;
        self
    }

    pub fn chave_acesso(mut self, chave_acesso: &str) -> Self {
        self.chave_acesso = chave_acesso.to_string();
        self
    }

    pub async fn send(self) -> Result<ManifestacaoResposta, String> {
        if self.cert_path.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_path".to_string());
        }
        if self.cert_pass.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_pass".to_string());
        }
        if self.cnpj.trim().is_empty() {
            return Err("Campo obrigatório não informado: cnpj".to_string());
        }
        if self.ambiente == 0 {
            return Err("Campo obrigatório não informado: ambiente".to_string());
        }
        if self.chave_acesso.trim().is_empty() {
            return Err("Campo obrigatório não informado: chave_acesso".to_string());
        }

        self.executar_ciencia_operacao().await
    }
}

impl ConfirmacaoOperacao {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            ambiente: 0,
            chave_acesso: String::new(),
        }
    }

    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    pub fn cnpj(mut self, cnpj: &str) -> Self {
        self.cnpj = cnpj.to_string();
        self
    }

    pub fn ambiente(mut self, ambiente: u8) -> Self {
        self.ambiente = ambiente;
        self
    }

    pub fn chave_acesso(mut self, chave_acesso: &str) -> Self {
        self.chave_acesso = chave_acesso.to_string();
        self
    }

    pub async fn send(self) -> Result<ManifestacaoResposta, String> {
        if self.cert_path.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_path".to_string());
        }
        if self.cert_pass.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_pass".to_string());
        }
        if self.cnpj.trim().is_empty() {
            return Err("Campo obrigatório não informado: cnpj".to_string());
        }
        if self.ambiente == 0 {
            return Err("Campo obrigatório não informado: ambiente".to_string());
        }
        if self.chave_acesso.trim().is_empty() {
            return Err("Campo obrigatório não informado: chave_acesso".to_string());
        }

        self.executar_confirmacao_operacao().await
    }
}

impl DesconhecimentoOperacao {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            ambiente: 0,
            chave_acesso: String::new(),
        }
    }

    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    pub fn cnpj(mut self, cnpj: &str) -> Self {
        self.cnpj = cnpj.to_string();
        self
    }

    pub fn ambiente(mut self, ambiente: u8) -> Self {
        self.ambiente = ambiente;
        self
    }

    pub fn chave_acesso(mut self, chave_acesso: &str) -> Self {
        self.chave_acesso = chave_acesso.to_string();
        self
    }

    pub async fn send(self) -> Result<ManifestacaoResposta, String> {
        if self.cert_path.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_path".to_string());
        }
        if self.cert_pass.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_pass".to_string());
        }
        if self.cnpj.trim().is_empty() {
            return Err("Campo obrigatório não informado: cnpj".to_string());
        }
        if self.ambiente == 0 {
            return Err("Campo obrigatório não informado: ambiente".to_string());
        }
        if self.chave_acesso.trim().is_empty() {
            return Err("Campo obrigatório não informado: chave_acesso".to_string());
        }

        self.executar_desconhecimento_operacao().await
    }
}

impl OperacaoNaoRealizada {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            ambiente: 0,
            chave_acesso: String::new(),
            justificativa: String::new(),
        }
    }

    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    pub fn cnpj(mut self, cnpj: &str) -> Self {
        self.cnpj = cnpj.to_string();
        self
    }

    pub fn ambiente(mut self, ambiente: u8) -> Self {
        self.ambiente = ambiente;
        self
    }

    pub fn chave_acesso(mut self, chave_acesso: &str) -> Self {
        self.chave_acesso = chave_acesso.to_string();
        self
    }

    pub fn justificativa(mut self, justificativa: &str) -> Self {
        self.justificativa = justificativa.to_string();
        self
    }

    pub async fn send(self) -> Result<ManifestacaoResposta, String> {
        if self.cert_path.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_path".to_string());
        }
        if self.cert_pass.trim().is_empty() {
            return Err("Campo obrigatório não informado: cert_pass".to_string());
        }
        if self.cnpj.trim().is_empty() {
            return Err("Campo obrigatório não informado: cnpj".to_string());
        }
        if self.ambiente == 0 {
            return Err("Campo obrigatório não informado: ambiente".to_string());
        }
        if self.chave_acesso.trim().is_empty() {
            return Err("Campo obrigatório não informado: chave_acesso".to_string());
        }
        if self.justificativa.trim().is_empty() {
            return Err("Campo obrigatório não informado: justificativa".to_string());
        }

        self.executar_operacao_nao_realizada().await
    }
}

/*
ambiente 2 url: https://hom1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx
ambiente 1 url: https://www1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx


SOAP 1.2
The following is a sample SOAP 1.2 request and response. The placeholders shown need to be replaced with actual values.

POST /NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx HTTP/1.1
Host: hom1.nfe.fazenda.gov.br
Content-Type: application/soap+xml; charset=utf-8
Content-Length: length

<?xml version="1.0" encoding="utf-8"?>
<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope">
  <soap12:Body>
    <nfeDistDFeInteresse xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeDistribuicaoDFe">
      <nfeDadosMsg>xml</nfeDadosMsg>
    </nfeDistDFeInteresse>
  </soap12:Body>
</soap12:Envelope>
HTTP/1.1 200 OK
Content-Type: application/soap+xml; charset=utf-8
Content-Length: length

<?xml version="1.0" encoding="utf-8"?>
<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope">
  <soap12:Body>
    <nfeDistDFeInteresseResponse xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeDistribuicaoDFe">
      <nfeDistDFeInteresseResult>xml</nfeDistDFeInteresseResult>
    </nfeDistDFeInteresseResponse>
  </soap12:Body>
</soap12:Envelope>
 */
