//! Structs de IPI do XML de emissão (A7: extraído de entity.rs).

use serde::{Deserialize, Serialize};
use super::{serialize_f64_2_decimals, serialize_option_f64_2_decimals, serialize_option_f64_4_decimals};


// ─── IPI ──────────────────────────────────────────────────────────────────────

/// IPITrib — CST de saída tributada (50, 99 ...)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPITrib {
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "vBC", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_bc: Option<f64>,
    #[serde(rename = "pIPI", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_ipi: Option<f64>,
    #[serde(rename = "qBCProd", skip_serializing_if = "Option::is_none")]
    pub q_bc_prod: Option<String>,
    #[serde(rename = "vAliqProd", skip_serializing_if = "Option::is_none")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vIPI", serialize_with = "serialize_f64_2_decimals")]
    pub v_ipi: f64,
}


/// IPINT — CST de saída não tributada
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPINT {
    #[serde(rename = "CST")]
    pub cst: String,
}


/// Container do IPI — serializado manualmente em to_xml() para controle das tags
#[derive(Debug, Clone)]
pub struct IpiProcess {
    pub c_enq: String,
    pub c_selo: Option<String>,
    pub q_selo: Option<u32>,
    pub tributado: bool, // true = IPITrib, false = IPINT
    pub inner: String,   // XML pré-serializado da variante interna
}
