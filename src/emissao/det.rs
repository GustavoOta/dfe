use super::det_process::entity::*;
use crate::arredondamento::{arred_decimal, fmt_dec, fmt_dec_ate, fmt_decimal, para_decimal};
use crate::tipos::{Cofins, Det, IbsCbs, Icms, Ipi, Pis};
use crate::error::Result;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

pub fn det_process(
    prod: Vec<Det>,
    _mod_: u32,
    tp_amb: u8,
    desconto_rateio: Option<Decimal>,
    frete_rateio: Option<Decimal>,
    outro_rateio: Option<Decimal>,
    _active_ibscbs: Option<String>,
) -> Result<Vec<DetProcess>> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    let mut first_item = 0;

    // desconto por rateio nos itens *********************************************************
    let desconto_rateado = desconto_rateio.unwrap_or_else(|| Decimal::new(0, 2));

    let mut total_produtos = Decimal::new(0, 2);
    for d in &prod {
        // soma o valor dos produtos para calcular o percentual do desconto
        total_produtos += para_decimal(d.v_prod);
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
        let v_prod_decimal = para_decimal(d.v_prod);
        let desconto_item = arred_decimal(v_prod_decimal * desconto_percentual, 2);
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
        let v_prod_decimal = para_decimal(d.v_prod);
        fretes_itens.push(arred_decimal(v_prod_decimal * frete_percentual, 2));
    }
    let soma_fretes: Decimal = fretes_itens.iter().cloned().sum();
    if soma_fretes != frete_rateado {
        if let Some(last) = fretes_itens.last_mut() {
            *last += frete_rateado - soma_fretes;
        }
    }
    // fim do frete por rateio nos itens *****************************************************

    // acréscimo por rateio nos itens (outras despesas acessórias → det/prod/vOutro) *********
    // Mesmo molde do frete: SEFAZ exige ICMSTot/vOutro == Σ det/prod/vOutro. Rateado
    // proporcional ao vProd, último item absorve a diferença de arredondamento.
    let outro_rateado = outro_rateio.unwrap_or_else(|| Decimal::new(0, 2));
    let outro_percentual = if total_produtos > Decimal::new(0, 2) {
        outro_rateado / total_produtos
    } else {
        Decimal::new(0, 2)
    };
    let mut outros_itens: Vec<Decimal> = Vec::new();
    for d in &prod {
        let v_prod_decimal = para_decimal(d.v_prod);
        outros_itens.push(arred_decimal(v_prod_decimal * outro_percentual, 2));
    }
    let soma_outros: Decimal = outros_itens.iter().cloned().sum();
    if soma_outros != outro_rateado {
        if let Some(last) = outros_itens.last_mut() {
            *last += outro_rateado - soma_outros;
        }
    }
    // fim do acréscimo por rateio nos itens *************************************************

    for (((d, desconto_item), frete_item), outro_item) in prod
        .iter()
        .zip(descontos_itens.iter())
        .zip(fretes_itens.iter())
        .zip(outros_itens.iter())
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

        // acréscimo rateado do item (None quando não há acréscimo → não emite <vOutro>)
        let v_outro_value: Option<Decimal> = if *outro_item > Decimal::new(0, 2) {
            Some(*outro_item)
        } else {
            None
        };

        // Base do ICMS da operação própria: vProd + frete + acréscimo − desconto, todos
        // já rateados. Só as tributações por parâmetros (`Icms::Parametros`) a usam.
        let base_icms = d.v_prod
            + frete_item.to_f64().unwrap_or(0.0)
            + outro_item.to_f64().unwrap_or(0.0)
            - desconto_item.to_f64().unwrap_or(0.0);

        det_process_values.push(DetProcess {
            prod: ProdProcess {
                c_prod: d.c_prod.to_string(),
                c_ean: d.c_ean.to_string(),
                x_prod: x_prod.clone(),
                ncm: d.ncm.to_string(),
                cfop: d.cfop.to_string(),
                cest: d.cest.clone(),
                c_benef: d.c_benef.as_deref().map(str::trim).filter(|v| !v.is_empty()).map(str::to_string),
                g_cred: g_cred_process(&d.cred_presumido, base_icms)?,
                u_com: d.u_com.to_string(),
                q_com: fmt_dec_ate(d.q_com, 3, 4),
                v_un_com: fmt_dec_ate(d.v_un_com, 2, 10),
                v_prod: fmt_dec(d.v_prod, 2),
                c_ean_trib: d.c_ean_trib.to_string(),
                u_trib: d.u_trib.to_string(),
                q_trib: fmt_dec_ate(d.q_trib, 3, 4),
                v_un_trib: fmt_dec_ate(d.v_un_trib, 2, 10),
                v_frete: v_frete_value,
                v_desc: v_desc_value,
                v_outro: v_outro_value,
                ind_tot: d.ind_tot.to_string(),
                x_ped: d.x_ped.clone(),
                n_item_ped: d.n_item_ped.clone(),
                comb: comb_process(d)?,
            },
            imposto: {
                let icms = select_icms_process(&d.icms, base_icms, d.q_trib)?;
                let pis = select_pis_process(&d.pis);
                let cofins = select_cofins_process(&d.cofins);
                let ibs_cbs = match d.ibs_cbs.as_ref() {
                    Some(ib) if ib.calcular => {
                        let v_bc = base_ibs_cbs(base_icms, &icms, &pis, &cofins);
                        ibs_cbs_process(Some(&ibs_cbs_calculado(ib, v_bc)))
                    }
                    outro => ibs_cbs_process(outro),
                };
                ImpostoProcess {
                v_tot_trib: fmt_dec(d.v_tot_trib, 2),
                icms,
                ipi: d.ipi.as_ref().map(select_ipi_process),
                pis,
                cofins,
                ibs_cbs,
                }
            },
            inf_ad_prod: d.inf_ad_prod.clone(),
        });
    }
    Ok(det_process_values)
}

/// Base do IBS/CBS do item (NT 2025.002-RTC, gIBSCBS/vBC): a base da operação (vProd + frete +
/// acréscimo − desconto, já rateados) menos os tributos que serão extintos, como impressos no
/// item. vSeg, vII, vServ, vICMSUFDest, vFCPUFDest, vICMSMono, vISSQN e vIS não existem nas
/// notas que a crate monta hoje: entram como zero.
fn base_ibs_cbs(base_operacao: f64, icms: &ICMSProcess, pis: &PISProcess, cofins: &COFINSProcess) -> f64 {
    use super::total::{cofins_v_cofins, icms_v_fcp, icms_v_icms, pis_v_pis};
    let v = crate::arredondamento::arred2(base_operacao)
        - crate::arredondamento::arred2(pis_v_pis(pis))
        - crate::arredondamento::arred2(cofins_v_cofins(cofins))
        - crate::arredondamento::arred2(icms_v_icms(icms))
        - crate::arredondamento::arred2(icms_v_fcp(icms));
    crate::arredondamento::arred2(v.max(0.0))
}

/// Valores do grupo a partir da base e das alíquotas (arredondamento único da crate).
fn ibs_cbs_calculado(ib: &IbsCbs, v_bc: f64) -> IbsCbs {
    use rust_decimal::prelude::FromPrimitive;
    let dec = |v: f64| Decimal::from_f64(crate::arredondamento::arred2(v)).unwrap_or(Decimal::ZERO);
    let pct = |p: Decimal| p.to_f64().unwrap_or(0.0);
    let (red_ibs, red_cbs) = reducoes(ib);
    IbsCbs {
        cst: ib.cst.clone(),
        class_trib: ib.class_trib.clone(),
        v_bc: dec(v_bc),
        p_ibs_uf: ib.p_ibs_uf,
        v_ibs_uf: dec(v_bc * pct(aliq_efetiva(ib.p_ibs_uf, red_ibs)) / 100.0),
        p_ibs_mun: ib.p_ibs_mun,
        v_ibs_mun: dec(v_bc * pct(aliq_efetiva(ib.p_ibs_mun, red_ibs)) / 100.0),
        p_cbs: ib.p_cbs,
        v_cbs: dec(v_bc * pct(aliq_efetiva(ib.p_cbs, red_cbs)) / 100.0),
        calcular: false,
        p_red_ibs: ib.p_red_ibs,
        p_red_cbs: ib.p_red_cbs,
    }
}

