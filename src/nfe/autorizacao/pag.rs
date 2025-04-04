use super::Pag;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "pag")]
pub struct PagProcess {
    #[serde(rename = "detPag")]
    pub det_pag: DetPag,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetPag {
    #[serde(rename = "tPag")]
    pub t_pag: String,
    #[serde(rename = "vPag")]
    pub v_pag: String,
    #[serde(rename = "xPag", skip_serializing_if = "Option::is_none")]
    pub x_pag: Option<String>,
}

pub fn pag_process(pag: Pag) -> Result<PagProcess, Error> {
    let pag_process = PagProcess {
        det_pag: DetPag {
            t_pag: format!("{}", pag.t_pag),
            v_pag: format!("{:.2}", pag.v_pag),
            x_pag: pag.x_pag,
        },
    };
    Ok(pag_process)
}
