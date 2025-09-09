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

    #[serde(rename = "xPag", skip_serializing_if = "Option::is_none")]
    pub x_pag: Option<String>,

    #[serde(rename = "vPag")]
    pub v_pag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<Card>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Card {
    #[serde(rename = "tpIntegra")]
    pub tp_integra: u8,
    #[serde(rename = "CNPJ", skip_serializing_if = "Option::is_none")]
    pub cnpj: Option<String>,
    #[serde(rename = "tBand", skip_serializing_if = "Option::is_none")]
    pub t_band: Option<String>,
    #[serde(rename = "cAut", skip_serializing_if = "Option::is_none")]
    pub c_aut: Option<String>,
    #[serde(rename = "vTroco", skip_serializing_if = "Option::is_none")]
    pub v_troco: Option<String>,
}

pub fn pag_process(pag: Pag) -> Result<PagProcess, Error> {
    let card = if pag.t_pag == "03" || pag.t_pag == "04" || pag.t_pag == "17" {
        if pag.tp_integra == Some(1) {
            Some(Card {
                tp_integra: 1,
                cnpj: Some(pag.cnpj.clone().unwrap_or_default()),
                t_band: Some(pag.t_band.clone().unwrap_or_default()),
                c_aut: Some(pag.c_aut.clone().unwrap_or_default()),
                v_troco: Some(pag.v_troco.clone().unwrap_or_default()),
            })
        } else {
            Some(Card {
                tp_integra: 2,
                cnpj: None,
                t_band: None,
                c_aut: None,
                v_troco: None,
            })
        }
    } else {
        None
    };

    let pag_process = PagProcess {
        det_pag: DetPag {
            ind_pag: pag.ind_pag,
            t_pag: format!("{}", pag.t_pag),
            x_pag: pag.x_pag.clone(),
            v_pag: format!("{:.2}", pag.v_pag),
            card,
        },
    };
    Ok(pag_process)
}
