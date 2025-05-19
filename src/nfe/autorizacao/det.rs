use super::det_process::entity::*;
use super::Det;
use anyhow::{Error, Result};

pub fn det_process(prod: Vec<Det>) -> Result<Vec<DetProcess>, Error> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    for d in &prod {
        det_process_values.push(DetProcess {
            prod: ProdProcess {
                c_prod: d.c_prod.to_string(),
                c_ean: d.c_ean.to_string(),
                x_prod: d.x_prod.to_string(),
                ncm: d.ncm.to_string(),
                cfop: d.cfop.to_string(),
                cest: d.cest.clone(),
                u_com: d.u_com.to_string(),
                q_com: format!("{:.2}", d.q_com),
                v_un_com: format!("{:.2}", d.v_un_com),
                v_prod: format!("{:.2}", d.v_prod),
                c_ean_trib: d.c_ean_trib.to_string(),
                u_trib: d.u_trib.to_string(),
                q_trib: format!("{:.2}", d.q_trib),
                v_un_trib: format!("{:.2}", d.v_un_trib),
                ind_tot: d.ind_tot.to_string(),
                x_ped: d.x_ped.clone(),
                n_item_ped: d.n_item_ped.clone(),
            },
            imposto: ImpostoProcess {
                v_tot_trib: format!("{:.2}", d.v_tot_trib),
                icms: select_icms_process(d),
                pis: select_pis_process(d),
                cofins: select_cofins_process(d),
            },
            inf_ad_prod: d.inf_ad_prod.clone(),
        });
    }
    Ok(det_process_values)
}

fn select_icms_process(d: &Det) -> ICMSProcess {
    let icms = match d.icms.as_str() {
        "ICMSSN101" => {
            // Tributada pelo Simples Nacional com permissão de crédito. (v2.0)
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };

            let csosn = match d.csosn {
                Some(csosn) => csosn,
                None => return validate_icms("csosn", d),
            };
            ICMSProcess::ICMSSN101(ICMSSN101 { orig, csosn })
        }
        "ICMSSN102" => {
            // Tributada pelo Simples Nacional sem permissão de crédito e com cobrança do ICMS por substituição tributária. (v2.0)
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };
            let csosn = match d.csosn {
                Some(csosn) => csosn,
                None => return validate_icms("csosn", d),
            };
            let p_cred_sn = match d.p_cred_sn {
                Some(p_cred_sn) => p_cred_sn,
                None => return validate_icms("p_cred_sn", d),
            };
            let v_cred_icmssn = match d.v_cred_icmssn {
                Some(v_cred_icmssn) => v_cred_icmssn,
                None => return validate_icms("v_cred_icmssn", d),
            };
            ICMSProcess::ICMSSN102(ICMSSN102 {
                orig,
                csosn,
                p_cred_sn,
                v_cred_icmssn,
            })
        }
        "ICMS00" => {
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };
            let cst = match d.cst.clone() {
                Some(cst) => cst,
                None => return validate_icms("cst", d),
            };
            let mod_bc = match d.mod_bc {
                Some(mod_bc) => mod_bc,
                None => return validate_icms("mod_bc", d),
            };
            let v_bc = match d.v_bc {
                Some(v_bc) => v_bc,
                None => return validate_icms("v_bc", d),
            };
            let p_icms = match d.p_icms {
                Some(p_icms) => p_icms,
                None => return validate_icms("p_icms", d),
            };
            let v_icms = match d.v_icms {
                Some(v_icms) => v_icms,
                None => return validate_icms("v_icms", d),
            };
            ICMSProcess::ICMS00(ICMS00 {
                orig,
                cst,
                mod_bc,
                v_bc,
                p_icms,
                v_icms,
            })
        }
        "ICMS40" => ICMSProcess::ICMS40(ICMS40 {
            orig: 0,
            cst: 41,
            ..Default::default()
        }),
        "ICMS90" => {
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };
            let cst = match d.cst.clone() {
                Some(cst) => cst,
                None => return validate_icms("cst", d),
            };
            ICMSProcess::ICMS90(ICMS90 { orig, cst })
        }
        _ => {
            return ICMSProcess::ICMSError(format!("Unsupported ICMS type: {}", d.icms));
        }
    };
    return icms;
}

fn select_pis_process(d: &Det) -> PISProcess {
    match d.pis.as_str() {
        "PISAliq" => PISProcess {
            pis_aliq: Some(PISAliq {
                cst: validade_pis_cst(d.pis_cst.clone()),
                v_bc: d.pis_v_bc.unwrap_or(0.0),
                p_pis: d.pis_p_pis.unwrap_or(0.0),
                v_pis: d.pis_v_pis.unwrap_or(0.0),
            }),
            ..Default::default()
        },
        "PISOutr" => PISProcess {
            pis_outr: Some(PISOutr {
                cst: "99".to_string(),
                qbc_prod: Some("0.00".to_string()),
                valiq_prod: Some("0.00".to_string()),
                vpis: Some("0.00".to_string()),
            }),
            ..Default::default()
        },
        _ => PISProcess {
            pis_invalid: Some(d.pis.clone()),
            ..Default::default()
        },
    }
}

