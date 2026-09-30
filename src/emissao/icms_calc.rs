//! Monta o grupo ICMS do XSD a partir dos parâmetros do cadastro ([`IcmsParametros`]) e
//! calcula os valores sobre a base do item.
//!
//! Base = vProd + vFrete + vOutro − vDesc do item, já com o rateio feito pela crate. É a
//! base da operação própria (art. 13 da LC 87/96): frete e despesas entram, desconto
//! incondicional sai. `q` é a quantidade tributável (qTrib): pauta, preço tabelado e o
//! ad rem dos combustíveis são por unidade.
//!
//! Todas as modalidades do leiaute são calculadas:
//! - `modBC`: 3 valor da operação; 0 MVA própria (`p_mva_proprio`); 1 pauta e 2 preço
//!   tabelado máximo (`v_pauta` por unidade).
//! - `modBCST`: 4 MVA; 6 valor da operação; 0 preço tabelado/máximo sugerido, 1/2/3 listas
//!   negativa/positiva/neutra e 5 pauta (`v_pauta_st` por unidade).
//!
//! Grupos: ICMS00–90, ICMSPart (`uf_st`), ICMSST (repasse, `v_bcst_dest`), ICMS02/15/53/61
//! (monofásico de combustíveis, NT 2023.001) e os do Simples. Parâmetro que o grupo exige e
//! falta vira erro de montagem, não uma nota com valor inventado.

use super::det::grupo_st_retido;
use super::det_process::entity::*;
use crate::arredondamento::{arred, arred2, fmt_dec};
use crate::error::{DfeError, Result};
use crate::tipos::IcmsParametros;

/// Motivos de desoneração aceitos pelo XSD em cada grupo (leiaute NT 2026.004).
const MOT_DES_20: &[u16] = &[3, 9, 10, 11, 12];
const MOT_DES_30: &[u16] = &[6, 7, 9];
const MOT_DES_40: &[u16] = &[1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 16, 90];
const MOT_DES_70_90: &[u16] = &[3, 9, 12];
/// motDesICMSST (CST 10, 70, 90).
const MOT_DES_ST: &[u16] = &[3, 9, 12];
/// motDesICMS do ICMSPart.
const MOT_DES_PART: &[u16] = &[9, 10, 11];

