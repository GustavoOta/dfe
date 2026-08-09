//! Consulta de situação da NF-e/NFC-e — `consSitNFe` (`NfeConsultaProtocolo4`).
//!
//! Consulta **não assinada** (só mTLS, como `status`) que devolve a situação atual de uma nota
//! pela chave de acesso: se está autorizada (`protNFe/infProt`), se não consta na base
//! (`cStat` 217 no nível da consulta, sem `protNFe`) e eventos já registrados contra ela
//! (ex.: cancelamento, em `procEventoNFe`).
//!
//! Uso previsto (Etapa 6c do `gravis-pdv/SEFAZ_LEDGER.md`): recuperar emissões órfãs — quando o
//! app fechou/timeout antes de a resposta da SEFAZ chegar, consultar a chave 1x (rate-limited,
//! 10/hora por chave — SEFAZ NT 2014.002) para decidir se a nota foi autorizada ou não. O consumo
//! disso pelo reconciliador do PDV **não** faz parte deste módulo — só o primitivo de consulta.

mod parser;
mod xml;

use crate::error::{DfeError, Result};
use crate::interno::chave_acesso::ChaveAcesso;
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::interno::ws::nfe_consulta_protocolo;
use crate::tipos::consulta_situacao::Response;

/// Builder fluente para consulta de situação da NF-e/NFC-e.
///
/// # Exemplo
/// ```no_run
/// use dfe::ConsultaSituacaoBuilder;
/// # async fn ex() -> Result<(), dfe::DfeError> {
/// let resp = ConsultaSituacaoBuilder::new()
///     .cert("./cert.pfx", "senha")
///     .tp_amb(2)
///     .chave("35...") // 44 dígitos
///     .send()
///     .await?;
/// println!("cStat: {}", resp.response.c_stat);
/// # Ok(()) }
/// ```
pub struct ConsultaSituacaoBuilder {
    cert_path: Option<String>,
    cert_pass: Option<String>,
    tp_amb: Option<u8>,
    chave: Option<String>,
}

impl ConsultaSituacaoBuilder {
    pub fn new() -> Self {
        Self {
            cert_path: None,
            cert_pass: None,
            tp_amb: None,
            chave: None,
        }
    }

    pub fn cert(mut self, path: &str, pass: &str) -> Self {
        self.cert_path = Some(path.to_string());
        self.cert_pass = Some(pass.to_string());
        self
    }

    /// 1 = Produção | 2 = Homologação
    pub fn tp_amb(mut self, v: u8) -> Self {
        self.tp_amb = Some(v);
        self
    }

    /// Chave de acesso de 44 dígitos da NF-e/NFC-e a consultar.
    pub fn chave(mut self, v: &str) -> Self {
        self.chave = Some(v.to_string());
        self
    }

    pub async fn send(self) -> Result<Response> {
        let cert_path = self
            .cert_path
            .ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass = self
            .cert_pass
            .ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let tp_amb = self
            .tp_amb
            .ok_or_else(|| DfeError::Configuracao("tp_amb não informado".to_string()))?;
        let chave = self
            .chave
            .ok_or_else(|| DfeError::Validacao("chave não informada".to_string()))?;

        if tp_amb != 1 && tp_amb != 2 {
            return Err(DfeError::Validacao(
                "tp_amb deve ser 1 (producao) ou 2 (homologacao)".to_string(),
            ));
        }
        // `ChaveAcesso::extract_composition` faz slicing direto da chave — checar o tamanho
        // ANTES de chamar evita panic (mesma ordem do `substituicao`).
        if chave.len() != 44 {
            return Err(DfeError::Validacao("chave deve ter 44 dígitos".to_string()));
        }

        consultar(cert_path, cert_pass, tp_amb, chave).await
    }
}

impl Default for ConsultaSituacaoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Deriva o endpoint da consulta a partir da própria chave: UF (posições 1-2) e **modelo**
/// (21-22) decidem o host. NF-e (55) e NFC-e (65) têm URLs distintas em 26 das 27 UFs — a NFC-e
/// roda em infraestrutura própria (`nfce.*`) — então errar o modelo aqui manda a consulta para o
/// autorizador errado. Só GO compartilha a mesma URL entre os dois modelos.
///
/// `svn = false` fixo: a NFC-e **não tem SVC** (a contingência dela é off-line, `tpEmis = 9`, e a
/// chave off-line é consultada no autorizador normal).
///
/// Extraída de `consultar` para ser testável sem rede/certificado.
fn resolver_url(tp_amb: u8, chave: &str) -> Result<&'static str> {
    let comp = ChaveAcesso::extract_composition(chave)?;
    let uf = crate::interno::uf::sigla_por_codigo(&comp.uf_code)?;
    let modelo: u32 = comp
        .modelo
        .parse()
        .map_err(|_| DfeError::Xml(format!("modelo inválido na chave: {}", comp.modelo)))?;
    nfe_consulta_protocolo(tp_amb, uf, modelo, false)
}

