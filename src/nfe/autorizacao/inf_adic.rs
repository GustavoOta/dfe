use super::InfAdic;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "infAdic")]
pub struct InfAdicProcess {
    #[serde(rename = "infCpl")]
    pub inf_cpl: String,
}

pub fn inf_adic_process(inf_adic: Option<InfAdic>) -> Result<InfAdicProcess, Error> {
    let inf_adic_process = InfAdicProcess {
        inf_cpl: inf_adic
            .and_then(|iad| iad.inf_cpl)
            .unwrap_or_else(|| "Sem informações adicionais".to_string()),
    };
    Ok(inf_adic_process)
}
