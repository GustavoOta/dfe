use super::det_process::entity::*;
use crate::tipos::{Cofins, Det, IbsCbs, Icms, Ipi, Pis};
use crate::error::Result;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub fn det_process(
    prod: Vec<Det>,
    _mod_: u32,
    tp_amb: u8,
    desconto_rateio: Option<Decimal>,
    _active_ibscbs: Option<String>,
) -> Result<Vec<DetProcess>> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    let mut first_item = 0;

    // desconto por rateio nos itens *********************************************************
    let desconto_rateado = if desconto_rateio.is_some() {
        desconto_rateio.unwrap()
    } else {
        Decimal::new(0, 2)
    };

    let mut total_produtos = Decimal::new(0, 2);
    for d in &prod {
        // soma o valor dos produtos para calcular o percentual do desconto
        total_produtos += Decimal::from_f64(d.v_prod).unwrap_or(Decimal::new(0, 2));
    }

    // vamos definir a porcentagem do desconto, temos o valor total dos produtos e o valor do desconto
    // exemplo: total produtos = 1000, desconto = 50, percentual = 50/1000 = 0.05 (5%)
    let desconto_percentual = if total_produtos > Decimal::new(0, 2) {
        desconto_rateado / total_produtos
    } else {
        Decimal::new(0, 2)
    };

    // com o valor do percentual, vamos calcular o valor do desconto para cada item
    // armazenar cada desconto em um vetor para aplicar depois
    let mut descontos_itens: Vec<Decimal> = Vec::new();
    for d in &prod {
        let v_prod_decimal = Decimal::from_f64(d.v_prod).unwrap_or(Decimal::new(0, 2));
        let desconto_item = (v_prod_decimal * desconto_percentual).round_dp(2);
        descontos_itens.push(desconto_item);
    }

    // verificar se a soma dos desconto é igual ao desconto total, se não for, ajustar o último item
    let soma_descontos: Decimal = descontos_itens.iter().cloned().sum();
    if soma_descontos != desconto_rateado {
        let diferenca = desconto_rateado - soma_descontos;
        if let Some(last) = descontos_itens.last_mut() {
            *last += diferenca;
        }
    }
    /* println!(
        "Desconto por rateio aplicado nos itens: {:?}",
        descontos_itens
    ); */
    // fim do desconto por rateio nos itens **************************************************

    for (d, desconto_item) in prod.iter().zip(descontos_itens.iter()) {
        let mut x_prod = d.x_prod.clone();
        // SEFAZ exige texto fixo no primeiro item em homologação (mod 55 e 65)
        if first_item == 0 && tp_amb == 2 {
            x_prod = "NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string();
        }
        first_item += 1;

        // pegar o valor do desconto do item
        let v_desc_value: Option<Decimal> = if *desconto_item > Decimal::new(0, 2) {
            Some(*desconto_item)
        } else {
            None
        };

        det_process_values.push(DetProcess {
            prod: ProdProcess {
                c_prod: d.c_prod.to_string(),
                c_ean: d.c_ean.to_string(),
                x_prod: x_prod.clone(),
                ncm: d.ncm.to_string(),
                cfop: d.cfop.to_string(),
                cest: d.cest.clone(),
                u_com: d.u_com.to_string(),
                q_com: format!("{:.3}", d.q_com),
                v_un_com: format!("{:.2}", d.v_un_com),
                v_prod: format!("{:.2}", d.v_prod),
                c_ean_trib: d.c_ean_trib.to_string(),
                u_trib: d.u_trib.to_string(),
                q_trib: format!("{:.3}", d.q_trib),
                v_un_trib: format!("{:.2}", d.v_un_trib),
                v_desc: v_desc_value,
                ind_tot: d.ind_tot.to_string(),
                x_ped: d.x_ped.clone(),
                n_item_ped: d.n_item_ped.clone(),
            },
            imposto: ImpostoProcess {
                v_tot_trib: format!("{:.2}", d.v_tot_trib),
                icms: select_icms_process(&d.icms),
                ipi: d.ipi.as_ref().map(select_ipi_process),
                pis: select_pis_process(&d.pis),
                cofins: select_cofins_process(&d.cofins),
                ibs_cbs: ibs_cbs_process(d.ibs_cbs.as_ref()),
            },
            inf_ad_prod: d.inf_ad_prod.clone(),
        });
    }
    Ok(det_process_values)
}