async fn consultar(cert_path: String, cert_pass: String, tp_amb: u8, chave: String) -> Result<Response> {
    let url = resolver_url(tp_amb, &chave)?;

    let request_xml =
        xml::cons_sit_nfe_request_xml(tp_amb, &chave).map_err(DfeError::Validacao)?;

    let send_xml = request_xml.clone();
    let response = MtlsTransport::new(&cert_path, &cert_pass)
        .send_soap(url, request_xml)
        .await?;

    let parsed = parser::parse_ret_cons_sit_nfe(&response)?;
    Ok(Response {
        response: parsed,
        send_xml,
        receive_xml: response,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAVE: &str = "35000000000000000000550010000000001000000001";

    /// Chave sintética de 44 posições com `cUF` e `modelo` controlados (DV não é validado por
    /// `extract_composition`, então não precisa fechar).
    fn chave_com(uf_code: &str, modelo: &str) -> String {
        let c = format!("{uf_code}260811222333000181{modelo}0010000000011000000011");
        assert_eq!(c.len(), 44, "chave sintética malformada: {c}");
        c
    }

    fn builder_valido() -> ConsultaSituacaoBuilder {
        ConsultaSituacaoBuilder::new()
            .cert("inexistente.pfx", "x")
            .tp_amb(2)
            .chave(CHAVE)
    }

    // Validações abaixo devem falhar ANTES de qualquer I/O (cert/rede) — provam o fail-fast
    // (mesmo padrão de `substituicao::tests`).
    #[tokio::test]
    async fn rejeita_cert_path_ausente() {
        let err = ConsultaSituacaoBuilder::new()
            .tp_amb(2)
            .chave(CHAVE)
            .send()
            .await
            .unwrap_err();
        assert!(matches!(err, DfeError::Configuracao(_)));
    }

    #[tokio::test]
    async fn rejeita_tp_amb_ausente() {
        let err = ConsultaSituacaoBuilder::new()
            .cert("inexistente.pfx", "x")
            .chave(CHAVE)
            .send()
            .await
            .unwrap_err();
        assert!(matches!(err, DfeError::Configuracao(_)));
    }

    #[tokio::test]
    async fn rejeita_tp_amb_invalido() {
        let err = builder_valido().tp_amb(9).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_chave_ausente() {
        let err = ConsultaSituacaoBuilder::new()
            .cert("inexistente.pfx", "x")
            .tp_amb(2)
            .send()
            .await
            .unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_chave_tamanho_invalido() {
        let err = builder_valido().chave("35065").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    // Derivação chave → URL. O que estes testes travam é que o **modelo lido da chave** decide o
    // host: uma chave de NFC-e (65) nunca pode cair no endpoint de NF-e (55) e vice-versa.

    #[test]
    fn deriva_url_nfce_sp_nao_usa_host_da_nfe() {
        let chave = chave_com("35", "65");
        assert_eq!(
            resolver_url(1, &chave).unwrap(),
            "https://nfce.fazenda.sp.gov.br/ws/NFeConsultaProtocolo4.asmx"
        );
        assert_eq!(
            resolver_url(2, &chave).unwrap(),
            "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeConsultaProtocolo4.asmx"
        );
    }

    #[test]
    fn deriva_url_nfe_sp() {
        let chave = chave_com("35", "55");
        assert_eq!(
            resolver_url(1, &chave).unwrap(),
            "https://nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx"
        );
        assert_eq!(
            resolver_url(2, &chave).unwrap(),
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx"
        );
    }

    // UF atendida pelo SVRS: a separação nfe/nfce vale também no autorizador virtual.
    #[test]
    fn deriva_url_por_modelo_via_svrs_rj() {
        assert_eq!(
            resolver_url(1, &chave_com("33", "65")).unwrap(),
            "https://nfce.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx"
        );
        assert_eq!(
            resolver_url(1, &chave_com("33", "55")).unwrap(),
            "https://nfe.svrs.rs.gov.br/ws/NfeConsulta/NfeConsulta4.asmx"
        );
    }

    // GO é a única UF que documenta os mesmos webservices para NF-e e NFC-e — se este teste
    // quebrar, GO passou a ter endpoint próprio de NFC-e.
    #[test]
    fn go_compartilha_url_entre_modelos() {
        for amb in [1u8, 2] {
            assert_eq!(
                resolver_url(amb, &chave_com("52", "55")).unwrap(),
                resolver_url(amb, &chave_com("52", "65")).unwrap(),
            );
        }
    }

    // Todas as 27 UFs resolvem NFC-e, e nenhuma reaproveita o host da NF-e por engano
    // (exceto GO, coberta acima).
    #[test]
    fn todas_as_ufs_resolvem_nfce_com_url_propria() {
        for (codigo, sigla) in crate::interno::uf::UFS {
            for amb in [1u8, 2] {
                let url_65 = resolver_url(amb, &chave_com(codigo, "65"))
                    .unwrap_or_else(|e| panic!("sem endpoint NFC-e p/ {sigla} amb {amb}: {e:?}"));
                let url_55 = resolver_url(amb, &chave_com(codigo, "55"))
                    .unwrap_or_else(|e| panic!("sem endpoint NF-e p/ {sigla} amb {amb}: {e:?}"));
                if *sigla != "GO" {
                    assert_ne!(url_65, url_55, "{sigla} amb {amb} usa a mesma URL p/ 55 e 65");
                }
            }
        }
    }

    #[test]
    fn rejeita_uf_desconhecida_na_chave() {
        let err = resolver_url(2, &chave_com("99", "65")).unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    // Chave de outro documento (57 = CT-e) não tem endpoint nesta crate.
    #[test]
    fn rejeita_modelo_sem_endpoint() {
        let err = resolver_url(2, &chave_com("35", "57")).unwrap_err();
        assert!(matches!(err, DfeError::Webservice(_)));
    }
}
