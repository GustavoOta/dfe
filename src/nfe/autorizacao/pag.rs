use super::NFe;
use anyhow::{Error, Result};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "pag")]
pub struct PagProcess {
    #[serde(rename = "detPag")]
    pub det_pag: DetPag,
    #[serde(rename = "vTroco", skip_serializing_if = "Option::is_none")]
    pub v_troco: Option<Decimal>,
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
    pub v_troco: Option<Decimal>,
}

pub fn pag_process(nfe: NFe) -> Result<PagProcess, Error> {
    let card = if nfe.pag.t_pag == "03" || nfe.pag.t_pag == "04" || nfe.pag.t_pag == "17" {
        if nfe.pag.tp_integra == Some(1) {
            Some(Card {
                tp_integra: 1,
                cnpj: Some(nfe.pag.cnpj.clone().unwrap_or_default()),
                t_band: Some(nfe.pag.t_band.clone().unwrap_or_default()),
                c_aut: Some(nfe.pag.c_aut.clone().unwrap_or_default()),
                v_troco: Some(nfe.pag.v_troco.unwrap_or_default()),
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

    let mut troco = Decimal::new(0, 2);
    let v_nf = Decimal::from_f64(nfe.total.v_nf).unwrap_or_default();
    let v_pag = Decimal::from_f64(nfe.pag.v_pag).unwrap_or_default();
    if v_pag > v_nf {
        troco = v_pag - v_nf;
    }

    let pag_process = PagProcess {
        det_pag: DetPag {
            ind_pag: nfe.pag.ind_pag,
            t_pag: format!("{}", nfe.pag.t_pag),
            x_pag: nfe.pag.x_pag.clone(),
            v_pag: format!("{:.2}", v_pag),
            card,
        },
        v_troco: Some(
            format!("{:.2}", troco.round_dp(2))
                .parse::<Decimal>()
                .unwrap_or_default(),
        ),
    };
    Ok(pag_process)
}
