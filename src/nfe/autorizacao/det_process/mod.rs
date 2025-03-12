use anyhow::{Error, Result};
pub mod entity;
use super::Det;
use entity::*;

#[allow(dead_code)]
pub fn icmssn101(d: Det) -> Result<ICMSProcess, Error> {
    // Tributada pelo Simples Nacional com permissão de crédito. (v2.0)
    let orig = match d.orig {
        Some(orig) => orig,
        None => return Err(Error::msg("Origem (var orig) da mercadoria não informada")),
    };

    let csosn = match d.csosn {
        Some(csosn) => csosn,
        None => return Err(Error::msg("CSOSN (var csosn) não informado para ICMSSN101")),
    };
    //ICMSProcess::ICMSSN101(ICMSSN101 { orig, csosn })
    Ok(ICMSProcess::ICMSSN101(ICMSSN101 { orig, csosn }))
}