/// Reduções que vão ao XML: só para CST com `ind_gRed = 1` — em outro CST o `gRed` é a
/// rejeição 1032, então a redução recebida é ignorada.
fn reducoes(ib: &IbsCbs) -> (Option<Decimal>, Option<Decimal>) {
    if IbsCbs::cst_exige_reducao(&ib.cst) { (ib.p_red_ibs, ib.p_red_cbs) } else { (None, None) }
}

/// `pAliqEfet = p × (1 − pRedAliq/100)`, 4 casas (TDec_0302_04). Sem redução, a própria alíquota.
fn aliq_efetiva(p: Decimal, p_red: Option<Decimal>) -> Decimal {
    match p_red {
        Some(r) => (p * (Decimal::ONE_HUNDRED - r) / Decimal::ONE_HUNDRED)
            .round_dp_with_strategy(4, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
            .max(Decimal::ZERO),
        None => p,
    }
}

/// Grupo `gRed` (TRed: pRedAliq, pAliqEfet) da alíquota `p`.
fn g_red(p: Decimal, p_red: Option<Decimal>) -> Option<GRed> {
    p_red.map(|r| GRed {
        p_red_aliq: fmt_decimal(r, 4),
        p_aliq_efet: fmt_decimal(aliq_efetiva(p, Some(r)), 4),
    })
}

fn ibs_cbs_process(ibs_cbs: Option<&IbsCbs>) -> Option<IBSCBSProcess> {
    let ibs = ibs_cbs?;
    let (red_ibs, red_cbs) = reducoes(ibs);
    Some(IBSCBSProcess {
        cst: ibs.cst.clone(),
        c_class_trib: ibs.class_trib.clone(),
        g_ibscbs: GIBSCBS {
            v_bc: fmt_decimal(ibs.v_bc, 2),
            g_ibs_uf: GIBSUF {
                p_ibs_uf: fmt_decimal(ibs.p_ibs_uf, 4),
                g_red: g_red(ibs.p_ibs_uf, red_ibs),
                v_ibs_uf: fmt_decimal(ibs.v_ibs_uf, 2),
                ..Default::default()
            },
            g_ibs_mun: GIBSMun {
                p_ibs_mun: fmt_decimal(ibs.p_ibs_mun, 4),
                g_red: g_red(ibs.p_ibs_mun, red_ibs),
                v_ibs_mun: fmt_decimal(ibs.v_ibs_mun, 2),
                ..Default::default()
            },
            v_ibs: fmt_decimal(ibs.v_ibs_uf + ibs.v_ibs_mun, 2),
            g_cbs: GCBS {
                p_cbs: fmt_decimal(ibs.p_cbs, 4),
                g_red: g_red(ibs.p_cbs, red_cbs),
                v_cbs: fmt_decimal(ibs.v_cbs, 2),
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
/// gCred do item: vCredPresumido = base do item × %. Máximo 4 (XSD).
fn g_cred_process(creds: &[crate::tipos::CredPresumido], base: f64) -> Result<Vec<GCredProcess>> {
    if creds.len() > 4 {
        return Err(crate::error::DfeError::Validacao(
            "Crédito presumido: no máximo 4 por item (gCred).".into(),
        ));
    }
    Ok(creds
        .iter()
        .filter(|c| !c.codigo.trim().is_empty())
        .map(|c| GCredProcess {
            c_cred_presumido: c.codigo.trim().to_uppercase(),
            p_cred_presumido: fmt_dec(c.p, 4),
            v_cred_presumido: fmt_dec(crate::arredondamento::arred2(base * c.p / 100.0), 2),
        })
        .collect())
}

/// Grupo comb do item. Obrigatório nos CST monofásicos (02, 15, 53, 61): sem ele a SEFAZ
/// recusa — melhor falhar na montagem com mensagem de cadastro.
fn comb_process(d: &crate::tipos::Det) -> Result<Option<CombProcess>> {
    let mono = matches!(&d.icms, crate::tipos::Icms::Parametros(p)
        if matches!(p.cst.trim().trim_start_matches("ICMS"), "02" | "15" | "53" | "61"));
    let Some(c) = d.comb.as_ref().filter(|c| !c.c_prod_anp.trim().is_empty()) else {
        if mono {
            return Err(crate::error::DfeError::Validacao(format!(
                "Produto {}: CST monofásico de combustível exige o código ANP (grupo comb).",
                d.c_prod
            )));
        }
        return Ok(None);
    };
    let f = |v: Option<f64>, casas: u32| v.map(|x| fmt_dec(x, casas));
    Ok(Some(CombProcess {
        c_prod_anp: c.c_prod_anp.trim().to_string(),
        desc_anp: c.desc_anp.trim().to_string(),
        p_glp: f(c.p_glp, 4),
        p_gnn: f(c.p_gnn, 4),
        p_gni: f(c.p_gni, 4),
        v_part: f(c.v_part, 2),
        codif: c.codif.clone().filter(|s| !s.trim().is_empty()),
        q_temp: f(c.q_temp, 4),
        uf_cons: c.uf_cons.trim().to_uppercase(),
        cide: c.cide.map(|(q, a)| CideProcess {
            q_bc_prod: fmt_dec(q, 4),
            v_aliq_prod: fmt_dec(a, 4),
            v_cide: fmt_dec(crate::arredondamento::arred2(q * a), 2),
        }),
        encerrante: c.encerrante.as_ref().map(|e| EncerranteProcess {
            n_bico: e.n_bico.clone(),
            n_bomba: e.n_bomba.clone(),
            n_tanque: e.n_tanque.clone(),
            v_enc_ini: fmt_dec(e.v_enc_ini, 3),
            v_enc_fin: fmt_dec(e.v_enc_fin, 3),
        }),
        p_bio: f(c.p_bio.filter(|v| *v > 0.0), 4),
        orig_comb: c.orig_comb.iter().map(|o| OrigCombProcess {
            ind_import: o.ind_import.to_string(),
            c_uf_orig: o.c_uf_orig.to_string(),
            p_orig: fmt_dec(o.p_orig, 4),
        }).collect(),
    }))
}

pub(super) fn grupo_st_retido(
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
        Some(fmt_dec(bcst.unwrap_or(0.0), 2)),
        Some(fmt_dec(pst.unwrap_or(0.0), 2)),
        subst.map(|v| fmt_dec(v, 2)),
        Some(fmt_dec(icmsst.unwrap_or(0.0), 2)),
    )
}

fn select_icms_process(icms: &Icms, base_icms: f64, q_trib: f64) -> Result<ICMSProcess> {
    Ok(match icms {
        Icms::Parametros(p) => super::icms_calc::icms_de_parametros(p, base_icms, q_trib)?,

        Icms::Icms00 { orig, mod_bc, v_bc, p_icms, v_icms } =>
            ICMSProcess::ICMS00(ICMS00 {
                orig: *orig, cst: "00".to_string(), mod_bc: *mod_bc,
                v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
                ..Default::default()
            }),

        Icms::Icms10 { orig, mod_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst } =>
            ICMSProcess::ICMS10(ICMS10 {
                orig: *orig, cst: "10".to_string(), mod_bc: *mod_bc,
                v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
                mod_bcst: *mod_bcst, p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
                ..Default::default()
            }),

        Icms::Icms20 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS20(ICMS20 {
                orig: *orig, cst: "20".to_string(), mod_bc: *mod_bc,
                p_red_bc: *p_red_bc, v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
                ..Default::default()
            }),

        Icms::Icms30 { orig, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS30(ICMS30 {
                orig: *orig, cst: "30".to_string(), mod_bcst: *mod_bcst,
                p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
                ..Default::default()
            }),

        Icms::Icms40 { orig, cst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS40(ICMS40 {
                orig: *orig, cst: *cst, vicmsdeson: *v_icms_deson, mot_des_icms: *mot_des_icms,
                ind_deduz_deson: None,
            }),

        Icms::Icms51 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms_op, p_dif, v_icms_dif, v_icms } =>
            ICMSProcess::ICMS51(ICMS51 {
                orig: *orig, cst: "51".to_string(), mod_bc: *mod_bc,
                p_red_bc: *p_red_bc, v_bc: *v_bc, p_icms: *p_icms,
                v_icms_op: *v_icms_op, p_dif: *p_dif, v_icms_dif: *v_icms_dif,
                v_icms: *v_icms,
                ..Default::default()
            }),

        Icms::Icms60 { orig, v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret } => {
            let (v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret) =
                grupo_st_retido(*v_bcst_ret, *p_st, *v_icms_substituto, *v_icmsst_ret);
            ICMSProcess::ICMS60(ICMS60 {
                orig: *orig, cst: "60".to_string(),
                v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret,
                ..Default::default()
            })
        }

        Icms::Icms70 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS70(ICMS70 {
                orig: *orig, cst: "70".to_string(), mod_bc: *mod_bc,
                p_red_bc: *p_red_bc, v_bc: *v_bc, p_icms: *p_icms, v_icms: *v_icms,
                mod_bcst: *mod_bcst, p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
                ..Default::default()
            }),

        Icms::Icms90 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst, v_icms_deson, mot_des_icms } =>
            ICMSProcess::ICMS90(ICMS90 {
                orig: *orig, cst: "90".to_string(),
                mod_bc: *mod_bc, p_red_bc: *p_red_bc, v_bc: *v_bc,
                p_icms: *p_icms, v_icms: *v_icms,
                mod_bcst: *mod_bcst, p_mvast: *p_mvast, p_red_bcst: *p_red_bcst,
                v_bcst: *v_bcst, p_icmsst: *p_icmsst, v_icmsst: *v_icmsst,
                v_icms_deson: *v_icms_deson, mot_des_icms: *mot_des_icms,
                ..Default::default()
            }),

        Icms::Sn101 { orig, p_cred_sn, v_cred_icmssn } =>
            ICMSProcess::ICMSSN101(ICMSSN101 {
                orig: *orig, csosn: "101".to_string(),
                p_cred_sn: fmt_dec(p_cred_sn, 2),
                v_cred_icmssn: fmt_dec(v_cred_icmssn, 2),
            }),

        Icms::Sn102 { orig, csosn } =>
            ICMSProcess::ICMSSN102(ICMSSN102 { orig: *orig, csosn: csosn.clone() }),

        Icms::Sn500 { orig, v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret } => {
            let (vbcst_ret, p_st, v_icms_substituto, vicmsst_ret) =
                grupo_st_retido(*v_bcst_ret, *p_st, *v_icms_substituto, *v_icmsst_ret);
            ICMSProcess::ICMSSN500(ICMSSN500 {
                orig: *orig, csosn: "500".to_string(),
                vbcst_ret, p_st, v_icms_substituto, vicmsst_ret,
                ..Default::default()
            })
        }

        Icms::Sn900 { orig, mod_bc, v_bc, p_red_bc, p_icms, v_icms, p_cred_sn, v_cred_icmssn,
                      mod_bcst, p_mvast, p_red_bcst, v_bcst, p_icmsst, v_icmsst } =>
            ICMSProcess::ICMSSN900(ICMSSN900 {
                orig: *orig, csosn: "900".to_string(),
                modbc: mod_bc.map(|v| v.to_string()),
                vbc: v_bc.map(|v| fmt_dec(v, 2)),
                pred_bc: p_red_bc.map(|v| fmt_dec(v, 4)),
                picms: p_icms.map(|v| fmt_dec(v, 4)),
                vicms: v_icms.map(|v| fmt_dec(v, 2)),
                pcred_sn: p_cred_sn.map(|v| fmt_dec(v, 4)),
                vcred_icmssn: v_cred_icmssn.map(|v| fmt_dec(v, 2)),
                modbcst: mod_bcst.map(|v| v.to_string()),
                pmvast: p_mvast.map(|v| fmt_dec(v, 4)),
                pred_bcst: p_red_bcst.map(|v| fmt_dec(v, 4)),
                vbcst: v_bcst.map(|v| fmt_dec(v, 2)),
                picmsst: p_icmsst.map(|v| fmt_dec(v, 4)),
                vicmsst: v_icmsst.map(|v| fmt_dec(v, 2)),
                ..Default::default()
            }),
    })
}

/// PIS sem valor: 04–09 no `PISNT`, o resto (49–99) no `PISOutr` zerado com o próprio CST.
fn pis_sem_valor(cst: &str) -> PISProcess {
    let cst = cst.trim().to_string();
    if crate::tipos::CST_PIS_COFINS_NT.contains(&cst.as_str()) {
        PISProcess { pis_nt: Some(PISNT { cst }), ..Default::default() }
    } else {
        PISProcess {
            pis_outr: Some(PISOutr {
                cst,
                v_bc: None,
                p_pis: None,
                qbc_prod: Some("0.00".to_string()),
                valiq_prod: Some("0.00".to_string()),
                vpis: Some("0.00".to_string()),
            }),
            ..Default::default()
        }
    }
}

/// COFINS sem valor — mesma regra do [`pis_sem_valor`].
fn cofins_sem_valor(cst: &str) -> COFINSProcess {
    let cst = cst.trim().to_string();
    if crate::tipos::CST_PIS_COFINS_NT.contains(&cst.as_str()) {
        COFINSProcess { cofins_nt: Some(COFINSNT { cst }), ..Default::default() }
    } else {
        COFINSProcess {
            cofins_outr: Some(COFINSOutr {
                cst,
                v_bc: Some("0.00".to_string()),
                p_cofins: Some("0.00".to_string()),
                qbc_prod: None,
                valiq_prod: None,
                v_cofins: Some("0.00".to_string()),
            }),
            ..Default::default()
        }
    }
}

fn select_pis_process(pis: &Pis) -> PISProcess {
    match pis {
        Pis::Aliq { cst, v_bc, p_pis, v_pis } => PISProcess {
            pis_aliq: Some(PISAliq { cst: cst.clone(), v_bc: *v_bc, p_pis: *p_pis, v_pis: *v_pis }),
            ..Default::default()
        },
        Pis::Outr => pis_sem_valor("99"),
        // O grupo sai pelo CST, não pela variante: 07 no PISOutr (ou 49 no PISNT) é
        // rejeição de schema (enumeration).
        Pis::Nt { cst } => pis_sem_valor(cst),
        Pis::Qtde { cst, q_bc_prod, v_aliq_prod, v_pis } => PISProcess {
            pis_qtde: Some(PISQtde {
                cst: cst.clone(),
                qbc_prod: fmt_dec(q_bc_prod, 4),
                valiq_prod: fmt_dec(v_aliq_prod, 4),
                vpis: fmt_dec(v_pis, 2),
            }),
            ..Default::default()
        },
        Pis::OutrAliq { cst, v_bc, p_pis, v_pis } => PISProcess {
            pis_outr: Some(PISOutr {
                cst: cst.clone(),
                v_bc: Some(fmt_dec(v_bc, 2)),
                p_pis: Some(fmt_dec(p_pis, 4)),
                qbc_prod: None,
                valiq_prod: None,
                vpis: Some(fmt_dec(v_pis, 2)),
            }),
            ..Default::default()
        },
        Pis::OutrQtde { cst, q_bc_prod, v_aliq_prod, v_pis } => PISProcess {
            pis_outr: Some(PISOutr {
                cst: cst.clone(),
                v_bc: None,
                p_pis: None,
                qbc_prod: Some(fmt_dec(q_bc_prod, 4)),
                valiq_prod: Some(fmt_dec(v_aliq_prod, 4)),
                vpis: Some(fmt_dec(v_pis, 2)),
            }),
            ..Default::default()
        },
        // CST 05 do substituto: `PISNT` CST 05 dentro do <PIS> e o `PISST` como grupo irmão
        // (o XSD não aceita o PISST dentro do <PIS>; ver `ImpostoProcess::to_xml`).
        Pis::St { v_bc, p_pis, q_bc_prod, v_aliq_prod, v_pis, ind_soma } => PISProcess {
            pis_nt: Some(PISNT { cst: "05".to_string() }),
            pis_st: Some(PISST {
                v_bc: v_bc.map(|v| fmt_dec(v, 2)),
                p_pis: p_pis.map(|v| fmt_dec(v, 4)),
                qbc_prod: q_bc_prod.map(|v| fmt_dec(v, 4)),
                valiq_prod: v_aliq_prod.map(|v| fmt_dec(v, 4)),
                vpis: Some(fmt_dec(v_pis, 2)),
                ind_soma: ind_soma.map(|i| i.to_string()),
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
        Cofins::Outr { cst } | Cofins::Nt { cst } => cofins_sem_valor(cst),
        Cofins::Qtde { cst, q_bc_prod, v_aliq_prod, v_cofins } => COFINSProcess {
            cofins_qtde: Some(COFINSQtde {
                cst: cst.clone(),
                qbc_prod: fmt_dec(q_bc_prod, 4),
                valiq_prod: fmt_dec(v_aliq_prod, 4),
                vcofins: fmt_dec(v_cofins, 2),
            }),
            ..Default::default()
        },
        Cofins::OutrAliq { cst, v_bc, p_cofins, v_cofins } => COFINSProcess {
            cofins_outr: Some(COFINSOutr {
                cst: cst.clone(),
                v_bc: Some(fmt_dec(v_bc, 2)),
                p_cofins: Some(fmt_dec(p_cofins, 4)),
                qbc_prod: None,
                valiq_prod: None,
                v_cofins: Some(fmt_dec(v_cofins, 2)),
            }),
            ..Default::default()
        },
        Cofins::OutrQtde { cst, q_bc_prod, v_aliq_prod, v_cofins } => COFINSProcess {
            cofins_outr: Some(COFINSOutr {
                cst: cst.clone(),
                v_bc: None,
                p_cofins: None,
                qbc_prod: Some(fmt_dec(q_bc_prod, 4)),
                valiq_prod: Some(fmt_dec(v_aliq_prod, 4)),
                v_cofins: Some(fmt_dec(v_cofins, 2)),
            }),
            ..Default::default()
        },
        Cofins::St { v_bc, p_cofins, q_bc_prod, v_aliq_prod, v_cofins, ind_soma } => COFINSProcess {
            cofins_nt: Some(COFINSNT { cst: "05".to_string() }),
            cofins_st: Some(COFINSST {
                v_bc: v_bc.map(|v| fmt_dec(v, 2)),
                p_cofins: p_cofins.map(|v| fmt_dec(v, 4)),
                qbc_prod: q_bc_prod.map(|v| fmt_dec(v, 4)),
                valiq_prod: v_aliq_prod.map(|v| fmt_dec(v, 4)),
                vcofins: Some(fmt_dec(v_cofins, 2)),
                ind_soma: ind_soma.map(|i| i.to_string()),
            }),
            ..Default::default()
        },
    }
}

#[cfg(test)]
mod tests_pis_cofins_grupo {
    use super::*;
    use crate::emissao::det_process::entity::ImpostoProcess;
    use crate::interno::validation::validar_fragmento;
    use crate::tipos::{Cofins, Pis};

    /// Todo CST que vai sem valor: 04–09 (NT) e 49–99 (Outr), como o XSD lista.
    const SEM_VALOR: &[&str] = &[
        "04", "05", "06", "07", "08", "09", "49", "50", "51", "52", "53", "54", "55", "56",
        "60", "61", "62", "63", "64", "65", "66", "67", "70", "71", "72", "73", "74", "75", "98", "99",
    ];

    fn fragmento(xml: &str, tag: &str) -> String {
        let ini = xml.find(&format!("<{tag}>")).unwrap();
        let fim = xml.find(&format!("</{tag}>")).unwrap() + tag.len() + 3;
        xml[ini..fim].to_string()
    }

    fn xml(pis: &Pis, cofins: &Cofins) -> String {
        ImpostoProcess {
            v_tot_trib: "0.00".into(),
            icms: ICMSProcess::ICMSSN102(ICMSSN102 { orig: 0, csosn: "102".into() }),
            ipi: None,
            pis: select_pis_process(pis),
            cofins: select_cofins_process(cofins),
            ibs_cbs: None,
        }
        .to_xml()
    }

    fn valida(xml: &str, cst: &str) {
        for tag in ["PIS", "COFINS"] {
            let f = fragmento(xml, tag);
            if let Err(e) = validar_fragmento(tag, &f) {
                panic!("{tag} CST {cst}: XSD recusou: {e}
{f}");
            }
            assert!(f.contains(&format!("<CST>{cst}</CST>")), "{tag} saiu com outro CST: {f}");
        }
    }

    #[test]
    fn nao_tributado_escolhe_o_grupo_pelo_cst_e_passa_no_xsd() {
        for cst in SEM_VALOR {
            let x = xml(&Pis::nao_tributado(cst), &Cofins::nao_tributado(cst));
            valida(&x, cst);
            let grupo = if cst.as_bytes()[0] == b'0' { "NT" } else { "Outr" };
            assert!(x.contains(&format!("<PIS{grupo}>")) && x.contains(&format!("<COFINS{grupo}>")), "{cst}: {x}");
        }
    }

    /// Regressão da rejeição de schema: os conversores mandavam 04–09 em `Outr` (COFINS 07 no
    /// COFINSOutr) e o `Pis::Outr` saía sempre com CST 99. A variante errada não quebra mais.
    #[test]
    fn variante_errada_ainda_sai_no_grupo_certo() {
        for cst in SEM_VALOR {
            valida(&xml(&Pis::Nt { cst: cst.to_string() }, &Cofins::Outr { cst: cst.to_string() }), cst);
            valida(&xml(&Pis::Nt { cst: cst.to_string() }, &Cofins::Nt { cst: cst.to_string() }), cst);
        }
        valida(&xml(&Pis::Outr, &Cofins::Outr { cst: "99".into() }), "99");
    }

    // ── Qualquer CST, qualquer modalidade (Pis/Cofins::por_parametros) ─────────────

    use crate::tipos::PisCofinsParametros;

    /// Os 32 CST de PIS/COFINS do leiaute.
    const TODOS: &[&str] = &[
        "01", "02", "03", "04", "05", "06", "07", "08", "09", "49", "50", "51", "52", "53", "54",
        "55", "56", "60", "61", "62", "63", "64", "65", "66", "67", "70", "71", "72", "73", "74",
        "75", "98", "99",
    ];

    fn par(cst: &str, aliquota: Option<f64>, v_aliq_prod: Option<f64>) -> PisCofinsParametros {
        PisCofinsParametros { cst: cst.into(), v_bc: 100.0, aliquota, q_bc_prod: 3.5, v_aliq_prod }
    }

    fn por(p: &PisCofinsParametros) -> String {
        xml(&Pis::por_parametros(p), &Cofins::por_parametros(p))
    }

    #[test]
    fn todo_cst_em_toda_modalidade_passa_no_xsd() {
        for cst in TODOS {
            for (aliq, reais) in [(None, None), (Some(1.65), None), (None, Some(0.1234))] {
                valida(&por(&par(cst, aliq, reais)), cst);
            }
        }
    }

    #[test]
    fn outras_49_a_99_levam_o_valor_por_aliquota_ou_por_quantidade() {
        for cst in &TODOS[9..] {
            let x = por(&par(cst, Some(1.65), None));
            assert!(x.contains(&format!("<PISOutr><CST>{cst}</CST><vBC>100.00</vBC><pPIS>1.6500</pPIS><vPIS>1.65</vPIS></PISOutr>")), "{x}");
            assert!(x.contains(&format!("<COFINSOutr><CST>{cst}</CST><vBC>100.00</vBC><pCOFINS>1.6500</pCOFINS><vCOFINS>1.65</vCOFINS></COFINSOutr>")), "{x}");
            // 3,5 × 0,1234 = 0,4319 → 0,43
            let x = por(&par(cst, None, Some(0.1234)));
            assert!(x.contains(&format!("<PISOutr><CST>{cst}</CST><qBCProd>3.5000</qBCProd><vAliqProd>0.1234</vAliqProd><vPIS>0.43</vPIS></PISOutr>")), "{x}");
            assert!(x.contains("<vCOFINS>0.43</vCOFINS>"), "{x}");
        }
    }

    #[test]
    fn nt_04_a_09_ignora_aliquota_e_01_02_03_seguem_os_grupos_proprios() {
        for cst in &TODOS[3..9] {
            let x = por(&par(cst, Some(1.65), Some(0.5)));
            assert!(x.contains(&format!("<PISNT><CST>{cst}</CST></PISNT>")), "{x}");
        }
        assert!(por(&par("01", Some(1.65), None)).contains("<PISAliq><CST>01</CST><vBC>100.00</vBC>"));
        assert!(por(&par("03", None, Some(0.1234))).contains("<PISQtde><CST>03</CST><qBCProd>3.5000</qBCProd>"));
    }

    #[test]
    fn st_do_substituto_sai_como_grupo_irmao_e_valido() {
        let pis = Pis::St { v_bc: Some(100.0), p_pis: Some(1.65), q_bc_prod: None, v_aliq_prod: None, v_pis: 1.65, ind_soma: Some(1) };
        let cofins = Cofins::St { v_bc: None, p_cofins: None, q_bc_prod: Some(3.5), v_aliq_prod: Some(0.1234), v_cofins: 0.43, ind_soma: Some(0) };
        let x = xml(&pis, &cofins);
        assert!(x.contains("<PIS><PISNT><CST>05</CST></PISNT></PIS><PISST><vBC>100.00</vBC><pPIS>1.6500</pPIS><vPIS>1.65</vPIS><indSomaPISST>1</indSomaPISST></PISST><COFINS>"), "{x}");
        assert!(x.contains("</COFINS><COFINSST><qBCProd>3.5000</qBCProd><vAliqProd>0.1234</vAliqProd><vCOFINS>0.43</vCOFINS><indSomaCOFINSST>0</indSomaCOFINSST></COFINSST>"), "{x}");
        for tag in ["PISST", "COFINSST"] {
            let f = fragmento(&x, tag);
            if let Err(e) = validar_fragmento(tag, &f) {
                panic!("{tag}: XSD recusou: {e}\n{f}");
            }
        }
    }
}

#[cfg(test)]
mod tests_pis_cofins_total {
    use super::super::total::total_process;
    use super::det_process;
    use crate::tipos::{Cofins, Det, Pis, PisCofinsParametros, Total};

    fn item(pis: Pis, cofins: Cofins) -> Det {
        Det { v_prod: 100.0, pis, cofins, ..Default::default() }
    }

    #[test]
    fn total_soma_o_outr_com_valor_no_vpis_e_no_vcofins() {
        let p = PisCofinsParametros { cst: "49".into(), v_bc: 100.0, aliquota: Some(1.65), ..Default::default() };
        let c = PisCofinsParametros { aliquota: Some(7.6), ..p.clone() };
        let dets = det_process(vec![item(Pis::por_parametros(&p), Cofins::por_parametros(&c))], 55, 1, None, None, None, None).unwrap();
        let tot = total_process(Total::default(), dets, 1, None).unwrap().icms_tot;
        assert_eq!((tot.v_pis.as_str(), tot.v_cofins.as_str()), ("1.65", "7.60"));
        assert_eq!(tot.v_nf, "100.00", "PIS/COFINS próprios não somam no vNF");
    }

    #[test]
    fn st_fica_fora_do_vpis_e_entra_no_vnf_so_com_ind_soma_1() {
        let st = |ind| item(
            Pis::St { v_bc: Some(100.0), p_pis: Some(1.65), q_bc_prod: None, v_aliq_prod: None, v_pis: 1.65, ind_soma: ind },
            Cofins::St { v_bc: Some(100.0), p_cofins: Some(7.6), q_bc_prod: None, v_aliq_prod: None, v_cofins: 7.6, ind_soma: ind },
        );
        let tot = |ind| {
            let dets = det_process(vec![st(ind)], 55, 1, None, None, None, None).unwrap();
            total_process(Total::default(), dets, 1, None).unwrap().icms_tot
        };
        let com = tot(Some(1));
        assert_eq!((com.v_pis.as_str(), com.v_cofins.as_str()), ("0.00", "0.00"));
        assert_eq!(com.v_nf, "109.25");
        assert_eq!(tot(Some(0)).v_nf, "100.00");
        assert_eq!(tot(None).v_nf, "100.00");
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

    // ── Arredondamento (crate::arredondamento) ────────────────────────────────
    // Item e total têm de usar o MESMO centavo: o total é a soma do que foi impresso nos
    // itens. Antes o item saía por format!("{:.2}") (binário, desempate para o par) e o
    // total somava o f64 cru do chamador — 0.125 + 0.125 imprimia 0.12 + 0.12 no item e
    // 0.25 no total (rejeição 532/602).

    fn item_tributado(v_prod: f64, v_icms: f64, v_pis: f64) -> Det {
        Det {
            v_prod,
            icms: crate::tipos::Icms::icms00(0, 3, v_prod, 12.5, v_icms),
            pis: crate::tipos::Pis::Aliq { cst: "01".into(), v_bc: v_prod, p_pis: 0.65, v_pis },
            cofins: crate::tipos::Cofins::Aliq { cst: "01".into(), v_bc: v_prod, p_cofins: 3.0, v_cofins: v_pis },
            ..Default::default()
        }
    }

    #[test]
    fn meio_centavo_do_item_sobe_e_o_total_soma_o_impresso() {
        let prod = vec![item_tributado(1.0, 0.125, 0.125), item_tributado(1.0, 0.125, 0.125)];
        let dets = det_process(prod, 55, 3, None, None, None, None).unwrap();

        let pis = dets[0].imposto.pis.pis_aliq.as_ref().unwrap();
        let xml = quick_xml::se::to_string(pis).unwrap();
        assert!(xml.contains("<vPIS>0.13</vPIS>"), "{xml}");

        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_icms, "0.26");
        assert_eq!(tot.icms_tot.v_pis, "0.26");
        assert_eq!(tot.icms_tot.v_cofins, "0.26");
    }

    // ── ICMS por parâmetros + cBenef (ICMS_CADASTRO_PLANO.md) ─────────────────

    fn item_param(v_prod: f64, p: crate::tipos::IcmsParametros) -> Det {
        Det { v_prod, icms: crate::tipos::Icms::Parametros(p), ..Default::default() }
    }

    #[test]
    fn cbenef_sai_entre_cest_e_cfop_e_vazio_nao_sai() {
        let det = Det { cest: Some("0300100".into()), c_benef: Some(" SP070001 ".into()), ..item(10.0) };
        let dets = det_process(vec![det], 65, 2, None, None, None, None).unwrap();
        let xml = quick_xml::se::to_string(&dets[0].prod).unwrap();
        assert!(xml.contains("<CEST>0300100</CEST><cBenef>SP070001</cBenef><CFOP>"), "{xml}");

        let det = Det { c_benef: Some("  ".into()), ..item(10.0) };
        let dets = det_process(vec![det], 65, 2, None, None, None, None).unwrap();
        assert!(!quick_xml::se::to_string(&dets[0].prod).unwrap().contains("cBenef"));
    }

    #[test]
    fn base_do_icms_por_parametros_desconta_o_rateio() {
        // 100 + 100, desconto de 20 rateado → base 90 em cada item
        let p = crate::tipos::IcmsParametros { cst: "00".into(), p_icms: Some(18.0), ..Default::default() };
        let prod = vec![item_param(100.0, p.clone()), item_param(100.0, p)];
        let dets = det_process(prod, 65, 1, Some(d("20.00")), None, None, None).unwrap();
        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_bc, "180.00");
        assert_eq!(tot.icms_tot.v_icms, "32.40");
    }

    #[test]
    fn total_soma_st_fcp_e_tira_desonerado_deduzido() {
        let st = crate::tipos::IcmsParametros {
            cst: "10".into(), p_icms: Some(18.0), p_fcp: Some(2.0), p_mvast: Some(40.0),
            p_icmsst: Some(18.0), p_fcpst: Some(2.0), ..Default::default()
        };
        let isento = crate::tipos::IcmsParametros {
            cst: "40".into(), p_icms: Some(18.0), mot_des_icms: Some(9), ind_deduz_deson: Some(1),
            ..Default::default()
        };
        let dets = det_process(vec![item_param(100.0, st), item_param(100.0, isento)], 55, 1, None, None, None, None).unwrap();
        let tot = total_process(Total::default(), dets, 1, None).unwrap().icms_tot;
        // ST: vBCST 140, vICMSST 25,20 − 18 = 7,20; FCP próprio 2,00; FCP-ST 2,80 − 2,00 = 0,80
        assert_eq!((tot.v_bc_st.as_str(), tot.v_st.as_str()), ("140.00", "7.20"));
        assert_eq!((tot.v_fcp.as_str(), tot.v_fcpst.as_str()), ("2.00", "0.80"));
        assert_eq!(tot.v_icms_deson, "18.00");
        // vNF = 200 + 7,20 + 0,80 − 18,00
        assert_eq!(tot.v_nf, "190.00");
    }

    #[test]
    fn erro_de_cadastro_no_icms_interrompe_a_montagem() {
        let p = crate::tipos::IcmsParametros { cst: "20".into(), p_icms: Some(18.0), ..Default::default() };
        assert!(det_process(vec![item_param(10.0, p)], 65, 1, None, None, None, None).is_err());
    }

    // ── IBS/CBS calculado pela crate (NT 2025.002-RTC, rejeição 1115) ──────────

    fn ibs(calcular: bool) -> crate::tipos::IbsCbs {
        let d = |s: &str| Decimal::from_str_exact(s).unwrap();
        crate::tipos::IbsCbs {
            cst: "000".into(), class_trib: "000001".into(),
            v_bc: d("999.00"), p_ibs_uf: d("0.10"), v_ibs_uf: d("9.99"), p_ibs_mun: d("0.00"),
            v_ibs_mun: d("0.00"), p_cbs: d("0.90"), v_cbs: d("9.99"), calcular,
            p_red_ibs: None, p_red_cbs: None,
        }
    }

    #[test]
    fn base_do_ibs_cbs_tira_icms_pis_cofins_e_desconto() {
        // 2 × 100, desconto 20 → base da operação 90/item; ICMS 18% sobre 90 = 16,20;
        // PIS 1,65% = 1,49; COFINS 7,6% = 6,84 → vBC = 90 − 16,20 − 1,49 − 6,84 = 65,47
        let item = || Det {
            v_prod: 100.0,
            icms: crate::tipos::Icms::Parametros(crate::tipos::IcmsParametros {
                cst: "00".into(), p_icms: Some(18.0), ..Default::default()
            }),
            pis: crate::tipos::Pis::Aliq { cst: "01".into(), v_bc: 90.0, p_pis: 1.65, v_pis: 1.49 },
            cofins: crate::tipos::Cofins::Aliq { cst: "01".into(), v_bc: 90.0, p_cofins: 7.6, v_cofins: 6.84 },
            ibs_cbs: Some(ibs(true)),
            ..Default::default()
        };
        let dets = det_process(vec![item(), item()], 65, 1, Some(d("20.00")), None, None, None).unwrap();
        let g = &dets[0].imposto.ibs_cbs.as_ref().unwrap().g_ibscbs;
        assert_eq!(g.v_bc, "65.47");
        // 65,47 × 0,10% = 0,065 → 0,07; × 0,90% = 0,589 → 0,59 (valores recebidos ignorados)
        assert_eq!(g.g_ibs_uf.v_ibs_uf, "0.07");
        assert_eq!(g.g_cbs.v_cbs, "0.59");
        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        let t = tot.ibs_cbs_tot.expect("IBSCBSTot com itens tributados");
        assert_eq!(t.v_bc_ibs_cbs, "130.94");
        assert_eq!(t.g_cbs.v_cbs, "1.18");
    }

    #[test]
    fn ibscbstot_manda_credito_presumido_zerado_e_nao_vazio() {
        // vCredPres/vCredPresCondSus são obrigatórios no IBSCBSTot (gIBS e gCBS); `<vCredPres/>`
        // vazio reprova no XSD (pattern TDec1302RTC) antes de chegar à SEFAZ.
        let det = Det { v_prod: 10.0, ibs_cbs: Some(ibs(true)), ..Default::default() };
        let dets = det_process(vec![det], 65, 1, None, None, None, None).unwrap();
        let xml = quick_xml::se::to_string(&total_process(Total::default(), dets, 1, None).unwrap()).unwrap();
        assert!(!xml.contains("<vCredPres/>") && !xml.contains("<vCredPresCondSus/>"), "{xml}");
        assert_eq!(xml.matches("<vCredPres>0.00</vCredPres>").count(), 2, "{xml}");
        assert_eq!(xml.matches("<vCredPresCondSus>0.00</vCredPresCondSus>").count(), 2, "{xml}");
    }

    fn ibs_reduzido(cst: &str, red: &str) -> crate::tipos::IbsCbs {
        let d = |s: &str| Decimal::from_str_exact(s).unwrap();
        crate::tipos::IbsCbs {
            cst: cst.into(), class_trib: "200014".into(),
            p_red_ibs: Some(d(red)), p_red_cbs: Some(d(red)),
            ..ibs(true)
        }
    }

    fn xml_do_item(ib: crate::tipos::IbsCbs, v_prod: f64) -> (Vec<crate::emissao::det_process::entity::DetProcess>, String) {
        let det = Det { v_prod, ibs_cbs: Some(ib), ..Default::default() };
        let dets = det_process(vec![det], 65, 3, None, None, None, None).unwrap();
        let xml = quick_xml::se::to_string(dets[0].imposto.ibs_cbs.as_ref().unwrap()).unwrap();
        // Ordem, obrigatoriedade e formato (TDec_0302_04) contra o XSD oficial (TTribNFe).
        if let Err(e) = crate::interno::validation::validar_fragmento("IBSCBS", &xml) {
            panic!("IBSCBS fora do XSD: {e}");
        }
        (dets, xml)
    }

    #[test]
    fn cst_200_com_reducao_total_manda_gred_e_zera_os_valores() {
        // Rejeição 1033/1074: CST 200 (ind_gRed = 1) exige gRed em gIBSUF, gIBSMun e gCBS.
        // 200014 (hortícolas, Anexo XV) = 100% → pAliqEfet 0 e valores zerados (Flexdocs/ACBr).
        let (dets, xml) = xml_do_item(ibs_reduzido("200", "100"), 50.0);
        assert!(xml.contains(
            "<gIBSUF><pIBSUF>0.1000</pIBSUF><gRed><pRedAliq>100.0000</pRedAliq><pAliqEfet>0.0000</pAliqEfet></gRed><vIBSUF>0.00</vIBSUF></gIBSUF>"
        ), "{xml}");
        assert!(xml.contains(
            "<gIBSMun><pIBSMun>0.0000</pIBSMun><gRed><pRedAliq>100.0000</pRedAliq><pAliqEfet>0.0000</pAliqEfet></gRed><vIBSMun>0.00</vIBSMun></gIBSMun>"
        ), "{xml}");
        assert!(xml.contains(
            "<gCBS><pCBS>0.9000</pCBS><gRed><pRedAliq>100.0000</pRedAliq><pAliqEfet>0.0000</pAliqEfet></gRed><vCBS>0.00</vCBS></gCBS>"
        ), "{xml}");
        let t = total_process(Total::default(), dets, 1, None).unwrap().ibs_cbs_tot.unwrap();
        assert_eq!((t.v_bc_ibs_cbs.as_str(), t.g_cbs.v_cbs.as_str()), ("50.00", "0.00"));
    }

    #[test]
    fn cst_200_com_reducao_parcial_calcula_pela_aliquota_efetiva() {
        // 60%: pAliqEfet IBS UF = 0,10 × 0,4 = 0,04; CBS = 0,90 × 0,4 = 0,36.
        // vBC 1000 → vIBSUF 0,40 e vCBS 3,60 (vBC × pAliqEfet, não × pIBSUF).
        let (_, xml) = xml_do_item(ibs_reduzido("200", "60"), 1000.0);
        assert!(xml.contains("<gRed><pRedAliq>60.0000</pRedAliq><pAliqEfet>0.0400</pAliqEfet></gRed><vIBSUF>0.40</vIBSUF>"), "{xml}");
        assert!(xml.contains("<gRed><pRedAliq>60.0000</pRedAliq><pAliqEfet>0.3600</pAliqEfet></gRed><vCBS>3.60</vCBS>"), "{xml}");
        assert!(xml.contains("<vIBS>0.40</vIBS>"), "{xml}");
    }

    #[test]
    fn xsd_do_ibscbs_reprova_gred_fora_de_ordem() {
        // Trava o próprio validador: gRed depois de vIBSUF não pode passar.
        let (_, xml) = xml_do_item(ibs_reduzido("200", "100"), 50.0);
        let fora = xml.replacen(
            "<gRed><pRedAliq>100.0000</pRedAliq><pAliqEfet>0.0000</pAliqEfet></gRed><vIBSUF>0.00</vIBSUF>",
            "<vIBSUF>0.00</vIBSUF><gRed><pRedAliq>100.0000</pRedAliq><pAliqEfet>0.0000</pAliqEfet></gRed>",
            1,
        );
        assert_ne!(fora, xml);
        assert!(crate::interno::validation::validar_fragmento("IBSCBS", &fora).is_err());
        let sem_casas = xml.replacen("<pAliqEfet>0.0000</pAliqEfet>", "<pAliqEfet>0.1</pAliqEfet>", 1);
        assert!(crate::interno::validation::validar_fragmento("IBSCBS", &sem_casas).is_err());
    }

    #[test]
    fn reducao_em_cst_sem_ind_gred_nao_vai_ao_xml() {
        // CST 000 com gRed é a rejeição 1032: a redução recebida é ignorada.
        let (_, xml) = xml_do_item(ibs_reduzido("000", "60"), 1000.0);
        assert!(!xml.contains("gRed"), "{xml}");
        assert!(xml.contains("<vIBSUF>1.00</vIBSUF>"), "{xml}");
    }

    #[test]
    fn ibs_cbs_com_valores_prontos_segue_como_antes() {
        let det = Det { v_prod: 10.0, ibs_cbs: Some(ibs(false)), ..Default::default() };
        let dets = det_process(vec![det], 65, 1, None, None, None, None).unwrap();
        assert_eq!(dets[0].imposto.ibs_cbs.as_ref().unwrap().g_ibscbs.v_bc, "999.00");
    }

    #[test]
    fn valor_unitario_nao_perde_casas() {
        // 3,5 × 602,6097 = 2109,13 — vUnCom "602.61" daria 2109,135 (rejeição 629 em
        // preços de 4 casas com quantidade maior).
        let det = Det { q_com: 3.5, v_un_com: 602.6097, q_trib: 3.5, v_un_trib: 602.6097, v_prod: 2109.13, ..Default::default() };
        let dets = det_process(vec![det], 55, 3, None, None, None, None).unwrap();
        assert_eq!(dets[0].prod.v_un_com, "602.6097");
        assert_eq!(dets[0].prod.v_un_trib, "602.6097");
        assert_eq!(dets[0].prod.v_prod, "2109.13");
    }

    #[test]
    fn pis_cofins_por_quantidade_serializam_e_somam_no_total() {
        // CST 03: qBCProd aceita 4 casas (TDec_1204v) — 3,5 kg de 0,1234 kg não pode virar 0.123.
        let item = |q: f64| Det {
            v_prod: 10.0,
            pis: crate::tipos::Pis::Qtde { cst: "03".into(), q_bc_prod: q, v_aliq_prod: 0.1234, v_pis: 0.43 },
            cofins: crate::tipos::Cofins::Qtde { cst: "03".into(), q_bc_prod: q, v_aliq_prod: 0.5678, v_cofins: 1.99 },
            ..Default::default()
        };
        let dets = det_process(vec![item(3.5), item(0.1235)], 55, 3, None, None, None, None).unwrap();

        let pis = dets[1].imposto.pis.pis_qtde.as_ref().unwrap();
        assert_eq!((pis.cst.as_str(), pis.qbc_prod.as_str(), pis.valiq_prod.as_str(), pis.vpis.as_str()),
                   ("03", "0.1235", "0.1234", "0.43"));
        let cofins = dets[0].imposto.cofins.cofins_qtde.as_ref().unwrap();
        assert_eq!(cofins.qbc_prod, "3.5000");

        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_pis, "0.86");
        assert_eq!(tot.icms_tot.v_cofins, "3.98");
    }

    #[test]
    fn rateio_de_desconto_desempata_para_cima() {
        // 0.25 + 0.25, desconto 0.25 → 0.125 por item: meio para cima dá 0.13 e o último
        // absorve (0.12). O round_dp bancário dava 0.12 + 0.13.
        let prod = vec![item(0.25), item(0.25)];
        let dets = det_process(prod, 55, 1, Some(d("0.25")), None, None, None).unwrap();
        assert_eq!(dets[0].prod.v_desc, Some(d("0.13")));
        assert_eq!(dets[1].prod.v_desc, Some(d("0.12")));
    }

    #[test]
    fn frete_rateado_soma_bate_com_total_e_icmstot() {
        // 100 + 200 + 300 = 600; frete 30 → 5 / 10 / 15 (proporcional ao vProd).
        let prod = vec![item(100.0), item(200.0), item(300.0)];
        let dets = det_process(prod, 65, 1, None, Some(d("30.00")), None, None).unwrap();

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
        let dets = det_process(prod, 65, 1, None, Some(d("10.00")), None, None).unwrap();

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
        let dets = det_process(prod, 65, 1, None, None, None, None).unwrap();
        assert!(dets.iter().all(|x| x.prod.v_frete.is_none()));

        let total = Total { v_frete: 15.0, ..Default::default() };
        let tot = total_process(total, dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_frete, "15.00");
    }

    // ── Acréscimo rateado (det/prod/vOutro) ─────────────────────────────────────
    // O PDV mandava `acrescimo_rateio` e o gravisServer descartava: o acréscimo combinado
    // com o cliente não aparecia na nota. Agora vira vOutro por item e soma no vNF.

    #[test]
    fn outro_rateado_soma_bate_com_total_e_entra_no_vnf() {
        // 100 + 200 + 300 = 600; acréscimo 30 → 5 / 10 / 15.
        let prod = vec![item(100.0), item(200.0), item(300.0)];
        let dets = det_process(prod, 55, 1, None, None, Some(d("30.00")), None).unwrap();

        assert_eq!(dets[0].prod.v_outro, Some(d("5.00")));
        assert_eq!(dets[1].prod.v_outro, Some(d("10.00")));
        assert_eq!(dets[2].prod.v_outro, Some(d("15.00")));

        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_outro, "30.00");
        assert_eq!(tot.icms_tot.v_nf, "630.00");
    }

    #[test]
    fn outro_rateado_ajusta_ultimo_item_no_arredondamento() {
        let prod = vec![item(10.0), item(10.0), item(10.0)];
        let dets = det_process(prod, 55, 1, None, None, Some(d("10.00")), None).unwrap();

        let soma: Decimal = dets.iter().map(|x| x.prod.v_outro.unwrap_or(Decimal::ZERO)).sum();
        assert_eq!(soma, d("10.00"), "a soma do acréscimo rateado deve bater exatamente");
        assert_eq!(dets[2].prod.v_outro, Some(d("3.34")));
    }

    #[test]
    fn desconto_e_acrescimo_juntos_fecham_o_vnf() {
        // 100 + 100 = 200; desconto 30, acréscimo 10 → vNF = 180.
        let prod = vec![item(100.0), item(100.0)];
        let dets = det_process(prod, 55, 1, Some(d("30.00")), None, Some(d("10.00")), None).unwrap();

        let tot = total_process(Total::default(), dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_desc, "30.00");
        assert_eq!(tot.icms_tot.v_outro, "10.00");
        assert_eq!(tot.icms_tot.v_nf, "180.00");
    }

    #[test]
    fn sem_outro_rateado_e_aditivo() {
        // Sem outro_rateio: nenhum <vOutro> por item; ICMSTot/vOutro usa o global informado.
        let prod = vec![item(100.0)];
        let dets = det_process(prod, 55, 1, None, None, None, None).unwrap();
        assert!(dets.iter().all(|x| x.prod.v_outro.is_none()));

        let total = Total { v_outro: 7.0, ..Default::default() };
        let tot = total_process(total, dets, 1, None).unwrap();
        assert_eq!(tot.icms_tot.v_outro, "7.00");
        assert_eq!(tot.icms_tot.v_nf, "107.00");
    }

    #[test]
    fn voutro_sai_depois_do_vdesc_e_antes_do_indtot() {
        // A ordem das tags de <prod> é normativa (xs:sequence do XSD).
        let prod = vec![item(100.0)];
        let dets = det_process(prod, 55, 1, Some(d("5.00")), None, Some(d("2.00")), None).unwrap();
        let xml = quick_xml::se::to_string(&dets[0].prod).unwrap();

        let pos = |t: &str| xml.find(t).unwrap_or_else(|| panic!("faltou {} em {}", t, xml));
        assert!(pos("<vDesc>") < pos("<vOutro>"));
        assert!(pos("<vOutro>") < pos("<indTot>"));
        assert!(xml.contains("<vOutro>2.00</vOutro>"));
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
        let ICMSProcess::ICMSSN500(sn500) = select_icms_process(&icms, 0.0, 0.0).unwrap() else {
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
        let ICMSProcess::ICMSSN500(sn500) = select_icms_process(&Icms::sn500(0), 0.0, 0.0).unwrap() else {
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
        let ICMSProcess::ICMS60(icms60) = select_icms_process(&icms, 0.0, 0.0).unwrap() else {
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
            q_bc_prod: ipi.q_bc_prod.map(|v| fmt_dec(v, 3)),
            v_aliq_prod: ipi.v_aliq_prod.map(|v| fmt_dec(v, 4)),
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

#[cfg(test)]
mod tests_icms_tudo {
    use super::super::total::total_process;
    use super::det_process;
    use crate::interno::validation::validar_fragmento;
    use crate::tipos::{Comb, CredPresumido, Det, Encerrante, Icms, IcmsParametros, OrigComb, Total};

    fn item(v_prod: f64, q: f64, p: IcmsParametros) -> Det {
        Det { v_prod, q_com: q, q_trib: q, icms: Icms::Parametros(p), ..Default::default() }
    }

    fn comb() -> Comb {
        Comb {
            c_prod_anp: "320102001".into(), desc_anp: "GASOLINA C COMUM".into(), uf_cons: "sp".into(),
            p_bio: Some(14.0), cide: Some((250.0, 0.1)),
            encerrante: Some(Encerrante { n_bico: "1".into(), n_bomba: Some("2".into()), n_tanque: "3".into(), v_enc_ini: 1000.0, v_enc_fin: 1250.0 }),
            orig_comb: vec![OrigComb { ind_import: 0, c_uf_orig: 35, p_orig: 100.0 }],
            ..Default::default()
        }
    }

    #[test]
    fn total_do_monofasico_e_vnf_com_retencao() {
        let p15 = IcmsParametros { cst: "15".into(), ad_rem_icms: Some(1.22), ad_rem_icms_reten: Some(0.5), ..Default::default() };
        let p61 = IcmsParametros { cst: "61".into(), ad_rem_icms_ret: Some(1.22), ..Default::default() };
        let d15 = Det { comb: Some(comb()), ..item(1500.0, 250.0, p15) };
        let d61 = Det { comb: Some(comb()), ..item(600.0, 100.0, p61) };
        let dets = det_process(vec![d15, d61], 65, 1, None, None, None, None).unwrap();
        let tot = total_process(Total::default(), dets, 1, None).unwrap().icms_tot;
        assert_eq!(tot.q_bc_mono.as_deref(), Some("250.0000"));
        assert_eq!(tot.v_icms_mono.as_deref(), Some("305.00"));
        assert_eq!(tot.v_icms_mono_reten.as_deref(), Some("125.00"));
        assert_eq!(tot.v_icms_mono_ret.as_deref(), Some("122.00"));
        // vNF = 1500 + 600 + vICMSMonoReten 125 (NT 2023.001, regra 610)
        assert_eq!(tot.v_nf, "2225.00");
    }

    #[test]
    fn sem_monofasico_o_total_nao_leva_os_campos() {
        let p = IcmsParametros { cst: "00".into(), p_icms: Some(18.0), ..Default::default() };
        let dets = det_process(vec![item(100.0, 1.0, p)], 65, 1, None, None, None, None).unwrap();
        let tot = total_process(Total::default(), dets, 1, None).unwrap().icms_tot;
        assert!(tot.q_bc_mono.is_none() && tot.v_icms_mono_reten.is_none());
    }

    #[test]
    fn total_soma_fcp_st_retido_e_part() {
        let p60 = IcmsParametros { cst: "60".into(), v_bcst_ret: Some(80.0), p_st: Some(18.0), v_icmsst_ret: Some(14.4), p_fcpst_ret: Some(2.0), ..Default::default() };
        let part = IcmsParametros {
            cst: "10".into(), p_icms: Some(12.0), p_mvast: Some(40.0), p_icmsst: Some(18.0),
            uf_st: Some("MG".into()), p_bc_op: Some(100.0), ..Default::default()
        };
        let dets = det_process(vec![item(100.0, 1.0, p60), item(100.0, 1.0, part)], 55, 1, None, None, None, None).unwrap();
        let tot = total_process(Total::default(), dets, 1, None).unwrap().icms_tot;
        assert_eq!(tot.v_fcpst_ret, "1.60");
        assert_eq!((tot.v_bc.as_str(), tot.v_icms.as_str()), ("100.00", "12.00"));
        // ST do Part: 140 × 18% − 12 = 13,20
        assert_eq!((tot.v_bc_st.as_str(), tot.v_st.as_str()), ("140.00", "13.20"));
    }

    #[test]
    fn prod_com_gcred_e_comb_passa_no_xsd() {
        let p = IcmsParametros { cst: "02".into(), ad_rem_icms: Some(1.22), ..Default::default() };
        let d = Det {
            c_prod: "1".into(), x_prod: "GASOLINA".into(), ncm: "27101259".into(), u_com: "L".into(),
            u_trib: "L".into(), v_un_com: 6.0, v_un_trib: 6.0, cfop: 5656,
            c_benef: Some("SP000001".into()),
            cred_presumido: vec![CredPresumido { codigo: "sp000002".into(), p: 3.0 }],
            comb: Some(comb()),
            ..item(1500.0, 250.0, p)
        };
        let dets = det_process(vec![d], 65, 1, None, None, None, None).unwrap();
        let xml = quick_xml::se::to_string(&dets[0].prod).unwrap();
        assert!(xml.contains("<cBenef>SP000001</cBenef><gCred><cCredPresumido>SP000002</cCredPresumido><pCredPresumido>3.0000</pCredPresumido><vCredPresumido>45.00</vCredPresumido></gCred><CFOP>"), "{xml}");
        assert!(xml.contains("<comb><cProdANP>320102001</cProdANP><descANP>GASOLINA C COMUM</descANP><UFCons>SP</UFCons><CIDE>"), "{xml}");
        if let Err(e) = validar_fragmento("prod", &xml) {
            panic!("XSD recusou o <prod>: {e}\n{xml}");
        }
    }

    #[test]
    fn monofasico_sem_codigo_anp_e_erro_de_cadastro() {
        let p = IcmsParametros { cst: "02".into(), ad_rem_icms: Some(1.22), ..Default::default() };
        let e = det_process(vec![item(100.0, 10.0, p)], 65, 1, None, None, None, None).unwrap_err();
        assert!(e.to_string().contains("ANP"), "{e}");
    }

    #[test]
    fn mais_de_4_creditos_presumidos_e_erro() {
        let c = CredPresumido { codigo: "SP000002".into(), p: 1.0 };
        let p = IcmsParametros { cst: "00".into(), p_icms: Some(18.0), ..Default::default() };
        let d = Det { cred_presumido: vec![c; 5], ..item(100.0, 1.0, p) };
        assert!(det_process(vec![d], 65, 1, None, None, None, None).is_err());
    }
}
