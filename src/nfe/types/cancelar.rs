use serde::{Deserialize, Serialize};

/// Estrutura para cancelar uma NFe
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFeCancelar {
    /// Caminho do certificado digital .pfx
    /// Exemplo: "D:/Projetos/cert.pfx"
    pub cert_path: String,
    /// Senha do certificado digital
    /// Exemplo: "1234"
    pub cert_pass: String,
    /// Tipo de ambiente (1 - Produção, 2 - Homologação)
    pub tp_amb: u8,
    /// Chave de acesso da NFe
    /// Exemplo: "35241211111111111111550010000000361491395167"
    pub chave: String,
    /// Número do protocolo de autorização da NFe
    /// Exemplo: "135190000000000"
    pub protocolo: String,
    /// Justificativa para o cancelamento
    /// Exemplo: "Nota fiscal duplicada e cancelada ou produto equivocado"
    pub justificativa: String,
}

/// Estrutura de resposta do cancelamento de uma NFe
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
}