fn ibs_cbs_process(ibs_cbs: Option<&IbsCbs>) -> Option<IBSCBSProcess> {
    let ibs = ibs_cbs?;
    Some(IBSCBSProcess {
        cst: ibs.cst.clone(),
        c_class_trib: ibs.class_trib.clone(),
        g_ibscbs: GIBSCBS {
            v_bc: format!("{:.2}", ibs.v_bc),
            g_ibs_uf: GIBSUF {
                p_ibs_uf: format!("{:.4}", ibs.p_ibs_uf),
                v_ibs_uf: format!("{:.2}", ibs.v_ibs_uf),
                ..Default::default()
            },
            g_ibs_mun: GIBSMun {
                p_ibs_mun: format!("{:.4}", ibs.p_ibs_mun),
                v_ibs_mun: format!("{:.2}", ibs.v_ibs_mun),
                ..Default::default()
            },
            v_ibs: format!("{:.2}", ibs.v_ibs_uf + ibs.v_ibs_mun),
            g_cbs: GCBS {
                p_cbs: format!("{:.4}", ibs.p_cbs),
                v_cbs: format!("{:.2}", ibs.v_cbs),
                ..Default::default()
            },
            ..Default::default()
        },
    })
}

fn select_icms_process(icms: &Icms) -> ICMSProcess {
    match icms {
        Icms::Icms00 { orig, mod_bc, v_bc, p_icms, v_icms } =>
            ICMSProcess::ICMS00(ICMS00 {
                orig: *orig, cst: "00".to_string(), mod_bc: *mod_bc,
                v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
            }),

        Icms::Icms10 { orig, mod_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst } =>
            ICMSProcess::ICMS10(ICMS10 {
                orig: *orig, cst: "10".to_string(), mod_bc: *mod_bc,
                v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
                mod_bcst: *mod_bcst, p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
            }),

        Icms::Icms20 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS20(ICMS20 {
                orig: *orig, cst: "20".to_string(), mod_bc: *mod_bc,
                p_red_bc: *p_red_bc, v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
            }),

        Icms::Icms30 { orig, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS30(ICMS30 {
                orig: *orig, cst: "30".to_string(), mod_bcst: *mod_bcst,
                p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
            }),

        Icms::Icms40 { orig, cst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS40(ICMS40 {
                orig: *orig, cst: *cst, vicmsdeson: *v_icms_deson, mot_des_icms: *mot_des_icms,
            }),

        Icms::Icms51 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms_op, p_dif, v_icms_dif, v_icms } =>
            ICMSProcess::ICMS51(ICMS51 {
                orig: *orig, cst: "51".to_string(), mod_bc: *mod_bc,
                p_red_bc: *p_red_bc, v_bc: *v_bc, p_icms: *p_icms,
                v_icms_op: *v_icms_op, p_dif: *p_dif, v_icms_dif: *v_icms_dif,
                v_icms: *v_icms,
            }),

        Icms::Icms60 { orig, v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret } => {
            // xs:sequence minOccurs="0": todos presentes ou nenhum (NT 2011/004)
            let bcst   = v_bcst_ret.filter(|&v| v > 0.0);
            let pst    = p_st.filter(|&v| v > 0.0);
            let subst  = v_icms_substituto.filter(|&v| v > 0.0);
            let icmsst = v_icmsst_ret.filter(|&v| v > 0.0);
            let (v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret) =
                if bcst.is_some() || pst.is_some() || icmsst.is_some() {
                    (
                        Some(format!("{:.2}", bcst.unwrap_or(0.0))),
                        Some(format!("{:.2}", pst.unwrap_or(0.0))),
                        subst.map(|v| format!("{:.2}", v)),
                        Some(format!("{:.2}", icmsst.unwrap_or(0.0))),
                    )
                } else {
                    (None, None, None, None)
                };
            ICMSProcess::ICMS60(ICMS60 {
                orig: *orig, cst: "60".to_string(),
                v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret,
            })
        }

        Icms::Icms70 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS70(ICMS70 {
                orig: *orig, cst: "70".to_string(), mod_bc: *mod_bc,
                p_red_bc: *p_red_bc, v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
                mod_bcst: *mod_bcst, p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
            }),

        Icms::Icms90 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS90(ICMS90 {
                orig: *orig, cst: "90".to_string(),
                mod_bc: *mod_bc, p_red_bc: *p_red_bc, v_bc: *v_bc,
                p_icms: *p_icms, v_icms: *v_icms,
                mod_bcst: *mod_bcst, p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
            }),

        Icms::Sn101 { orig, p_cred_sn, v_cred_icmssn } =>
            ICMSProcess::ICMSSN101(ICMSSN101 {
                orig: *orig, csosn: "101".to_string(),
                p_cred_sn: format!("{:.2}", p_cred_sn),
                v_cred_icmssn: format!("{:.2}", v_cred_icmssn),
            }),

        Icms::Sn102 { orig, csosn } =>
            ICMSProcess::ICMSSN102(ICMSSN102 { orig: *orig, csosn: csosn.clone() }),

        Icms::Sn500 { orig, v_bcst_ret, v_icmsst_ret } =>
            ICMSProcess::ICMSSN500(ICMSSN500 {
                orig: *orig, csosn: "500".to_string(),
                vbcst_ret: v_bcst_ret.map(|v| format!("{:.2}", v)),
                vicmsst_ret: v_icmsst_ret.map(|v| format!("{:.2}", v)),
            }),

        Icms::Sn900 { orig, mod_bc, v_bc, p_red_bc, p_icms, v_icms, p_cred_sn, v_cred_icmssn,
                      mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst } =>
            ICMSProcess::ICMSSN900(ICMSSN900 {
                orig: *orig, csosn: "900".to_string(),
                modbc: mod_bc.map(|v| v.to_string()),
                vbc: v_bc.map(|v| format!("{:.2}", v)),
                pred_bc: p_red_bc.map(|v| format!("{:.4}", v)),
                picms: p_icms.map(|v| format!("{:.4}", v)),
                vicms: v_icms.map(|v| format!("{:.2}", v)),
                pcred_sn: p_cred_sn.map(|v| format!("{:.4}", v)),
                vcred_icmssn: v_cred_icmssn.map(|v| format!("{:.2}", v)),
                modbcst: mod_bcst.map(|v| v.to_string()),
                pmvast: p_mvast.map(|v| format!("{:.4}", v)),
                pred_bcst: p_red_bcst.map(|v| format!("{:.4}", v)),
                vbcst: v_bcst.map(|v| format!("{:.2}", v)),
                picmsst: p_icmsst.map(|v| format!("{:.4}", v)),
                vicmsst: v_icmsst.map(|v| format!("{:.2}", v)),
                ..Default::default()
            }),
    }
}

fn select_pis_process(pis: &Pis) -> PISProcess {
    match pis {
        Pis::Aliq { cst, v_bc, p_pis, v_pis } => PISProcess {
            pis_aliq: Some(PISAliq { cst: cst.clone(), v_bc: *v_bc, p_pis: *p_pis, v_pis: *v_pis }),
            ..Default::default()
        },
        Pis::Outr => PISProcess {
            pis_outr: Some(PISOutr {
                cst: "99".to_string(),
                qbc_prod: Some("0.00".to_string()),
                valiq_prod: Some("0.00".to_string()),
                vpis: Some("0.00".to_string()),
            }),
            ..Default::default()
        },
        Pis::Nt { cst } => PISProcess {
            pis_nt: Some(PISNT { cst: cst.clone() }),
            ..Default::default()
        },
        Pis::Qtde { cst, q_bc_prod, v_aliq_prod, v_pis } => PISProcess {
            pis_qtde: Some(PISQtde {
                cst: cst.clone(),
                qbc_prod: format!("{:.3}", q_bc_prod),
                valiq_prod: format!("{:.4}", v_aliq_prod),
                vpis: format!("{:.2}", v_pis),
            }),
            ..Default::default()
        },
        Pis::St { v_bc, p_pis, q_bc_prod, v_aliq_prod, v_pis } => PISProcess {
            pis_st: Some(PISST {
                v_bc: v_bc.map(|v| format!("{:.2}", v)),
                p_pis: p_pis.map(|v| format!("{:.4}", v)),
                qbc_prod: q_bc_prod.map(|v| format!("{:.3}", v)),
                valiq_prod: v_aliq_prod.map(|v| format!("{:.4}", v)),
                vpis: Some(format!("{:.2}", v_pis)),
            }),
            ..Default::default()
        },
    }
}

fn select_cofins_process(cofins: &Cofins) -> COFINSProcess {
    match cofins {
        Cofins::Aliq { cst, v_bc, p_cofins, v_cofins } => COFINSProcess {
            cofins_aliq: Some(COFINSAliq { cst: cst.clone(), v_bc: *v_bc, p_cofins: *p_cofins, v_cofins: *v_cofins }),
            ..Default::default()
        },
        Cofins::Outr { cst } => COFINSProcess {
            cofins_outr: Some(COFINSOutr { cst: cst.clone(), v_bc: Some(0.0), p_cofins: Some(0.0), v_cofins: Some(0.0) }),
            ..Default::default()
        },
        Cofins::Nt { cst } => COFINSProcess {
            cofins_nt: Some(COFINSNT { cst: cst.clone() }),
            ..Default::default()
        },
        Cofins::Qtde { cst, q_bc_prod, v_aliq_prod, v_cofins } => COFINSProcess {
            cofins_qtde: Some(COFINSQtde {
                cst: cst.clone(),
                qbc_prod: format!("{:.3}", q_bc_prod),
                valiq_prod: format!("{:.4}", v_aliq_prod),
                vcofins: format!("{:.2}", v_cofins),
            }),
            ..Default::default()
        },
        Cofins::St { v_bc, p_cofins, q_bc_prod, v_aliq_prod, v_cofins } => COFINSProcess {
            cofins_st: Some(COFINSST {
                v_bc: v_bc.map(|v| format!("{:.2}", v)),
                p_cofins: p_cofins.map(|v| format!("{:.4}", v)),
                qbc_prod: q_bc_prod.map(|v| format!("{:.3}", v)),
                valiq_prod: v_aliq_prod.map(|v| format!("{:.4}", v)),
                vcofins: Some(format!("{:.2}", v_cofins)),
            }),
            ..Default::default()
        },
    }
}

fn select_ipi_process(ipi: &Ipi) -> IpiProcess {
    use quick_xml::se::to_string;
    let cst_num: u8 = ipi.cst.trim().parse().unwrap_or(99);
    // CST de saída tributada: 50, 99 (outros tributados)
    let tributado = matches!(cst_num, 50 | 99);
    let inner = if tributado {
        let trib = IPITrib {
            cst: ipi.cst.clone(),
            v_bc: ipi.v_bc,
            p_ipi: ipi.p_ipi,
            q_bc_prod: ipi.q_bc_prod.map(|v| format!("{:.3}", v)),
            v_aliq_prod: ipi.v_aliq_prod.map(|v| format!("{:.4}", v)),
            v_ipi: ipi.v_ipi.unwrap_or(0.0),
        };
        let xml = to_string(&trib).unwrap_or_default();
        format!("<IPITrib>{}</IPITrib>", &xml[xml.find('>').map(|i| i + 1).unwrap_or(0)..xml.rfind('<').unwrap_or(xml.len())])
    } else {
        let nt = IPINT { cst: ipi.cst.clone() };
        to_string(&nt).unwrap_or_default()
    };
    IpiProcess {
        c_enq: ipi.c_enq.clone(),
        c_selo: ipi.c_selo.clone(),
        q_selo: ipi.q_selo,
        tributado,
        inner,
    }
}
