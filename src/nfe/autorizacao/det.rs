use super::det_process::entity::*;
use super::Det;
use anyhow::{Error, Result};

pub fn det_process(
    prod: Vec<Det>,
    mod_: u32,
    tp_amb: u8,
    active_ibscbs: Option<String>,
) -> Result<Vec<DetProcess>, Error> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    let mut first_item = 0;

    for d in &prod {
        let mut x_prod = d.x_prod.clone();
        if first_item == 0 {
            if mod_ == 65 && tp_amb == 2 {
                x_prod =
                    "NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string();
            }
        }
        first_item += 1;
        det_process_values.push(DetProcess {
            prod: ProdProcess {
                c_prod: d.c_prod.to_string(),
                c_ean: d.c_ean.to_string(),
                x_prod: x_prod.clone(),
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
                ibs_cbs: ibs_cbs_process(d, tp_amb, active_ibscbs.clone()),
            },
            inf_ad_prod: d.inf_ad_prod.clone(),
        });
    }
    Ok(det_process_values)
}

fn ibs_cbs_process(d: &Det, _tp_amb: u8, active_ibscbs: Option<String>) -> Option<IBSCBSProcess> {
    let send_ibscbs = Some(IBSCBSProcess {
        cst: d.ibs_cbs_cst.clone(),
        c_class_trib: d.ibs_cbs_class_trib.clone(),
        g_ibscbs: GIBSCBS {
            v_bc: format!("{:.2}", d.ibs_cbs_v_bc),
            g_ibs_uf: GIBSUF {
                p_ibs_uf: format!("{:.4}", d.p_ibs_uf.clone()), // 0,1%
                v_ibs_uf: format!("{:.2}", d.v_ibs_uf.clone()),
                ..Default::default()
            },
            g_ibs_mun: GIBSMun {
                p_ibs_mun: format!("{:.4}", d.p_ibs_mun.clone()), // 0,0%
                v_ibs_mun: format!("{:.2}", d.v_ibs_mun.clone()),
                ..Default::default()
            },
            v_ibs: format!("{:.2}", d.v_ibs_uf + d.v_ibs_mun),
            g_cbs: GCBS {
                p_cbs: format!("{:.4}", d.p_cbs.clone()), // 0,9%
                v_cbs: format!("{:.2}", d.v_cbs.clone()),
                ..Default::default()
            },
            ..Default::default()
        },
    });

    // Se a data atual for menor que 2026-01-01, não enviar o IBSCBS
    // verificar se a data do sistema é maior ou igual a  que 2026-01-01
    let data_limite =
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("Data limite inválida para IBSCBS");
    let data_atual = chrono::Local::now().naive_local().date();
    if data_atual >= data_limite {
        return send_ibscbs;
    }
    if let Some(_) = active_ibscbs {
        return None;
    } else {
        return send_ibscbs;
    }
}

fn select_icms_process(d: &Det) -> ICMSProcess {
    //println!("Processing ICMS for product on DFE RAW: {:?}", d);
    let icms = match d.icms.as_str() {
        "ICMSSN101" => {
            // Tributada pelo Simples Nacional com permissão de crédito. (v2.0)
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };

            let csosn = match d.csosn.clone() {
                Some(csosn) => csosn,
                None => return validate_icms("csosn", d),
            };

            let p_cred_sn = match d.p_cred_sn {
                Some(p_cred_sn) => {
                    format!("{:.2}", p_cred_sn)
                }
                None => return validate_icms("p_cred_sn", d),
            };
            let v_cred_icmssn = match d.v_cred_icmssn {
                Some(v_cred_icmssn) => {
                    format!("{:.2}", v_cred_icmssn)
                }
                None => return validate_icms("v_cred_icmssn", d),
            };
            ICMSProcess::ICMSSN101(ICMSSN101 {
                orig,
                csosn,
                p_cred_sn,
                v_cred_icmssn,
            })
        }
        "ICMSSN102" => {
            // Tributada pelo Simples Nacional sem permissão de crédito e com cobrança do ICMS por substituição tributária. (v2.0)
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };
            let csosn = match d.csosn.clone() {
                Some(csosn) => csosn,
                None => return validate_icms("csosn", d),
            };

            ICMSProcess::ICMSSN102(ICMSSN102 { orig, csosn })
        }
        "ICMSSN500" => {
            // ICMS cobrado anteriormente por substituição tributária (v2.0)
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };
            let csosn = match d.csosn.clone() {
                Some(csosn) => csosn,
                None => return validate_icms("csosn", d),
            };

            ICMSProcess::ICMSSN500(ICMSSN500 {
                orig,
                csosn,
                ..Default::default()
            })
        }
        "ICMSSN900" => {
            // Tributada pelo Simples Nacional com permissão de crédito e com cobrança do ICMS por substituição tributária. (v2.0)
            let orig = match d.orig {
                Some(orig) => orig,
                None => return validate_icms("orig", d),
            };
            let csosn = match d.csosn.clone() {
                Some(csosn) => csosn,
                None => return validate_icms("csosn", d),
            };

            ICMSProcess::ICMSSN900(ICMSSN900 {
                orig,
                csosn,
                ..Default::default()
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
            println!("Unsupported ICMS type on DFE RAW: {}", d.icms);
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
                qbc_prod: Some("0.00".to_string()), // informando 0 por calculo em valor
                valiq_prod: Some("0.00".to_string()), // informando 0 por calculo em valor
                vpis: Some("0.00".to_string()),     // informando 0 por calculo em valor
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
                v_bc: Some(0.0),
                p_cofins: Some(0.0),
                v_cofins: Some(0.0),
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
