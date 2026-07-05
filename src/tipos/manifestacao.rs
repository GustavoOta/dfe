use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub response: InfEvento,
    pub send_xml: String,
    pub receive_xml: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfEvento {
    #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    #[serde(rename = "cOrgao")]
    pub c_orgao: String,
    #[serde(rename = "cStat")]
    pub c_stat: String,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
    #[serde(rename = "chNFe")]
    pub ch_nfe: String,
    #[serde(rename = "tpEvento")]
    pub tp_evento: String,
    #[serde(rename = "nSeqEvento")]
    pub n_seq_evento: String,
    #[serde(rename = "dhRegEvento")]
    pub dh_reg_evento: String,
    #[serde(rename = "xEvento", default)]
    pub x_evento: String,
    #[serde(rename = "CNPJDest", default)]
    pub cnpj_dest: String,
    #[serde(rename = "nProt", default)]
    pub n_prot: String,
}
