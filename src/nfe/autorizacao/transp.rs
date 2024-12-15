use super::Transp;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "transp")]
pub struct TranspProcess {
    #[serde(rename = "modFrete")]
    pub mod_frete: String,
}

pub fn transp_process(transp: Transp) -> Result<TranspProcess, Error> {
    let transp_process = TranspProcess {
        mod_frete: format!("{}", transp.mod_frete),
    };
    Ok(transp_process)
}
