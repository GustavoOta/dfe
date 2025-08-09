//use anyhow::{Error, Result};
pub mod entity;
////use super::Det;
//use entity::*;

/* pub fn icmssn101(d: Det) -> Result<ICMSProcess, Error> {
    // Tributada pelo Simples Nacional com permissão de crédito. (v2.0)
    let orig = match d.orig {
        Some(orig) => orig,
        None => return Err(Error::msg("Origem (var orig) da mercadoria não informada")),
    };

    let csosn = match d.csosn {
        Some(csosn) => csosn,
        None => return Err(Error::msg("CSOSN (var csosn) não informado para ICMSSN101")),
    };

    let p_cred_sn = match d.p_cred_sn {
        Some(p_cred_sn) => {
            format!("{:.2}", p_cred_sn)
        },
        None => return Err(Error::msg("Percentual de crédito do Simples Nacional (var p_cred_sn) não informado para ICMSSN101")),
    };
    let v_cred_icmssn = match d.v_cred_icmssn {
        Some(v_cred_icmssn) => {
            format!("{:.2}", v_cred_icmssn)
        },
        None => return Err(Error::msg("Valor de crédito do ICMS do Simples Nacional (var v_cred_icmssn) não informado para ICMSSN101")),
    };
    //ICMSProcess::ICMSSN101(ICMSSN101 { orig, csosn })
    Ok(ICMSProcess::ICMSSN101(ICMSSN101 {
        orig,
        csosn,
        p_cred_sn,
        v_cred_icmssn,
    }))
}
 */
