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
    frete_rateio: Option<Decimal>,
    _active_ibscbs: Option<String>,
) -> Result<Vec<DetProcess>> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    let mut first_item = 0;

    // desconto por rateio nos itens *********************************************************
    let desconto_rateado = desconto_rateio.unwrap_or_else(|| Decimal::new(0, 2));

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

    // frete por rateio nos itens (mesmo molde do desconto) **********************************
    // SEFAZ exige ICMSTot/vFrete == Σ det/prod/vFrete. Rateamos o frete total proporcional
    // ao vProd de cada item e ajustamos o último para a soma bater exatamente.
    let frete_rateado = frete_rateio.unwrap_or_else(|| Decimal::new(0, 2));
    let frete_percentual = if total_produtos > Decimal::new(0, 2) {
        frete_rateado / total_produtos
    } else {
        Decimal::new(0, 2)
    };
    let mut fretes_itens: Vec<Decimal> = Vec::new();
    for d in &prod {
        let v_prod_decimal = Decimal::from_f64(d.v_prod).unwrap_or(Decimal::new(0, 2));
        fretes_itens.push((v_prod_decimal * frete_percentual).round_dp(2));
    }
    let soma_fretes: Decimal = fretes_itens.iter().cloned().sum();
    if soma_fretes != frete_rateado {
        if let Some(last) = fretes_itens.last_mut() {
            *last += frete_rateado - soma_fretes;
        }
    }
    // fim do frete por rateio nos itens *****************************************************

    for ((d, desconto_item), frete_item) in
        prod.iter().zip(descontos_itens.iter()).zip(fretes_itens.iter())
    {
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

        // frete rateado do item (None quando não há frete → não emite <vFrete>)
        let v_frete_value: Option<Decimal> = if *frete_item > Decimal::new(0, 2) {
            Some(*frete_item)
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
                v_frete: v_frete_value,
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

/// Grupo do ICMS-ST retido anteriormente, compartilhado por CST 60 e CSOSN 500.
///
/// No XSD os quatro campos vivem dentro de um `xs:sequence minOccurs="0"` em que `vBCSTRet`,
/// `pST` e `vICMSSTRet` são obrigatórios e `vICMSSubstituto` é opcional — ou seja, é tudo ou
/// nada (NT 2011/004). Omitir o grupo em NF-e (modelo 55) gera a rejeição 938; em NFC-e
/// (modelo 65) a SEFAZ não cobra.
///
/// Zero é tratado como "não informado": `pST` é `TDec_0302a04Opc`, que por definição não aceita
/// valor zero, então um grupo zerado seria recusado de qualquer forma.
fn grupo_st_retido(
    v_bcst_ret: Option<f64>,
    p_st: Option<f64>,
    v_icms_substituto: Option<f64>,
    v_icmsst_ret: Option<f64>,
) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let bcst = v_bcst_ret.filter(|&v| v > 0.0);
    let pst = p_st.filter(|&v| v > 0.0);
    let subst = v_icms_substituto.filter(|&v| v > 0.0);
    let icmsst = v_icmsst_ret.filter(|&v| v > 0.0);

    if bcst.is_none() && pst.is_none() && icmsst.is_none() {
        return (None, None, None, None);
    }

    (
        Some(format!("{:.2}", bcst.unwrap_or(0.0))),
        Some(format!("{:.2}", pst.unwrap_or(0.0))),
        subst.map(|v| format!("{:.2}", v)),
        Some(format!("{:.2}", icmsst.unwrap_or(0.0))),
    )
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
            let (v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret) =
                grupo_st_retido(*v_bcst_ret, *p_st, *v_icms_substituto, *v_icmsst_ret);
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

        Icms::Sn500 { orig, v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret } => {
            let (vbcst_ret, p_st, v_icms_substituto, vicmsst_ret) =
                grupo_st_retido(*v_bcst_ret, *p_st, *v_icms_substituto, *v_icmsst_ret);
            ICMSProcess::ICMSSN500(ICMSSN500 {
                orig: *orig, csosn: "500".to_string(),
                vbcst_ret, p_st, v_icms_substituto, vicmsst_ret,
            })
        }

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

#[cfg(test)]
mod tests {
    use super::super::total::total_process;
    use super::det_process;
    use crate::tipos::{Det, Total};
    use rust_decimal::Decimal;

    fn item(v_prod: f64) -> Det {
        Det { v_prod, ..Default::default() }
    }

    fn d(s: &str) -> Decimal {
        Decimal::from_str_exact(s).unwrap()
    }

    #[test]
    fn frete_rateado_soma_bate_com_total_e_icmstot() {
        // 100 + 200 + 300 = 600; frete 30 → 5 / 10 / 15 (proporcional ao vProd).
        let prod = vec![item(100.0), item(200.0), item(300.0)];
        let dets = det_process(prod, 65, 1, None, Some(d("30.00")), None).unwrap();

        assert_eq!(dets[0].prod.v_frete, Some(d("5.00")));
        assert_eq!(dets[1].prod.v_frete, Some(d("10.00")));
        assert_eq!(dets[2].prod.v_frete, Some(d("15.00")));

        let soma: Decimal = dets.iter().map(|x| x.prod.v_frete.unwrap_or(Decimal::ZERO)).sum();
        assert_eq!(soma, d("30.00"));

        // ICMSTot/vFrete == Σ det/prod/vFrete (Total sem frete global).
        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_frete, "30.00");
    }

    #[test]
    fn frete_rateado_ajusta_ultimo_item_no_arredondamento() {
        // 10 + 10 + 10 = 30; frete 10 → 3.33 + 3.33 + 3.34 (último absorve a diferença) = 10.00.
        let prod = vec![item(10.0), item(10.0), item(10.0)];
        let dets = det_process(prod, 65, 1, None, Some(d("10.00")), None).unwrap();

        let soma: Decimal = dets.iter().map(|x| x.prod.v_frete.unwrap_or(Decimal::ZERO)).sum();
        assert_eq!(soma, d("10.00"), "a soma do frete rateado deve bater exatamente");
        assert_eq!(dets[2].prod.v_frete, Some(d("3.34")));

        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_frete, "10.00");
    }

    #[test]
    fn sem_frete_rateado_e_aditivo() {
        // Sem frete_rateio: nenhum <vFrete> por item; ICMSTot/vFrete usa o total global
        // informado (comportamento anterior preservado → mudança aditiva).
        let prod = vec![item(100.0), item(200.0)];
        let dets = det_process(prod, 65, 1, None, None, None).unwrap();
        assert!(dets.iter().all(|x| x.prod.v_frete.is_none()));

        let total = Total { v_frete: 15.0, ..Default::default() };
        let tot = total_process(total, dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_frete, "15.00");
    }

    // ── Grupo do ICMS-ST retido anteriormente (CST 60 / CSOSN 500) ──────────────
    // Rejeição 938 da SEFAZ: "Não informada vBCSTRet, pST, vICMSSubstituto e vICMSSTRet".
    // Vale para NF-e (modelo 55); NFC-e (modelo 65) não cobra o grupo.

    use super::{grupo_st_retido, select_icms_process};
    use crate::emissao::det_process::entity::ICMSProcess;
    use crate::tipos::Icms;

    #[test]
    fn grupo_st_retido_e_tudo_ou_nada() {
        // Nenhum valor → grupo inteiro omitido (comportamento válido só em NFC-e).
        assert_eq!(grupo_st_retido(None, None, None, None), (None, None, None, None));

        // Qualquer valor presente → os três obrigatórios saem, vICMSSubstituto continua opcional.
        let (bcst, pst, subst, icmsst) = grupo_st_retido(Some(100.0), Some(18.0), None, Some(18.0));
        assert_eq!(bcst.as_deref(), Some("100.00"));
        assert_eq!(pst.as_deref(), Some("18.00"));
        assert_eq!(subst, None, "vICMSSubstituto é opcional dentro do grupo");
        assert_eq!(icmsst.as_deref(), Some("18.00"));
    }

    #[test]
    fn grupo_st_retido_trata_zero_como_ausente() {
        // pST é TDec_0302a04Opc, que não aceita zero — um grupo zerado seria recusado.
        assert_eq!(
            grupo_st_retido(Some(0.0), Some(0.0), Some(0.0), Some(0.0)),
            (None, None, None, None)
        );
    }

    #[test]
    fn sn500_serializa_grupo_st_na_ordem_do_xsd() {
        let icms = Icms::Sn500 {
            orig: 0,
            v_bcst_ret: Some(231.0),
            p_st: Some(18.0),
            v_icms_substituto: None,
            v_icmsst_ret: Some(41.58),
        };
        let ICMSProcess::ICMSSN500(sn500) = select_icms_process(&icms) else {
            panic!("esperava ICMSSN500");
        };
        let xml = quick_xml::se::to_string(&sn500).unwrap();

        for tag in ["<orig>", "<CSOSN>", "<vBCSTRet>", "<pST>", "<vICMSSTRet>"] {
            assert!(xml.contains(tag), "faltou {} em {}", tag, xml);
        }
        assert!(!xml.contains("<vICMSSubstituto>"), "opcional não deve sair quando ausente");

        // O XSD define uma xs:sequence — a ordem das tags é normativa.
        let pos = |t: &str| xml.find(t).unwrap();
        assert!(pos("<vBCSTRet>") < pos("<pST>"));
        assert!(pos("<pST>") < pos("<vICMSSTRet>"));
    }

    #[test]
    fn sn500_sem_valores_omite_o_grupo_inteiro() {
        let ICMSProcess::ICMSSN500(sn500) = select_icms_process(&Icms::sn500(0)) else {
            panic!("esperava ICMSSN500");
        };
        let xml = quick_xml::se::to_string(&sn500).unwrap();

        assert!(xml.contains("<CSOSN>500</CSOSN>"));
        for tag in ["<vBCSTRet>", "<pST>", "<vICMSSTRet>"] {
            assert!(!xml.contains(tag), "{} não deveria sair em {}", tag, xml);
        }
    }

    #[test]
    fn icms60_mantem_o_mesmo_grupo_do_sn500() {
        let icms = Icms::Icms60 {
            orig: 0,
            v_bcst_ret: Some(231.0),
            p_st: Some(18.0),
            v_icms_substituto: Some(10.0),
            v_icmsst_ret: Some(41.58),
        };
        let ICMSProcess::ICMS60(icms60) = select_icms_process(&icms) else {
            panic!("esperava ICMS60");
        };
        let xml = quick_xml::se::to_string(&icms60).unwrap();

        assert!(xml.contains("<vBCSTRet>231.00</vBCSTRet>"));
        assert!(xml.contains("<pST>18.00</pST>"));
        assert!(xml.contains("<vICMSSubstituto>10.00</vICMSSubstituto>"));
        assert!(xml.contains("<vICMSSTRet>41.58</vICMSSTRet>"));
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
