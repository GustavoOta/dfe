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
    #[serde(rename = "indPag")]
    pub ind_pag: u8,
    #[serde(rename = "tPag")]
    pub t_pag: String,
    #[serde(rename = "vPag")]
    pub v_pag: String,
}

pub fn pag_process(pag: Pag) -> Result<PagProcess, Error> {
    let pag_process = PagProcess {
        det_pag: DetPag {
            ind_pag: pag.ind_pag,
            t_pag: format!("{}", pag.t_pag),
            v_pag: format!("{:.2}", pag.v_pag),
        },
    };
    Ok(pag_process)
}