pub(super) fn icms_de_parametros(p: &IcmsParametros, base: f64, q: f64) -> Result<ICMSProcess> {
    let codigo = normalizar_cst(&p.cst);
    let base = arred2(base);
    let q = arred(q, 4);
    let orig = p.orig;

    // Grupos especiais escolhidos por parâmetro: partilha e repasse de ST retido.
    if p.uf_st.as_deref().map(str::trim).is_some_and(|u| !u.is_empty())
        && matches!(codigo.as_str(), "10" | "20" | "90")
    {
        return icms_part(p, base, q, &codigo);
    }
    if p.v_bcst_dest.is_some() && matches!(codigo.as_str(), "41" | "60") {
        return icms_st_repasse(p, base, &codigo);
    }

    Ok(match codigo.as_str() {
        "00" => {
            let (mod_bc, v_bc, p_icms, v_icms) = icms_proprio(p, base, q, false, &codigo)?;
            let (_, p_fcp, v_fcp) = fcp(v_bc, p.p_fcp);
            ICMSProcess::ICMS00(ICMS00 { orig, cst: codigo, mod_bc, v_bc, p_icms, v_icms, p_fcp, v_fcp })
        }
        "10" => {
            let (mod_bc, v_bc, p_icms, v_icms) = icms_proprio(p, base, q, false, &codigo)?;
            let (v_bcfcp, p_fcp, v_fcp) = fcp(v_bc, p.p_fcp);
            let st = st(p, base, q, v_icms, v_fcp.unwrap_or(0.0), &codigo)?;
            let (v_icmsst_deson, mot_des_icms_st) = st_deson(p, &st, &codigo)?;
            ICMSProcess::ICMS10(ICMS10 {
                orig, cst: codigo, mod_bc, v_bc, p_icms, v_icms,
                v_bcfcp, p_fcp, v_fcp,
                mod_bcst: st.mod_bcst, p_mvast: st.p_mvast.unwrap_or(0.0), p_red_bcst: st.p_red_bcst,
                v_bcst: st.v_bcst, p_icmsst: st.p_icmsst, v_icmsst: st.v_icmsst,
                v_bcfcpst: st.v_bcfcpst, p_fcpst: st.p_fcpst, v_fcpst: st.v_fcpst,
                v_icmsst_deson, mot_des_icms_st,
            })
        }
        "20" => {
            let p_red_bc = exigir(p.p_red_bc, "% de redução da BC (pRedBC)", &codigo)?;
            let (mod_bc, v_bc, p_icms, v_icms) = icms_proprio(p, base, q, true, &codigo)?;
            let (v_bcfcp, p_fcp, v_fcp) = fcp(v_bc, p.p_fcp);
            let (v_icms_deson, mot_des_icms, ind_deduz_deson) =
                desoneracao(p, base, v_icms, MOT_DES_20, &codigo)?;
            ICMSProcess::ICMS20(ICMS20 {
                orig, cst: codigo, mod_bc, p_red_bc, v_bc, p_icms, v_icms,
                v_bcfcp, p_fcp, v_fcp, v_icms_deson, mot_des_icms, ind_deduz_deson,
            })
        }
        "30" => {
            // Isenta na operação própria: nada a deduzir do ICMS-ST.
            let st = st(p, base, q, 0.0, 0.0, &codigo)?;
            let (v_icms_deson, mot_des_icms, ind_deduz_deson) =
                desoneracao(p, base, 0.0, MOT_DES_30, &codigo)?;
            ICMSProcess::ICMS30(ICMS30 {
                orig, cst: codigo,
                mod_bcst: st.mod_bcst, p_mvast: st.p_mvast.unwrap_or(0.0), p_red_bcst: st.p_red_bcst,
                v_bcst: st.v_bcst, p_icmsst: st.p_icmsst, v_icmsst: st.v_icmsst,
                v_bcfcpst: st.v_bcfcpst, p_fcpst: st.p_fcpst, v_fcpst: st.v_fcpst,
                v_icms_deson, mot_des_icms, ind_deduz_deson,
            })
        }
        "40" | "41" | "50" => {
            let (vicmsdeson, mot_des_icms, ind_deduz_deson) =
                desoneracao(p, base, 0.0, MOT_DES_40, &codigo)?;
            ICMSProcess::ICMS40(ICMS40 {
                orig, cst: codigo.parse().unwrap_or(40), vicmsdeson, mot_des_icms, ind_deduz_deson,
            })
        }
        "51" => {
            // Todos os campos do grupo são opcionais no XSD: sem alíquota, é diferimento
            // sem destaque (só orig + CST).
            match p.p_icms {
                None => ICMSProcess::ICMS51(ICMS51 { orig, cst: codigo, ..Default::default() }),
                Some(_) => {
                    let (mod_bc, v_bc, p_icms, v_icms_op) = icms_proprio(p, base, q, true, &codigo)?;
                    let p_dif = p.p_dif.unwrap_or(0.0);
                    let v_icms_dif = arred2(v_icms_op * p_dif / 100.0);
                    let v_icms = arred2(v_icms_op - v_icms_dif);
                    let (v_bcfcp, p_fcp, v_fcp) = fcp(v_bc, p.p_fcp);
                    let (p_fcp_dif, v_fcp_dif, v_fcp_efet) = fcp_diferido(p, v_fcp);
                    let p_red_bc = p.p_red_bc.filter(|v| *v > 0.0);
                    ICMSProcess::ICMS51(ICMS51 {
                        orig, cst: codigo, mod_bc: Some(mod_bc),
                        p_red_bc, c_benef_rbc: c_benef_rbc(p, p_red_bc.is_some()),
                        v_bc: Some(v_bc), p_icms: Some(p_icms), v_icms_op: Some(v_icms_op),
                        p_dif: Some(p_dif), v_icms_dif: Some(v_icms_dif), v_icms: Some(v_icms),
                        v_bcfcp, p_fcp, v_fcp, p_fcp_dif, v_fcp_dif, v_fcp_efet,
                    })
                }
            }
        }
        "60" => {
            let (v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret) =
                grupo_st_retido(p.v_bcst_ret, p.p_st, p.v_icms_substituto, p.v_icmsst_ret);
            let r = retido_extra(p, base, v_bcst_ret.as_deref());
            ICMSProcess::ICMS60(ICMS60 {
                orig, cst: codigo, v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret,
                v_bcfcpst_ret: r.v_bcfcpst_ret, p_fcpst_ret: r.p_fcpst_ret, v_fcpst_ret: r.v_fcpst_ret,
                p_red_bc_efet: r.p_red_bc_efet, v_bc_efet: r.v_bc_efet,
                p_icms_efet: r.p_icms_efet, v_icms_efet: r.v_icms_efet,
            })
        }
        "70" => {
            let p_red_bc = exigir(p.p_red_bc, "% de redução da BC (pRedBC)", &codigo)?;
            let (mod_bc, v_bc, p_icms, v_icms) = icms_proprio(p, base, q, true, &codigo)?;
            let (v_bcfcp, p_fcp, v_fcp) = fcp(v_bc, p.p_fcp);
            let st = st(p, base, q, v_icms, v_fcp.unwrap_or(0.0), &codigo)?;
            let (v_icms_deson, mot_des_icms, ind_deduz_deson) =
                desoneracao(p, base, v_icms, MOT_DES_70_90, &codigo)?;
            let (v_icmsst_deson, mot_des_icms_st) = st_deson(p, &st, &codigo)?;
            ICMSProcess::ICMS70(ICMS70 {
                orig, cst: codigo, mod_bc, p_red_bc: Some(p_red_bc), v_bc, p_icms, v_icms,
                v_bcfcp, p_fcp, v_fcp,
                mod_bcst: st.mod_bcst, p_mvast: st.p_mvast.unwrap_or(0.0), p_red_bcst: st.p_red_bcst,
                v_bcst: st.v_bcst, p_icmsst: st.p_icmsst, v_icmsst: st.v_icmsst,
                v_bcfcpst: st.v_bcfcpst, p_fcpst: st.p_fcpst, v_fcpst: st.v_fcpst,
                v_icms_deson, mot_des_icms, ind_deduz_deson, v_icmsst_deson, mot_des_icms_st,
            })
        }
        "90" => {
            // Cada bloco do grupo é opcional: entra o ICMS próprio se houver alíquota, o ST
            // se houver alíquota de ST.
            let mut g = ICMS90 { orig, cst: codigo.clone(), ..Default::default() };
            let mut v_icms = 0.0;
            let mut v_fcp_proprio = 0.0;
            if p.p_icms.is_some() {
                let (mod_bc, v_bc, p_icms, v_icms_op) = icms_proprio(p, base, q, true, &codigo)?;
                // Diferimento no 90: vICMSOp, pDif e vICMSDif; o vICMS é o que sobra.
                let (vi, dif) = match p.p_dif.filter(|v| *v > 0.0) {
                    Some(p_dif) => {
                        let v_dif = arred2(v_icms_op * p_dif / 100.0);
                        (arred2(v_icms_op - v_dif), Some((v_icms_op, p_dif, v_dif)))
                    }
                    None => (v_icms_op, None),
                };
                let (v_bcfcp, p_fcp, v_fcp) = fcp(v_bc, p.p_fcp);
                v_icms = vi;
                v_fcp_proprio = v_fcp.unwrap_or(0.0);
                g.mod_bc = Some(mod_bc);
                g.v_bc = Some(v_bc);
                g.p_red_bc = p.p_red_bc.filter(|v| *v > 0.0);
                g.c_benef_rbc = c_benef_rbc(p, g.p_red_bc.is_some());
                g.p_icms = Some(p_icms);
                if let Some((op, p_dif, v_dif)) = dif {
                    g.v_icms_op = Some(op);
                    g.p_dif = Some(p_dif);
                    g.v_icms_dif = Some(v_dif);
                }
                g.v_icms = Some(vi);
                g.v_bcfcp = v_bcfcp;
                g.p_fcp = p_fcp;
                g.v_fcp = v_fcp;
                let (p_fcp_dif, v_fcp_dif, v_fcp_efet) = fcp_diferido(p, v_fcp);
                g.p_fcp_dif = p_fcp_dif;
                g.v_fcp_dif = v_fcp_dif;
                g.v_fcp_efet = v_fcp_efet;
            }
            if p.p_icmsst.is_some() {
                let st = st(p, base, q, v_icms, v_fcp_proprio, &codigo)?;
                let (v_icmsst_deson, mot_des_icms_st) = st_deson(p, &st, &codigo)?;
                g.mod_bcst = Some(st.mod_bcst);
                g.p_mvast = st.p_mvast;
                g.p_red_bcst = st.p_red_bcst;
                g.v_bcst = Some(st.v_bcst);
                g.p_icmsst = Some(st.p_icmsst);
                g.v_icmsst = Some(st.v_icmsst);
                g.v_bcfcpst = st.v_bcfcpst;
                g.p_fcpst = st.p_fcpst;
                g.v_fcpst = st.v_fcpst;
                g.v_icmsst_deson = v_icmsst_deson;
                g.mot_des_icms_st = mot_des_icms_st;
            }
            let (v_icms_deson, mot_des_icms, ind_deduz_deson) =
                desoneracao(p, base, v_icms, MOT_DES_70_90, &codigo)?;
            g.v_icms_deson = v_icms_deson;
            g.mot_des_icms = mot_des_icms;
            g.ind_deduz_deson = ind_deduz_deson;
            ICMSProcess::ICMS90(g)
        }

        // ── Monofásico de combustíveis (NT 2023.001) ─────────────────────────
        "02" => {
            let ad_rem = exigir(p.ad_rem_icms, "alíquota ad rem do ICMS (adRemICMS)", &codigo)?;
            ICMSProcess::ICMS02(ICMS02 {
                orig, cst: codigo, q_bc_mono: Some(q), ad_rem_icms: ad_rem,
                v_icms_mono: arred2(q * ad_rem),
            })
        }
        "15" => {
            let ad_rem = exigir(p.ad_rem_icms, "alíquota ad rem do ICMS (adRemICMS)", &codigo)?;
            let ad_rem_reten = exigir(p.ad_rem_icms_reten, "alíquota ad rem do ICMS retido (adRemICMSReten)", &codigo)?;
            let (p_red_ad_rem, mot_red_ad_rem) = match p.p_red_ad_rem.filter(|v| *v > 0.0) {
                Some(pr) => {
                    let mot = p.mot_red_ad_rem.unwrap_or(9);
                    if mot != 1 && mot != 9 {
                        return Err(DfeError::Validacao(format!(
                            "CST 15: motivo da redução do ad rem {mot} não existe (1 = transporte coletivo, 9 = outros)."
                        )));
                    }
                    (Some(fmt_dec(pr, 2)), Some(mot))
                }
                None => (None, None),
            };
            ICMSProcess::ICMS15(ICMS15 {
                orig, cst: codigo, q_bc_mono: Some(q), ad_rem_icms: ad_rem,
                v_icms_mono: arred2(q * ad_rem),
                q_bc_mono_reten: Some(q), ad_rem_icms_reten: ad_rem_reten,
                v_icms_mono_reten: arred2(q * ad_rem_reten),
                p_red_ad_rem, mot_red_ad_rem,
            })
        }
        "53" => match p.ad_rem_icms.filter(|v| *v > 0.0) {
            // Todos os campos são opcionais: sem ad rem, só orig + CST.
            None => ICMSProcess::ICMS53(ICMS53 { orig, cst: codigo, ..Default::default() }),
            Some(ad_rem) => {
                let op = arred2(q * ad_rem);
                let p_dif = p.p_dif.unwrap_or(0.0);
                let dif = arred2(op * p_dif / 100.0);
                ICMSProcess::ICMS53(ICMS53 {
                    orig, cst: codigo, q_bc_mono: Some(q), ad_rem_icms: Some(ad_rem),
                    v_icms_mono_op: Some(op), p_dif: Some(p_dif), v_icms_mono_dif: Some(dif),
                    v_icms_mono: Some(arred2(op - dif)),
                })
            }
        },
        "61" => {
            let ad_rem = exigir(p.ad_rem_icms_ret, "alíquota ad rem do ICMS retido anteriormente (adRemICMSRet)", &codigo)?;
            ICMSProcess::ICMS61(ICMS61 {
                orig, cst: codigo, q_bc_mono_ret: Some(q), ad_rem_icms_ret: ad_rem,
                v_icms_mono_ret: arred2(q * ad_rem),
            })
        }

        // ── Simples Nacional ────────────────────────────────────────────────
        "101" => {
            let p_cred = exigir(p.p_cred_sn, "% de crédito do Simples (pCredSN)", &codigo)?;
            ICMSProcess::ICMSSN101(ICMSSN101 {
                orig, csosn: codigo,
                p_cred_sn: fmt_dec(p_cred, 2),
                v_cred_icmssn: fmt_dec(arred2(base * p_cred / 100.0), 2),
            })
        }
        "102" | "103" | "300" | "400" => ICMSProcess::ICMSSN102(ICMSSN102 { orig, csosn: codigo }),
        "201" | "202" | "203" => {
            // No Simples o ICMS próprio não é destacado, mas é deduzido do ICMS-ST pela
            // alíquota interna (`p_icms`) quando o cadastro a informa.
            let proprio = arred2(base * p.p_icms.unwrap_or(0.0) / 100.0);
            let st = st(p, base, q, proprio, 0.0, &codigo)?;
            if codigo == "201" {
                let p_cred = exigir(p.p_cred_sn, "% de crédito do Simples (pCredSN)", &codigo)?;
                ICMSProcess::ICMSSN201(ICMSSN201 {
                    orig, csosn: codigo,
                    mod_bcst: st.mod_bcst, p_mvast: st.p_mvast, p_red_bcst: st.p_red_bcst,
                    v_bcst: st.v_bcst, p_icmsst: st.p_icmsst, v_icmsst: st.v_icmsst,
                    v_bcfcpst: st.v_bcfcpst, p_fcpst: st.p_fcpst, v_fcpst: st.v_fcpst,
                    p_cred_sn: p_cred, v_cred_icmssn: arred2(base * p_cred / 100.0),
                })
            } else {
                ICMSProcess::ICMSSN202(ICMSSN202 {
                    orig, csosn: codigo,
                    mod_bcst: st.mod_bcst, p_mvast: st.p_mvast, p_red_bcst: st.p_red_bcst,
                    v_bcst: st.v_bcst, p_icmsst: st.p_icmsst, v_icmsst: st.v_icmsst,
                    v_bcfcpst: st.v_bcfcpst, p_fcpst: st.p_fcpst, v_fcpst: st.v_fcpst,
                })
            }
        }
        "500" => {
            let (vbcst_ret, p_st, v_icms_substituto, vicmsst_ret) =
                grupo_st_retido(p.v_bcst_ret, p.p_st, p.v_icms_substituto, p.v_icmsst_ret);
            let r = retido_extra(p, base, vbcst_ret.as_deref());
            ICMSProcess::ICMSSN500(ICMSSN500 {
                orig, csosn: codigo, vbcst_ret, p_st, v_icms_substituto, vicmsst_ret,
                v_bcfcpst_ret: r.v_bcfcpst_ret, p_fcpst_ret: r.p_fcpst_ret, v_fcpst_ret: r.v_fcpst_ret,
                p_red_bc_efet: r.p_red_bc_efet, v_bc_efet: r.v_bc_efet,
                p_icms_efet: r.p_icms_efet, v_icms_efet: r.v_icms_efet,
            })
        }
        "900" => {
            let mut g = ICMSSN900 { orig, csosn: codigo.clone(), ..Default::default() };
            let mut v_icms = 0.0;
            if p.p_icms.is_some() {
                let (mod_bc, v_bc, p_icms, vi) = icms_proprio(p, base, q, true, &codigo)?;
                v_icms = vi;
                g.modbc = Some(mod_bc.to_string());
                g.vbc = Some(fmt_dec(v_bc, 2));
                g.pred_bc = p.p_red_bc.filter(|v| *v > 0.0).map(|v| fmt_dec(v, 4));
                g.picms = Some(fmt_dec(p_icms, 4));
                g.vicms = Some(fmt_dec(vi, 2));
            }
            if p.p_icmsst.is_some() {
                let st = st(p, base, q, v_icms, 0.0, &codigo)?;
                g.modbcst = Some(st.mod_bcst.to_string());
                g.pmvast = st.p_mvast.map(|v| fmt_dec(v, 4));
                g.pred_bcst = st.p_red_bcst.map(|v| fmt_dec(v, 4));
                g.vbcst = Some(fmt_dec(st.v_bcst, 2));
                g.picmsst = Some(fmt_dec(st.p_icmsst, 4));
                g.vicmsst = Some(fmt_dec(st.v_icmsst, 2));
                g.vbcfcpst = st.v_bcfcpst.map(|v| fmt_dec(v, 2));
                g.pfcpst = st.p_fcpst.map(|v| fmt_dec(v, 4));
                g.vfcpst = st.v_fcpst.map(|v| fmt_dec(v, 2));
            }
            if let Some(p_cred) = p.p_cred_sn.filter(|v| *v > 0.0) {
                g.pcred_sn = Some(fmt_dec(p_cred, 4));
                g.vcred_icmssn = Some(fmt_dec(arred2(base * p_cred / 100.0), 2));
            }
            ICMSProcess::ICMSSN900(g)
        }
        outro => {
            return Err(DfeError::Validacao(format!(
                "CST/CSOSN [{outro}] não reconhecido no cadastro do produto."
            )))
        }
    })
}

