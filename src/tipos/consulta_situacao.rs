use serde::{Deserialize, Serialize};

/// Resposta da consulta de situação de uma NF-e/NFC-e (`consSitNFe`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub response: RetConsSitNFe,
    pub send_xml: String,
    pub receive_xml: String,
}

/// `retConsSitNFe` — retorno do pedido de consulta da situação atual da NF-e.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetConsSitNFe {
    #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    /// Status da **consulta** (ex.: `217` = NF-e não consta na base de dados da SEFAZ).
    #[serde(rename = "cStat")]
    pub c_stat: String,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
    #[serde(rename = "cUF")]
    pub c_uf: String,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: String,
    #[serde(rename = "chNFe")]
    pub ch_nfe: String,
    /// Protocolo de autorização — presente só quando a nota consta na base (`cStat` de
    /// autorização, ex. `100`).
    #[serde(rename = "protNFe", default)]
    pub prot_nfe: Option<ProtNFe>,
    /// Eventos já registrados contra a chave (ex.: cancelamento `135`/`110111`).
    #[serde(rename = "procEventoNFe", default)]
    pub proc_evento_nfe: Vec<ProcEventoNFe>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtNFe {
    #[serde(rename = "infProt")]
    pub inf_prot: InfProt,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfProt {
    #[serde(rename = "chNFe")]
    pub ch_nfe: String,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: String,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
    #[serde(rename = "digVal", default)]
    pub dig_val: Option<String>,
    /// Status da **nota** (ex.: `100` = autorizado o uso da NF-e, `101` = cancelada).
    #[serde(rename = "cStat")]
    pub c_stat: String,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcEventoNFe {
    #[serde(rename = "retEvento")]
    pub ret_evento: RetEventoInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetEventoInfo {
    #[serde(rename = "infEvento")]
    pub inf_evento: RetEventoInfEvento,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetEventoInfEvento {
    #[serde(rename = "tpEvento")]
    pub tp_evento: String,
    #[serde(rename = "cStat")]
    pub c_stat: String,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
    #[serde(rename = "nProt", default)]
    pub n_prot: Option<String>,
}
