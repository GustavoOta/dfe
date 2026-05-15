use base64::Engine;
use flate2::read::GzDecoder;
use serde::{Deserialize, Deserializer, Serialize};
use std::io::Read;

mod service;
#[cfg(test)]
mod test;

#[derive(Debug, Serialize, Deserialize)]
pub struct DistribuicaoResposta {
    #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    #[serde(rename = "cStat")]
    pub c_stat: String,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
    #[serde(rename = "dhResp")]
    pub dh_resp: String,
    #[serde(rename = "ultNSU", default)]
    pub ult_nsu: String,
    #[serde(rename = "maxNSU", default)]
    pub max_nsu: String,
    #[serde(
        rename = "loteDistDFeInt",
        default,
        deserialize_with = "deserialize_lote_dist_dfe_int"
    )]
    pub lote_dist_dfe_int: Option<Vec<LoteDistDFeInt>>,
}

#[derive(Debug, Serialize)]
pub struct LoteDistDFeInt {
    pub nsu: String,
    pub schema: String,
    pub content: Option<ResNFe>,
    pub content_xml: Option<String>,
    pub content_raw: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResNFe {
    #[serde(rename = "chNFe")]
    pub ch_nfe: String,
    #[serde(rename = "CNPJ")]
    pub cnpj: String,
    #[serde(rename = "xNome")]
    pub x_nome: String,
    #[serde(rename = "IE")]
    pub ie: String,
    #[serde(rename = "dhEmi")]
    pub dh_emi: String,
    #[serde(rename = "tpNF")]
    pub tp_nf: String,
    #[serde(rename = "vNF")]
    pub v_nf: String,
    #[serde(rename = "digVal")]
    pub dig_val: String,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: String,
    #[serde(rename = "nProt")]
    pub n_prot: String,
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

pub struct Consulta {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub uf: u8,
    pub ambiente: u8,
    pub check_flag: Option<bool>,
}

pub struct ConsultaNSU {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub uf: u8,
    pub ambiente: u8,
    pub nsu: String,
    pub check_flag: Option<bool>,
}

pub struct ConsultaChaveAcesso {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub uf: u8,
    pub ambiente: u8,
    pub chave_acesso: String,
    pub check_flag: Option<bool>,
}

pub struct CienciaOperacao {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub ambiente: u8,
    pub chave_acesso: String,
}

pub struct ConfirmacaoOperacao {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub ambiente: u8,
    pub chave_acesso: String,
}

pub struct DesconhecimentoOperacao {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub ambiente: u8,
    pub chave_acesso: String,
}

pub struct OperacaoNaoRealizada {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub ambiente: u8,
    pub chave_acesso: String,
    pub justificativa: String,
}

pub type ManifestacaoResposta = crate::nfe::types::manifestacao::Response;

pub type Distribuicao = Consulta;
pub type DistribuicaoNSU = ConsultaNSU;
pub type DistribuicaoChaveAcesso = ConsultaChaveAcesso;

impl Consulta {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            cnpj: String::new(),
            uf: 0,
            ambiente: 0,
            check_flag: None,
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

    pub fn check_flag(mut self) -> Self {
        self.check_flag = Some(true);
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
            check_flag: None,
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
        self.check_flag = Some(true);
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
            check_flag: None,
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
        self.check_flag = Some(true);
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