/// ICMSPart — partilha (CST 10, 20 ou 90 com `uf_st`). Próprio + ST obrigatórios.
fn icms_part(p: &IcmsParametros, base: f64, q: f64, codigo: &str) -> Result<ICMSProcess> {
    let uf_st = p.uf_st.as_deref().unwrap_or("").trim().to_uppercase();
    let p_bc_op = exigir(p.p_bc_op, "% da BC da operação própria (pBCOp)", codigo)?;
    let com_reducao = codigo != "10";
    let (mod_bc, v_bc, p_icms, v_icms) = icms_proprio(p, base, q, com_reducao, codigo)?;
    let st = st(p, base, q, v_icms, 0.0, codigo)?;
    let (v_icms_deson, mot_des_icms, ind_deduz_deson) =
        desoneracao(p, base, v_icms, MOT_DES_PART, codigo)?;
    Ok(ICMSProcess::ICMSPart(ICMSPart {
        orig: p.orig, cst: codigo.to_string(), mod_bc, v_bc,
        p_red_bc: if com_reducao { p.p_red_bc.filter(|v| *v > 0.0) } else { None },
        p_icms, v_icms,
        mod_bcst: st.mod_bcst, p_mvast: st.p_mvast, p_red_bcst: st.p_red_bcst,
        v_bcst: st.v_bcst, p_icmsst: st.p_icmsst, v_icmsst: st.v_icmsst,
        v_bcfcpst: st.v_bcfcpst, p_fcpst: st.p_fcpst, v_fcpst: st.v_fcpst,
        p_bc_op, uf_st,
        v_icms_deson, mot_des_icms, ind_deduz_deson,
    }))
}

/// ICMSST — repasse de ICMS-ST retido (CST 41 ou 60 com `v_bcst_dest`). Os valores retidos
/// e os da UF de destino são da operação, não do cadastro: vêm prontos.
fn icms_st_repasse(p: &IcmsParametros, base: f64, codigo: &str) -> Result<ICMSProcess> {
    let v_bcst_ret = exigir(p.v_bcst_ret, "BC do ICMS-ST retido (vBCSTRet)", codigo)?;
    let v_icmsst_ret = exigir(p.v_icmsst_ret, "ICMS-ST retido (vICMSSTRet)", codigo)?;
    let v_bcst_dest = exigir(p.v_bcst_dest, "BC do ICMS-ST da UF de destino (vBCSTDest)", codigo)?;
    let v_icmsst_dest = exigir(p.v_icmsst_dest, "ICMS-ST da UF de destino (vICMSSTDest)", codigo)?;
    let bc_ret = fmt_dec(v_bcst_ret, 2);
    let r = retido_extra(p, base, Some(&bc_ret));
    Ok(ICMSProcess::ICMSST(ICMSST {
        orig: p.orig, cst: codigo.to_string(),
        v_bcst_ret: arred2(v_bcst_ret),
        p_st: p.p_st.filter(|v| *v > 0.0),
        v_icms_substituto: p.v_icms_substituto.filter(|v| *v > 0.0),
        v_icmsst_ret: arred2(v_icmsst_ret),
        v_bcfcpst_ret: r.v_bcfcpst_ret, p_fcpst_ret: r.p_fcpst_ret, v_fcpst_ret: r.v_fcpst_ret,
        v_bcst_dest: arred2(v_bcst_dest), v_icmsst_dest: arred2(v_icmsst_dest),
        p_red_bc_efet: r.p_red_bc_efet, v_bc_efet: r.v_bc_efet,
        p_icms_efet: r.p_icms_efet, v_icms_efet: r.v_icms_efet,
    }))
}

/// "ICMS20" → "20", "ICMSSN201" → "201", " 40 " → "40".
fn normalizar_cst(cst: &str) -> String {
    let c = cst.trim().to_uppercase();
    let c = c.strip_prefix("ICMSSN").or_else(|| c.strip_prefix("ICMS")).unwrap_or(&c);
    c.to_string()
}

fn exigir(v: Option<f64>, campo: &str, cst: &str) -> Result<f64> {
    v.ok_or_else(|| {
        DfeError::Validacao(format!(
            "CST/CSOSN {cst}: {campo} é obrigatório. Preencha no cadastro de ICMS do produto."
        ))
    })
}