fn select_cofins_process(d: &Det) -> COFINSProcess {
    match d.cofins.as_str() {
        "COFINSAliq" => COFINSProcess {
            cofins_aliq: Some(COFINSAliq {
                cst: validade_cofins_cst(d.cofins_cst.clone()),
                v_bc: d.cofins_v_bc.unwrap_or(0.0),
                p_cofins: d.cofins_p_cofins.unwrap_or(0.0),
                v_cofins: d.cofins_v_cofins.unwrap_or(0.0),
            }),
            ..Default::default()
        },
        "COFINSOutr" => COFINSProcess {
            cofins_outr: Some(COFINSOutr {
                cst: d
                    .cofins_cst
                    .clone()
                    .unwrap_or("cofins cst inválido.".to_string()),
                vbc: d.cofins_v_bc,
                p_cofins: d.cofins_p_cofins,
                qbc_prod: d.cofins_q_bc_prod,
                valiq_prod: d.cofins_v_aliq_prod,
                vcofins: d.cofins_v_cofins,
            }),
            ..Default::default()
        },
        &_ => COFINSProcess {
            cofins_invalid: Some(d.cofins.clone()),
            ..Default::default()
        },
    }
}

fn validate_icms(variant: &str, d: &Det) -> ICMSProcess {
    match variant {
        "orig" => ICMSProcess::ICMSError(format!(
            "O código de origem do produto não foi identificado. Cód. recebido: {:?}",
            d.orig
        )),
        "csosn" => ICMSProcess::ICMSError(format!("O CSOSN é inválido: {:?}", d.csosn)),
        "p_cred_sn" => {
            ICMSProcess::ICMSError(format!("O p_cred_sn não é válido: {:?}", d.p_cred_sn))
        }
        "v_cred_icmssn" => ICMSProcess::ICMSError(format!(
            "O v_cred_icmssn não é válido: {:?}",
            d.v_cred_icmssn
        )),
        "cst" => ICMSProcess::ICMSError(format!("CST não informado: {:?}", d.cst)),
        "mod_bc" => ICMSProcess::ICMSError(format!("mod_bc não informado: {:?}", d.mod_bc)),
        &_ => ICMSProcess::ICMSError(format!(
            "Erro não identificado na validação do ICMS do produto no campo: {}",
            variant
        )),
    }
}

fn validade_pis_cst(pis_cst_opt: Option<String>) -> String {
    match pis_cst_opt {
        Some(cst) => cst,
        None => "PIS CST não recebido, verifique se pis_cst foi enviado.".to_string(), // or handle as needed
    }
}

fn validade_cofins_cst(pis_cst_opt: Option<String>) -> String {
    match pis_cst_opt {
        Some(cst) => cst,
        None => "COFINS CST não recebido, verifique se cofins_cst foi enviado.".to_string(), // or handle as needed
    }
}
/*

ERro: duplicando vetor por extend
pub fn det_process(prod: Vec<Det>) -> Result<Vec<DetProcess>, Error> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    for d in &prod {
        let icms = match d.icms.as_str() {
            "ICMSSN102" => ICMSProcess::ICMSSN102(ICMSSN102 {
                orig: 0,
                csosn: 102,
            }),
            "ICMS40" => ICMSProcess::ICMS40(ICMS40 {
                orig: 0,
                cst: 41,
                ..Default::default()
            }),
            _ => return Err(Error::msg("Unsupported ICMS type")),
        };

        det_process_values.extend(
            prod.iter()
                .map(|d| DetProcess {
                    prod: ProdProcess {
                        c_prod: d.c_prod.to_string(),
                        c_ean: d.c_ean.to_string(),
                        x_prod: d.x_prod.to_string(),
                        ncm: d.ncm.to_string(),
                        cfop: d.cfop.to_string(),
                        cest: d.cest.clone(),
                        u_com: d.u_com.to_string(),
                        q_com: format!("{:.2}", d.q_com),
                        v_un_com: format!("{:.2}", d.v_un_com),
                        v_prod: format!("{:.2}", d.v_prod),
                        c_ean_trib: d.c_ean_trib.to_string(),
                        u_trib: d.u_trib.to_string(),
                        q_trib: format!("{:.2}", d.q_trib),
                        v_un_trib: format!("{:.2}", d.v_un_trib),
                        ind_tot: d.ind_tot.to_string(),
                    },
                    imposto: ImpostoProcess {
                        v_tot_trib: format!("{:.2}", d.v_tot_trib),
                        icms: icms.clone(),
                        pis: PISProcess {
                            pis_outr: PISOutr {
                                cst: 99.to_string(),
                                qbc_prod: Some("0.00".to_string()),
                                valiq_prod: Some("0.00".to_string()),
                                vpis: Some("0.00".to_string()),
                            },
                        },
                        cofins: COFINSProcess {
                            cofins_outr: COFINSOutr {
                                cst: 99.to_string(),
                                qbc_prod: "0.00".to_string(),
                                valiq_prod: "0.00".to_string(),
                                vcofins: "0.00".to_string(),
                            },
                        },
                    },
                })
                .collect::<Vec<_>>(),
        );
    }
    Ok(det_process_values)
} */
