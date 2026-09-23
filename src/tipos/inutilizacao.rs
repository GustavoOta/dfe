use serde::{Deserialize, Serialize};

/// Resposta da inutilização de faixa de numeração (`inutNFe`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub response: InfInut,
    pub send_xml: String,
    pub receive_xml: String,
}

/// `retInutNFe/infInut` — retorno do pedido de inutilização.
///
/// `cStat 102` = "Inutilização de número homologado" (único sucesso). Qualquer outro código é
/// recusa — as mais comuns são a faixa já ter sido usada por uma nota autorizada ou já estar
/// inutilizada.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfInut {
    #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    #[serde(rename = "cStat")]
    pub c_stat: String,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
    #[serde(rename = "cUF")]
    pub c_uf: String,
    /// Campos abaixo só voltam quando o pedido é homologado — daí serem opcionais.
    #[serde(rename = "ano", default)]
    pub ano: Option<String>,
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,
    #[serde(rename = "mod", default)]
    pub mod_: Option<String>,
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,
    #[serde(rename = "nNFIni", default)]
    pub n_nf_ini: Option<String>,
    #[serde(rename = "nNFFin", default)]
    pub n_nf_fin: Option<String>,
    #[serde(rename = "dhRecbto", default)]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}