/// (modBC, BC antes da redução) da operação própria, pela modalidade do cadastro.
fn base_propria(p: &IcmsParametros, base: f64, q: f64, cst: &str) -> Result<(u8, f64)> {
    match p.mod_bc.unwrap_or(3) {
        3 => Ok((3, base)),
        0 => {
            let mva = exigir(p.p_mva_proprio, "% de MVA do ICMS próprio (modalidade 0)", cst)?;
            Ok((0, arred2(base * (1.0 + mva / 100.0))))
        }
        m @ (1 | 2) => {
            let pauta = exigir(p.v_pauta, "valor unitário de pauta / preço tabelado (modalidade 1 ou 2)", cst)?;
            Ok((m, arred2(q * pauta)))
        }
        m => Err(DfeError::Validacao(format!(
            "CST/CSOSN {cst}: modalidade da BC do ICMS {m} não existe no leiaute (0, 1, 2 ou 3)."
        ))),
    }
}

/// (modBC, vBC, pICMS, vICMS) da operação própria. `com_reducao` aplica o pRedBC do cadastro.
fn icms_proprio(p: &IcmsParametros, base: f64, q: f64, com_reducao: bool, cst: &str) -> Result<(u8, f64, f64, f64)> {
    let (mod_bc, bruta) = base_propria(p, base, q, cst)?;
    let p_icms = exigir(p.p_icms, "alíquota do ICMS (pICMS)", cst)?;
    let red = if com_reducao { p.p_red_bc.unwrap_or(0.0) } else { 0.0 };
    let v_bc = arred2(bruta * (1.0 - red / 100.0));
    Ok((mod_bc, v_bc, p_icms, arred2(v_bc * p_icms / 100.0)))
}

/// (vBCFCP, pFCP, vFCP) — ausentes quando o cadastro não tem FCP.
fn fcp(v_bc: f64, p_fcp: Option<f64>) -> (Option<f64>, Option<f64>, Option<f64>) {
    match p_fcp.filter(|v| *v > 0.0) {
        Some(pf) => (Some(v_bc), Some(pf), Some(arred2(v_bc * pf / 100.0))),
        None => (None, None, None),
    }
}

/// (pFCPDif, vFCPDif, vFCPEfet) — FCP diferido (CST 51 e 90), só com FCP e pFCPDif.
fn fcp_diferido(p: &IcmsParametros, v_fcp: Option<f64>) -> (Option<f64>, Option<f64>, Option<f64>) {
    match (v_fcp, p.p_fcp_dif.filter(|v| *v > 0.0)) {
        (Some(v), Some(pd)) => {
            let dif = arred2(v * pd / 100.0);
            (Some(pd), Some(dif), Some(arred2(v - dif)))
        }
        _ => (None, None, None),
    }
}

/// cBenefRBC (CST 51/90) — só acompanha a redução da BC.
fn c_benef_rbc(p: &IcmsParametros, tem_reducao: bool) -> Option<String> {
    if !tem_reducao {
        return None;
    }
    p.c_benef_rbc.as_deref().map(|c| c.trim().to_uppercase()).filter(|c| !c.is_empty())
}

struct St {
    mod_bcst: u8,
    p_mvast: Option<f64>,
    p_red_bcst: Option<f64>,
    v_bcst: f64,
    p_icmsst: f64,
    v_icmsst: f64,
    v_bcfcpst: Option<f64>,
    p_fcpst: Option<f64>,
    v_fcpst: Option<f64>,
    /// ICMS-ST sem a redução da BC-ST — base do vICMSSTDeson.
    v_icmsst_sem_reducao: f64,
}

/// ICMS-ST sobre a base do item. `deduzir` é o ICMS da operação própria e
/// `fcp_proprio` o FCP dela; ambos saem do valor retido.
fn st(p: &IcmsParametros, base: f64, q: f64, deduzir: f64, fcp_proprio: f64, cst: &str) -> Result<St> {
    let mod_bcst = p.mod_bcst.unwrap_or(4);
    let mva = p.p_mvast.unwrap_or(0.0);
    let red = p.p_red_bcst.unwrap_or(0.0);
    let bruta = match mod_bcst {
        4 => base * (1.0 + mva / 100.0),
        6 => base,
        0 | 1 | 2 | 3 | 5 => {
            let pauta = exigir(
                p.v_pauta_st,
                "valor unitário de referência do ICMS-ST (preço tabelado, lista ou pauta)",
                cst,
            )?;
            q * pauta
        }
        m => {
            return Err(DfeError::Validacao(format!(
                "CST/CSOSN {cst}: modalidade da BC do ICMS-ST {m} não existe no leiaute (0 a 6)."
            )))
        }
    };
    let v_bcst = arred2(bruta * (1.0 - red / 100.0));
    let p_icmsst = exigir(p.p_icmsst, "alíquota do ICMS-ST (pICMSST)", cst)?;
    let v_icmsst = arred2((v_bcst * p_icmsst / 100.0 - deduzir).max(0.0));
    let v_icmsst_sem_reducao = arred2((arred2(bruta) * p_icmsst / 100.0 - deduzir).max(0.0));
    let (v_bcfcpst, p_fcpst, v_fcpst) = match p.p_fcpst.filter(|v| *v > 0.0) {
        Some(pf) => (
            Some(v_bcst),
            Some(pf),
            Some(arred2((v_bcst * pf / 100.0 - fcp_proprio).max(0.0))),
        ),
        None => (None, None, None),
    };
    Ok(St {
        mod_bcst,
        // pMVAST/pRedBCST são opcionais no XSD: só entram com valor.
        p_mvast: Some(mva).filter(|v| *v > 0.0 && mod_bcst == 4),
        p_red_bcst: Some(red).filter(|v| *v > 0.0),
        v_bcst,
        p_icmsst,
        v_icmsst,
        v_bcfcpst,
        p_fcpst,
        v_fcpst,
        v_icmsst_sem_reducao,
    })
}

/// (vICMSSTDeson, motDesICMSST) — CST 10, 70, 90. Só com o motivo preenchido.
/// vICMSSTDeson = ICMS-ST sem a redução da BC-ST − ICMS-ST destacado.
fn st_deson(p: &IcmsParametros, st: &St, cst: &str) -> Result<(Option<f64>, Option<u16>)> {
    let Some(mot) = p.mot_des_icms_st else {
        return Ok((None, None));
    };
    if !MOT_DES_ST.contains(&mot) {
        return Err(DfeError::Validacao(format!(
            "CST {cst}: motivo de desoneração do ICMS-ST {mot} não é aceito (aceitos: {MOT_DES_ST:?})."
        )));
    }
    Ok((Some(arred2((st.v_icmsst_sem_reducao - st.v_icmsst).max(0.0))), Some(mot)))
}

/// (vICMSDeson, motDesICMS, indDeduzDeson). Só existe com o motivo preenchido.
/// vICMSDeson = ICMS que seria devido sem o benefício (base × pICMS) − ICMS destacado.
fn desoneracao(
    p: &IcmsParametros,
    base: f64,
    v_icms_destacado: f64,
    motivos: &[u16],
    cst: &str,
) -> Result<(Option<f64>, Option<u16>, Option<u8>)> {
    let Some(mot) = p.mot_des_icms else {
        return Ok((None, None, None));
    };
    if !motivos.contains(&mot) {
        return Err(DfeError::Validacao(format!(
            "CST {cst}: motivo de desoneração {mot} não é aceito neste CST (aceitos: {:?}).",
            motivos
        )));
    }
    let p_icms = exigir(
        p.p_icms,
        "alíquota do ICMS (pICMS) que seria devida sem o benefício, para calcular a desoneração",
        cst,
    )?;
    let devido = arred2(base * p_icms / 100.0);
    let v = arred2((devido - v_icms_destacado).max(0.0));
    Ok((Some(v), Some(mot), p.ind_deduz_deson))
}

/// FCP-ST retido e ICMS efetivo (CST 60, CSOSN 500, ICMSST).
#[derive(Default)]
struct RetidoExtra {
    v_bcfcpst_ret: Option<f64>,
    p_fcpst_ret: Option<f64>,
    v_fcpst_ret: Option<f64>,
    p_red_bc_efet: Option<f64>,
    v_bc_efet: Option<f64>,
    p_icms_efet: Option<f64>,
    v_icms_efet: Option<f64>,
}

/// FCP-ST retido sobre a BC do ST retido (ou a base do item, sem ela); efetivo sobre a base
/// do item: vBCEfet = base × (1 − pRedBCEfet), vICMSEfet = vBCEfet × pICMSEfet.
fn retido_extra(p: &IcmsParametros, base: f64, v_bcst_ret: Option<&str>) -> RetidoExtra {
    let mut r = RetidoExtra::default();
    if let Some(pf) = p.p_fcpst_ret.filter(|v| *v > 0.0) {
        let bc = v_bcst_ret.and_then(|s| s.parse::<f64>().ok()).unwrap_or(base);
        r.v_bcfcpst_ret = Some(arred2(bc));
        r.p_fcpst_ret = Some(pf);
        r.v_fcpst_ret = Some(arred2(bc * pf / 100.0));
    }
    if let Some(pe) = p.p_icms_efet.filter(|v| *v > 0.0) {
        let red = p.p_red_bc_efet.unwrap_or(0.0);
        let bc = arred2(base * (1.0 - red / 100.0));
        r.p_red_bc_efet = Some(red);
        r.v_bc_efet = Some(bc);
        r.p_icms_efet = Some(pe);
        r.v_icms_efet = Some(arred2(bc * pe / 100.0));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emissao::det_process::entity::ImpostoProcess;
    use crate::interno::validation::validar_fragmento;

    fn par(cst: &str) -> IcmsParametros {
        IcmsParametros { cst: cst.into(), orig: 0, ..Default::default() }
    }

    /// Serializa só o `<ICMS>` e valida contra o leiaute embutido.
    fn xml_valido(g: &ICMSProcess) -> String {
        let imp = ImpostoProcess {
            v_tot_trib: "0.00".into(),
            icms: g.clone(),
            ipi: None,
            pis: PISProcess::default(),
            cofins: COFINSProcess::default(),
            ibs_cbs: None,
        };
        let xml = imp.to_xml();
        let ini = xml.find("<ICMS>").unwrap();
        let fim = xml.find("</ICMS>").unwrap() + "</ICMS>".len();
        let icms = xml[ini..fim].to_string();
        if let Err(e) = validar_fragmento("ICMS", &icms) {
            panic!("XSD recusou: {e}");
        }
        icms
    }

    #[test]
    fn cst00_calcula_icms_e_fcp_sobre_a_base() {
        let p = IcmsParametros { p_icms: Some(18.0), p_fcp: Some(2.0), ..par("00") };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        assert!(xml.contains("<vBC>100.00</vBC><pICMS>18.0000</pICMS><vICMS>18.00</vICMS><pFCP>2.0000</pFCP><vFCP>2.00</vFCP>"), "{xml}");
    }

    #[test]
    fn cst10_nao_vira_mais_cst00_e_deduz_o_proprio_do_st() {
        let p = IcmsParametros {
            p_icms: Some(18.0), mod_bcst: Some(4), p_mvast: Some(40.0), p_icmsst: Some(18.0),
            ..par("ICMS10")
        };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        // vBCST = 100 × 1,40 = 140; vICMSST = 140 × 18% − 18 = 7,20
        assert!(xml.contains("<ICMS10><orig>0</orig><CST>10</CST>"), "{xml}");
        assert!(xml.contains("<vBCST>140.00</vBCST><pICMSST>18.0000</pICMSST><vICMSST>7.20</vICMSST>"), "{xml}");
    }

    #[test]
    fn cst20_aplica_reducao_e_calcula_desoneracao() {
        let p = IcmsParametros {
            p_icms: Some(18.0), p_red_bc: Some(33.33), mot_des_icms: Some(9), ind_deduz_deson: Some(0),
            ..par("20")
        };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        // vBC = 66,67; vICMS = 12,00; desonerado = 18,00 − 12,00 = 6,00
        assert!(xml.contains("<pRedBC>33.3300</pRedBC><vBC>66.67</vBC><pICMS>18.0000</pICMS><vICMS>12.00</vICMS>"), "{xml}");
        assert!(xml.contains("<vICMSDeson>6.00</vICMSDeson><motDesICMS>9</motDesICMS><indDeduzDeson>0</indDeduzDeson>"), "{xml}");
    }

    #[test]
    fn cst20_sem_reducao_e_erro_de_cadastro() {
        let p = IcmsParametros { p_icms: Some(18.0), ..par("20") };
        let e = icms_de_parametros(&p, 100.0, 1.0).unwrap_err().to_string();
        assert!(e.contains("pRedBC"), "{e}");
    }

    #[test]
    fn cst30_st_sem_deducao() {
        let p = IcmsParametros { p_mvast: Some(50.0), p_icmsst: Some(18.0), p_fcpst: Some(2.0), ..par("30") };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        assert!(xml.contains("<vBCST>150.00</vBCST><pICMSST>18.0000</pICMSST><vICMSST>27.00</vICMSST><vBCFCPST>150.00</vBCFCPST><pFCPST>2.0000</pFCPST><vFCPST>3.00</vFCPST>"), "{xml}");
    }

    #[test]
    fn cst40_41_50_sem_e_com_desoneracao() {
        for cst in ["40", "41", "50"] {
            let xml = xml_valido(&icms_de_parametros(&par(cst), 100.0, 1.0).unwrap());
            assert!(xml.contains(&format!("<CST>{cst}</CST></ICMS40>")), "{xml}");
        }
        let p = IcmsParametros { p_icms: Some(18.0), mot_des_icms: Some(7), ..par("40") };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        assert!(xml.contains("<vICMSDeson>18.00</vICMSDeson><motDesICMS>7</motDesICMS>"), "{xml}");
    }

    #[test]
    fn motivo_de_desoneracao_fora_da_lista_do_cst_e_recusado() {
        let p = IcmsParametros { p_icms: Some(18.0), mot_des_icms: Some(1), ..par("30") };
        assert!(icms_de_parametros(&p, 100.0, 1.0).is_err());
    }

    #[test]
    fn cst51_diferimento_parcial_e_sem_aliquota() {
        let p = IcmsParametros { p_icms: Some(18.0), p_dif: Some(33.33), ..par("51") };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        // vICMSOp 18,00; diferido 6,00; devido 12,00
        assert!(xml.contains("<vICMSOp>18.00</vICMSOp><pDif>33.3300</pDif><vICMSDif>6.00</vICMSDif><vICMS>12.00</vICMS>"), "{xml}");
        let xml = xml_valido(&icms_de_parametros(&par("51"), 100.0, 1.0).unwrap());
        assert!(xml.contains("<ICMS51><orig>0</orig><CST>51</CST></ICMS51>"), "{xml}");
    }

    #[test]
    fn cst60_repassa_o_retido() {
        let p = IcmsParametros { v_bcst_ret: Some(50.0), p_st: Some(18.0), v_icmsst_ret: Some(9.0), ..par("60") };
        xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        xml_valido(&icms_de_parametros(&par("60"), 100.0, 1.0).unwrap());
    }

    #[test]
    fn cst70_reducao_mais_st() {
        let p = IcmsParametros {
            p_icms: Some(18.0), p_red_bc: Some(10.0), p_mvast: Some(40.0), p_icmsst: Some(18.0),
            mot_des_icms: Some(3),
            ..par("70")
        };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        // vBC 90; vICMS 16,20; vBCST 140; vICMSST 25,20 − 16,20 = 9,00; deson 1,80
        assert!(xml.contains("<vBC>90.00</vBC><pICMS>18.0000</pICMS><vICMS>16.20</vICMS>"), "{xml}");
        assert!(xml.contains("<vICMSST>9.00</vICMSST><vICMSDeson>1.80</vICMSDeson><motDesICMS>3</motDesICMS>"), "{xml}");
    }

    #[test]
    fn cst90_blocos_opcionais_e_ordem_do_xsd() {
        xml_valido(&icms_de_parametros(&par("90"), 100.0, 1.0).unwrap());
        let p = IcmsParametros { p_icms: Some(12.0), p_red_bc: Some(50.0), p_icmsst: Some(18.0), ..par("90") };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        assert!(xml.contains("<modBC>3</modBC><vBC>50.00</vBC><pRedBC>50.0000</pRedBC><pICMS>12.0000</pICMS><vICMS>6.00</vICMS>"), "{xml}");
    }

    #[test]
    fn simples_101_102_500_900() {
        let p = IcmsParametros { p_cred_sn: Some(1.25), ..par("101") };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        assert!(xml.contains("<pCredSN>1.25</pCredSN><vCredICMSSN>1.25</vCredICMSSN>"), "{xml}");
        for c in ["102", "103", "300", "400"] {
            let xml = xml_valido(&icms_de_parametros(&par(c), 100.0, 1.0).unwrap());
            assert!(xml.contains(&format!("<CSOSN>{c}</CSOSN></ICMSSN102>")), "{xml}");
        }
        xml_valido(&icms_de_parametros(&par("500"), 100.0, 1.0).unwrap());
        xml_valido(&icms_de_parametros(&par("900"), 100.0, 1.0).unwrap());
        let p = IcmsParametros { p_icms: Some(18.0), p_icmsst: Some(18.0), p_mvast: Some(30.0), p_cred_sn: Some(2.0), ..par("900") };
        xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
    }

    #[test]
    fn simples_201_202_203_ganham_grupo_proprio() {
        let st = IcmsParametros { p_icms: Some(18.0), p_mvast: Some(40.0), p_icmsst: Some(18.0), ..Default::default() };
        let p = IcmsParametros { cst: "201".into(), p_cred_sn: Some(1.5), ..st.clone() };
        let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
        assert!(xml.contains("<ICMSSN201><orig>0</orig><CSOSN>201</CSOSN>"), "{xml}");
        assert!(xml.contains("<vICMSST>7.20</vICMSST><pCredSN>1.5000</pCredSN><vCredICMSSN>1.50</vCredICMSSN>"), "{xml}");
        for c in ["202", "203"] {
            let p = IcmsParametros { cst: c.into(), ..st.clone() };
            let xml = xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap());
            assert!(xml.contains(&format!("<ICMSSN202><orig>0</orig><CSOSN>{c}</CSOSN>")), "{xml}");
        }
    }

    #[test]
    fn modalidade_de_pauta_nao_e_inventada() {
        let p = IcmsParametros { p_icms: Some(18.0), mod_bc: Some(1), ..par("00") };
        assert!(icms_de_parametros(&p, 100.0, 1.0).is_err());
        let p = IcmsParametros { p_icmsst: Some(18.0), mod_bcst: Some(5), ..par("30") };
        assert!(icms_de_parametros(&p, 100.0, 1.0).is_err());
    }

    #[test]
    fn cst_desconhecido_e_erro() {
        assert!(icms_de_parametros(&par("99"), 100.0, 1.0).is_err());
    }

    // ── Bateria por código: espelho da matriz da retaguarda (`icmsMatriz.js`) ──────────
    //
    // Para cada CST/CSOSN que o cadastro oferece: (a) só os obrigatórios da matriz monta um
    // grupo válido no XSD; (b) todos os campos que a tela mostra para o código, preenchidos,
    // também — e cada parâmetro aparece no XML (nada que o operador digitou some calado).

    /// Só o que a matriz marca como obrigatório (`O`).
    fn minimo(codigo: &str) -> IcmsParametros {
        let p = par(codigo);
        let proprio = IcmsParametros { p_icms: Some(18.0), ..p.clone() };
        let st = |b: IcmsParametros| IcmsParametros { mod_bcst: Some(4), p_icmsst: Some(18.0), ..b };
        match codigo {
            "00" => proprio,
            "10" => st(proprio),
            "20" => IcmsParametros { p_red_bc: Some(33.33), ..proprio },
            "30" => st(p),
            "70" => st(IcmsParametros { p_red_bc: Some(10.0), ..proprio }),
            "101" => IcmsParametros { p_cred_sn: Some(1.25), ..p },
            "201" => st(IcmsParametros { p_cred_sn: Some(1.25), ..p }),
            "202" | "203" => st(p),
            _ => p, // 40, 41, 50, 51, 60, 90, 102, 103, 300, 400, 500, 900: nada obrigatório
        }
    }

    /// Todo campo que a matriz mostra para o código, preenchido. `tags` = o que tem de sair
    /// no XML por causa dele.
    fn completo(codigo: &str) -> (IcmsParametros, Vec<&'static str>) {
        let proprio = IcmsParametros {
            p_icms: Some(18.0), p_fcp: Some(2.0), ..par(codigo)
        };
        let st = |b: IcmsParametros| IcmsParametros {
            mod_bcst: Some(4), p_mvast: Some(40.0), p_red_bcst: Some(10.0),
            p_icmsst: Some(18.0), p_fcpst: Some(2.0), ..b
        };
        let deson = |b: IcmsParametros, mot: u16| IcmsParametros {
            mot_des_icms: Some(mot), ind_deduz_deson: Some(1), ..b
        };
        const PROP: &[&str] = &["<pICMS>", "<pFCP>"];
        const ST: &[&str] = &["<modBCST>4", "<pMVAST>", "<pRedBCST>", "<pICMSST>", "<pFCPST>"];
        const DES: &[&str] = &["<vICMSDeson>", "<motDesICMS>", "<indDeduzDeson>1"];
        let junta = |xs: &[&[&'static str]]| xs.concat();
        match codigo {
            "00" => (proprio, PROP.to_vec()),
            "10" => (st(proprio), junta(&[PROP, ST])),
            "20" => (deson(IcmsParametros { p_red_bc: Some(33.33), ..proprio }, 9), junta(&[PROP, &["<pRedBC>"], DES])),
            "30" => (deson(st(IcmsParametros { p_icms: Some(18.0), ..par(codigo) }), 7), junta(&[ST, DES])),
            "40" | "41" | "50" => (deson(IcmsParametros { p_icms: Some(18.0), ..par(codigo) }, 9), DES.to_vec()),
            "51" => (
                IcmsParametros { p_red_bc: Some(10.0), p_dif: Some(33.33), ..proprio },
                junta(&[PROP, &["<pRedBC>", "<pDif>", "<vICMSDif>"]]),
            ),
            "60" => (IcmsParametros { v_bcst_ret: Some(50.0), p_st: Some(18.0), v_icmsst_ret: Some(9.0), ..par(codigo) }, vec!["<pST>"]),
            "70" => (deson(st(IcmsParametros { p_red_bc: Some(10.0), ..proprio }), 3), junta(&[PROP, &["<pRedBC>"], ST, DES])),
            "90" => (deson(st(IcmsParametros { p_red_bc: Some(10.0), ..proprio }), 12), junta(&[PROP, &["<pRedBC>"], ST, DES])),
            "101" => (IcmsParametros { p_cred_sn: Some(1.25), ..par(codigo) }, vec!["<pCredSN>"]),
            "201" => (
                st(IcmsParametros { p_icms: Some(18.0), p_cred_sn: Some(1.25), ..par(codigo) }),
                junta(&[ST, &["<pCredSN>"]]),
            ),
            "202" | "203" => (st(IcmsParametros { p_icms: Some(18.0), ..par(codigo) }), ST.to_vec()),
            "500" => (IcmsParametros { v_bcst_ret: Some(50.0), p_st: Some(18.0), v_icmsst_ret: Some(9.0), ..par(codigo) }, vec!["<pST>"]),
            "900" => (
                st(IcmsParametros { p_icms: Some(18.0), p_red_bc: Some(10.0), p_cred_sn: Some(1.25), ..par(codigo) }),
                // Sem FCP no ICMSSN900: o XSD não tem pFCP próprio no Simples.
                vec!["<pICMS>", "<pRedBC>", "<modBCST>4", "<pMVAST>", "<pRedBCST>", "<pICMSST>", "<pFCPST>", "<pCredSN>"],
            ),
            _ => (par(codigo), vec![]), // 102, 103, 300, 400: só orig + CSOSN
        }
    }

    const TODOS: &[&str] = &[
        "00", "10", "20", "30", "40", "41", "50", "51", "60", "70", "90",
        "101", "102", "103", "201", "202", "203", "300", "400", "500", "900",
    ];

    #[test]
    fn bateria_todo_codigo_com_so_os_obrigatorios_passa_no_xsd() {
        for c in TODOS {
            let g = icms_de_parametros(&minimo(c), 100.0, 1.0)
                .unwrap_or_else(|e| panic!("{c}: mínimo da matriz recusado: {e}"));
            let xml = xml_valido(&g);
            let tag = if c.len() == 3 { format!("<CSOSN>{c}</CSOSN>") } else { format!("<CST>{c}</CST>") };
            assert!(xml.contains(&tag), "{c}: {xml}");
        }
    }

    #[test]
    fn bateria_todo_codigo_com_todos_os_campos_passa_no_xsd_e_nada_some() {
        for c in TODOS {
            let (p, tags) = completo(c);
            let g = icms_de_parametros(&p, 100.0, 1.0)
                .unwrap_or_else(|e| panic!("{c}: cadastro completo recusado: {e}"));
            let xml = xml_valido(&g);
            for t in tags {
                assert!(xml.contains(t), "{c}: {t} sumiu do XML: {xml}");
            }
        }
    }

    #[test]
    fn bateria_obrigatorio_ausente_e_erro_de_cadastro() {
        // Cada obrigatório da matriz, tirado do mínimo, tem de virar erro — não XML torto.
        let casos: &[(&str, fn(&mut IcmsParametros))] = &[
            ("00", |p| p.p_icms = None),
            ("10", |p| p.p_icms = None),
            ("10", |p| p.p_icmsst = None),
            ("20", |p| p.p_red_bc = None),
            ("20", |p| p.p_icms = None),
            ("30", |p| p.p_icmsst = None),
            ("70", |p| p.p_red_bc = None),
            ("70", |p| p.p_icmsst = None),
            ("101", |p| p.p_cred_sn = None),
            ("201", |p| p.p_cred_sn = None),
            ("201", |p| p.p_icmsst = None),
            ("202", |p| p.p_icmsst = None),
            ("203", |p| p.p_icmsst = None),
        ];
        for (c, tira) in casos {
            let mut p = minimo(c);
            tira(&mut p);
            assert!(icms_de_parametros(&p, 100.0, 1.0).is_err(), "{c}: faltou obrigatório e montou assim mesmo");
        }
    }

    #[test]
    fn bateria_motivos_de_desoneracao_sao_os_do_xsd() {
        // Todo motivo que a matriz oferece monta; um fora da lista é recusado.
        let listas: &[(&str, &[u16])] = &[
            ("20", MOT_DES_20), ("30", MOT_DES_30), ("40", MOT_DES_40), ("41", MOT_DES_40),
            ("50", MOT_DES_40), ("70", MOT_DES_70_90), ("90", MOT_DES_70_90),
        ];
        for (c, motivos) in listas {
            for m in *motivos {
                let p = IcmsParametros { mot_des_icms: Some(*m), p_icms: Some(18.0), ..minimo(c) };
                xml_valido(&icms_de_parametros(&p, 100.0, 1.0).unwrap_or_else(|e| panic!("{c}/{m}: {e}")));
            }
            let p = IcmsParametros { mot_des_icms: Some(2), p_icms: Some(18.0), ..minimo(c) };
            assert!(icms_de_parametros(&p, 100.0, 1.0).is_err(), "{c}: motivo 2 não existe em grupo nenhum");
        }
    }

    // ── Tudo do leiaute: modalidades, grupos opcionais, Part, ST de repasse, monofásico ─

    fn calc(p: &IcmsParametros, q: f64) -> ICMSProcess {
        icms_de_parametros(p, 100.0, q).unwrap_or_else(|e| panic!("{}: {e}", p.cst))
    }

    #[test]
    fn modalidades_da_bc_propria_0_1_2_3() {
        // 3: valor da operação
        let x = xml_valido(&calc(&IcmsParametros { p_icms: Some(18.0), mod_bc: Some(3), ..par("00") }, 2.0));
        assert!(x.contains("<modBC>3</modBC><vBC>100.00</vBC>"), "{x}");
        // 0: MVA própria 30% → 130
        let x = xml_valido(&calc(&IcmsParametros { p_icms: Some(18.0), mod_bc: Some(0), p_mva_proprio: Some(30.0), ..par("00") }, 2.0));
        assert!(x.contains("<modBC>0</modBC><vBC>130.00</vBC><pICMS>18.0000</pICMS><vICMS>23.40</vICMS>"), "{x}");
        // 1 pauta e 2 preço tabelado: 2 un × 45,50 = 91,00
        for m in [1u8, 2] {
            let x = xml_valido(&calc(&IcmsParametros { p_icms: Some(18.0), mod_bc: Some(m), v_pauta: Some(45.5), ..par("00") }, 2.0));
            assert!(x.contains(&format!("<modBC>{m}</modBC><vBC>91.00</vBC>")), "{x}");
        }
        // pauta com redução (CST 20): 91 × (1 − 10%) = 81,90
        let x = xml_valido(&calc(&IcmsParametros { p_icms: Some(18.0), mod_bc: Some(1), v_pauta: Some(45.5), p_red_bc: Some(10.0), ..par("20") }, 2.0));
        assert!(x.contains("<pRedBC>10.0000</pRedBC><vBC>81.90</vBC>"), "{x}");
        // sem o parâmetro da modalidade: erro de cadastro
        assert!(icms_de_parametros(&IcmsParametros { p_icms: Some(18.0), mod_bc: Some(0), ..par("00") }, 100.0, 1.0).is_err());
        assert!(icms_de_parametros(&IcmsParametros { p_icms: Some(18.0), mod_bc: Some(1), ..par("00") }, 100.0, 1.0).is_err());
        assert!(icms_de_parametros(&IcmsParametros { p_icms: Some(18.0), mod_bc: Some(7), ..par("00") }, 100.0, 1.0).is_err());
    }

    #[test]
    fn modalidades_da_bc_st_0_a_6_em_todo_grupo_com_st() {
        for m in [0u8, 1, 2, 3, 4, 5, 6] {
            let st = IcmsParametros {
                mod_bcst: Some(m), p_mvast: Some(40.0), v_pauta_st: Some(60.0), p_icmsst: Some(18.0),
                p_icms: Some(18.0), p_red_bc: Some(10.0), p_cred_sn: Some(1.5),
                ..Default::default()
            };
            for c in ["10", "30", "70", "90", "201", "202", "203", "900"] {
                let x = xml_valido(&calc(&IcmsParametros { cst: c.into(), ..st.clone() }, 2.0));
                let v_bcst = match m { 4 => "140.00", 6 => "100.00", _ => "120.00" };
                assert!(x.contains(&format!("<modBCST>{m}</modBCST>")), "{c}/{m}: {x}");
                assert!(x.contains(&format!("<vBCST>{v_bcst}</vBCST>")), "{c}/{m}: {x}");
            }
        }
        let sem_pauta = IcmsParametros { mod_bcst: Some(5), p_icmsst: Some(18.0), ..par("30") };
        assert!(icms_de_parametros(&sem_pauta, 100.0, 1.0).is_err());
    }

    #[test]
    fn desoneracao_do_icms_st_em_10_70_90() {
        for c in ["10", "70", "90"] {
            for mot in MOT_DES_ST {
                let p = IcmsParametros {
                    p_icms: Some(18.0), p_red_bc: Some(10.0), p_mvast: Some(40.0), p_red_bcst: Some(50.0),
                    p_icmsst: Some(18.0), mot_des_icms_st: Some(*mot), ..par(c)
                };
                let x = xml_valido(&calc(&p, 1.0));
                assert!(x.contains(&format!("<motDesICMSST>{mot}</motDesICMSST>")), "{c}: {x}");
                assert!(x.contains("<vICMSSTDeson>"), "{c}: {x}");
            }
            let p = IcmsParametros { p_icms: Some(18.0), p_red_bc: Some(10.0), p_icmsst: Some(18.0), mot_des_icms_st: Some(1), ..par(c) };
            assert!(icms_de_parametros(&p, 100.0, 1.0).is_err(), "{c}: motivo 1 não vale no ST");
        }
        // 10: próprio 18; ST sem redução 140 × 18% − 18 = 7,20; com 50% de redução 70 × 18% − 18 = 0
        let p = IcmsParametros { p_icms: Some(18.0), p_mvast: Some(40.0), p_red_bcst: Some(50.0), p_icmsst: Some(18.0), mot_des_icms_st: Some(9), ..par("10") };
        assert!(xml_valido(&calc(&p, 1.0)).contains("<vICMSST>0.00</vICMSST><vICMSSTDeson>7.20</vICMSSTDeson><motDesICMSST>9</motDesICMSST>"));
    }

    #[test]
    fn cst51_e_90_com_cbenef_rbc_e_fcp_diferido() {
        let p = IcmsParametros {
            p_icms: Some(18.0), p_red_bc: Some(10.0), c_benef_rbc: Some("sp851001".into()),
            p_dif: Some(50.0), p_fcp: Some(2.0), p_fcp_dif: Some(50.0), ..par("51")
        };
        let x = xml_valido(&calc(&p, 1.0));
        assert!(x.contains("<pRedBC>10.0000</pRedBC><cBenefRBC>SP851001</cBenefRBC><vBC>90.00</vBC>"), "{x}");
        assert!(x.contains("<vFCP>1.80</vFCP><pFCPDif>50.0000</pFCPDif><vFCPDif>0.90</vFCPDif><vFCPEfet>0.90</vFCPEfet>"), "{x}");

        let p90 = IcmsParametros {
            p_mvast: Some(40.0), p_icmsst: Some(18.0), p_fcpst: Some(2.0), mot_des_icms: Some(9),
            mot_des_icms_st: Some(3), p_red_bcst: Some(10.0), ..IcmsParametros { cst: "90".into(), ..p.clone() }
        };
        let x = xml_valido(&calc(&p90, 1.0));
        // diferimento no 90: vICMSOp 16,20, dif 8,10, vICMS 8,10
        assert!(x.contains("<cBenefRBC>SP851001</cBenefRBC><pICMS>18.0000</pICMS><vICMSOp>16.20</vICMSOp><pDif>50.0000</pDif><vICMSDif>8.10</vICMSDif><vICMS>8.10</vICMS>"), "{x}");
        assert!(x.contains("<vICMSSTDeson>") && x.contains("<motDesICMSST>3</motDesICMSST>"), "{x}");
        // cBenefRBC sem redução não sai
        let sem_red = IcmsParametros { p_red_bc: None, ..p.clone() };
        assert!(!xml_valido(&calc(&sem_red, 1.0)).contains("cBenefRBC"));
    }

    #[test]
    fn cst60_e_sn500_com_fcp_st_retido_e_icms_efetivo() {
        for c in ["60", "500"] {
            let p = IcmsParametros {
                v_bcst_ret: Some(80.0), p_st: Some(18.0), v_icmsst_ret: Some(14.4),
                p_fcpst_ret: Some(2.0), p_red_bc_efet: Some(10.0), p_icms_efet: Some(18.0), ..par(c)
            };
            let x = xml_valido(&calc(&p, 1.0));
            assert!(x.contains("<vBCFCPSTRet>80.00</vBCFCPSTRet><pFCPSTRet>2.0000</pFCPSTRet><vFCPSTRet>1.60</vFCPSTRet>"), "{c}: {x}");
            assert!(x.contains("<pRedBCEfet>10.0000</pRedBCEfet><vBCEfet>90.00</vBCEfet><pICMSEfet>18.0000</pICMSEfet><vICMSEfet>16.20</vICMSEfet>"), "{c}: {x}");
            // só o efetivo, sem redução (0 sai "0.0000", que o TDec_0302a04Opc aceita)
            let x = xml_valido(&calc(&IcmsParametros { p_icms_efet: Some(18.0), ..par(c) }, 1.0));
            assert!(x.contains("<pRedBCEfet>0.0000</pRedBCEfet><vBCEfet>100.00</vBCEfet>"), "{c}: {x}");
        }
    }

    #[test]
    fn icms_part_para_10_20_90() {
        for c in ["10", "20", "90"] {
            let p = IcmsParametros {
                p_icms: Some(12.0), p_red_bc: Some(10.0), p_mvast: Some(40.0), p_icmsst: Some(18.0),
                p_fcpst: Some(2.0), uf_st: Some("mg".into()), p_bc_op: Some(100.0), mot_des_icms: Some(9),
                ..par(c)
            };
            let x = xml_valido(&calc(&p, 1.0));
            assert!(x.contains(&format!("<ICMSPart><orig>0</orig><CST>{c}</CST>")), "{x}");
            assert!(x.contains("<pBCOp>100.0000</pBCOp><UFST>MG</UFST>"), "{x}");
        }
        let sem_pbcop = IcmsParametros { p_icms: Some(12.0), p_icmsst: Some(18.0), uf_st: Some("MG".into()), ..par("10") };
        assert!(icms_de_parametros(&sem_pbcop, 100.0, 1.0).is_err());
        let motivo_errado = IcmsParametros { p_icms: Some(12.0), p_icmsst: Some(18.0), uf_st: Some("MG".into()), p_bc_op: Some(50.0), mot_des_icms: Some(3), ..par("10") };
        assert!(icms_de_parametros(&motivo_errado, 100.0, 1.0).is_err());
    }

    #[test]
    fn icms_st_repasse_para_41_e_60() {
        for c in ["41", "60"] {
            let p = IcmsParametros {
                v_bcst_ret: Some(80.0), p_st: Some(18.0), v_icmsst_ret: Some(14.4),
                v_bcst_dest: Some(90.0), v_icmsst_dest: Some(10.8), p_fcpst_ret: Some(2.0),
                p_icms_efet: Some(18.0), ..par(c)
            };
            let x = xml_valido(&calc(&p, 1.0));
            assert!(x.contains(&format!("<ICMSST><orig>0</orig><CST>{c}</CST><vBCSTRet>80.00</vBCSTRet>")), "{x}");
            assert!(x.contains("<vBCSTDest>90.00</vBCSTDest><vICMSSTDest>10.80</vICMSSTDest>"), "{x}");
        }
        let incompleto = IcmsParametros { v_bcst_dest: Some(90.0), ..par("41") };
        assert!(icms_de_parametros(&incompleto, 100.0, 1.0).is_err());
    }

    #[test]
    fn monofasico_de_combustiveis_02_15_53_61() {
        // 250 litros × R$ 1,2200 = 305,00
        let x = xml_valido(&calc(&IcmsParametros { ad_rem_icms: Some(1.22), ..par("02") }, 250.0));
        assert!(x.contains("<ICMS02><orig>0</orig><CST>02</CST><qBCMono>250.0000</qBCMono><adRemICMS>1.2200</adRemICMS><vICMSMono>305.00</vICMSMono></ICMS02>"), "{x}");

        let x = xml_valido(&calc(&IcmsParametros { ad_rem_icms: Some(1.22), ad_rem_icms_reten: Some(0.5), p_red_ad_rem: Some(10.0), mot_red_ad_rem: Some(1), ..par("15") }, 250.0));
        assert!(x.contains("<vICMSMonoReten>125.00</vICMSMonoReten><pRedAdRem>10.00</pRedAdRem><motRedAdRem>1</motRedAdRem>"), "{x}");
        xml_valido(&calc(&IcmsParametros { ad_rem_icms: Some(1.22), ad_rem_icms_reten: Some(0.5), ..par("15") }, 250.0));

        let x = xml_valido(&calc(&IcmsParametros { ad_rem_icms: Some(1.22), p_dif: Some(40.0), ..par("53") }, 250.0));
        assert!(x.contains("<vICMSMonoOp>305.00</vICMSMonoOp><pDif>40.0000</pDif><vICMSMonoDif>122.00</vICMSMonoDif><vICMSMono>183.00</vICMSMono>"), "{x}");
        assert!(xml_valido(&calc(&par("53"), 250.0)).contains("<ICMS53><orig>0</orig><CST>53</CST></ICMS53>"));

        let x = xml_valido(&calc(&IcmsParametros { ad_rem_icms_ret: Some(1.22), ..par("61") }, 250.0));
        assert!(x.contains("<qBCMonoRet>250.0000</qBCMonoRet><adRemICMSRet>1.2200</adRemICMSRet><vICMSMonoRet>305.00</vICMSMonoRet>"), "{x}");

        for c in ["02", "15", "61"] {
            assert!(icms_de_parametros(&par(c), 100.0, 1.0).is_err(), "{c} sem ad rem");
        }
        let motivo = IcmsParametros { ad_rem_icms: Some(1.0), ad_rem_icms_reten: Some(1.0), p_red_ad_rem: Some(5.0), mot_red_ad_rem: Some(2), ..par("15") };
        assert!(icms_de_parametros(&motivo, 100.0, 1.0).is_err());
    }

    /// Todos os 25 códigos (11 CST + 4 monofásicos + 10 CSOSN) com o cadastro mais completo
    /// que o grupo aceita — tudo junto no XSD.
    #[test]
    fn bateria_todo_codigo_com_todos_os_opcionais_do_leiaute() {
        let cheio = IcmsParametros {
            p_icms: Some(18.0), p_red_bc: Some(10.0), p_fcp: Some(2.0), p_dif: Some(30.0),
            p_mvast: Some(40.0), p_red_bcst: Some(5.0), p_icmsst: Some(18.0), p_fcpst: Some(2.0),
            p_cred_sn: Some(1.5), v_bcst_ret: Some(80.0), p_st: Some(18.0), v_icmsst_ret: Some(14.4),
            v_icms_substituto: Some(10.0), c_benef_rbc: Some("SP851001".into()), p_fcp_dif: Some(50.0),
            p_red_bc_efet: Some(10.0), p_icms_efet: Some(18.0), p_fcpst_ret: Some(2.0),
            ad_rem_icms: Some(1.22), ad_rem_icms_reten: Some(0.5), p_red_ad_rem: Some(10.0),
            mot_red_ad_rem: Some(9), ad_rem_icms_ret: Some(1.22), ..Default::default()
        };
        let motivo = |c: &str| match c {
            "20" => Some(9), "30" => Some(9), "40" | "41" | "50" => Some(9), "70" | "90" => Some(12), _ => None,
        };
        for c in [
            "00", "10", "20", "30", "40", "41", "50", "51", "60", "70", "90", "02", "15", "53", "61",
            "101", "102", "103", "201", "202", "203", "300", "400", "500", "900",
        ] {
            let p = IcmsParametros {
                cst: c.into(), mot_des_icms: motivo(c),
                mot_des_icms_st: matches!(c, "10" | "70" | "90").then_some(9),
                ..cheio.clone()
            };
            xml_valido(&calc(&p, 3.0));
        }
    }
}
