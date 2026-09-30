use super::det_process::entity::{COFINSProcess, DetProcess, ICMSProcess, PISProcess};
use crate::arredondamento::{arred2, fmt_dec};
use crate::tipos::Total;
use crate::error::Result;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "total")]
pub struct TotalProcess {
    #[serde(rename = "ICMSTot")]
    pub icms_tot: ICMSTot,
    /// Totais da NF-e com IBS e CBS
    #[serde(rename = "IBSCBSTot", skip_serializing_if = "Option::is_none")]
    pub ibs_cbs_tot: Option<IBSCBSTot>,
}

/// Totais da NF-e com IBS e CBS
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IBSCBSTot {
    /// Valor total da BC do IBS e da CBS
    #[serde(rename = "vBCIBSCBS")]
    pub v_bc_ibs_cbs: String,
    /// Grupo total do IBS
    #[serde(rename = "gIBS")]
    pub g_ibs: GIBS,
    /// Grupo total da CBS
    #[serde(rename = "gCBS")]
    pub g_cbs: GCBS,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GIBS {
    /// Grupo total do IBS da UF
    #[serde(rename = "gIBSUF")]
    pub g_ibs_uf: GIBSUF,
    /// Grupo total do IBS do Município
    #[serde(rename = "gIBSMun")]
    pub g_ibs_mun: GIBSMun,
    /// Valor total do IBS 13v2
    #[serde(rename = "vIBS")]
    pub v_ibs: String,
    /// Valor total do crédito presumido 13v2
    #[serde(rename = "vCredPres")]
    pub v_cred_pres: Decimal,
    /// Valor total do crédito presumido em condição suspensiva 13v2
    #[serde(rename = "vCredPresCondSus")]
    pub v_cred_pres_cond_sus: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GIBSUF {
    /// Valor total do diferimento 13v2
    #[serde(rename = "vDif")]
    pub v_dif: Decimal,
    /// Valor total de devolução de tributos 13v2
    #[serde(rename = "vDevTrib")]
    pub v_dev_trib: Decimal,
    /// Valor total do IBS da UF 13v2
    #[serde(rename = "vIBSUF")]
    pub v_ibs_uf: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GIBSMun {
    /// Valor total do diferimento 13v2
    #[serde(rename = "vDif")]
    pub v_dif: Decimal,
    /// Valor total de devolução de tributos 13v2
    #[serde(rename = "vDevTrib")]
    pub v_dev_trib: Decimal,
    /// Valor total do IBS do município 13v2
    #[serde(rename = "vIBSMun")]
    pub v_ibs_mun: String,
}

/// Grupo total da CBS
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GCBS {
    /// Valor total do diferimento 13v2
    #[serde(rename = "vDif")]
    pub v_dif: Decimal,
    /// Valor total de devolução de tributos 13v2
    #[serde(rename = "vDevTrib")]
    pub v_dev_trib: Decimal,
    /// Valor total da CBS 13v2
    #[serde(rename = "vCBS")]
    pub v_cbs: String,
    /// Valor total do crédito presumido 13v2
    #[serde(rename = "vCredPres")]
    pub v_cred_pres: Decimal,
    /// Valor total do crédito presumido em condição suspensiva 13v2
    #[serde(rename = "vCredPresCondSus")]
    pub v_cred_pres_cond_sus: Decimal,
}

/// ICMS Totais *************************************
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ICMSTot {
    #[serde(rename = "vBC")]
    pub v_bc: String,
    #[serde(rename = "vICMS")]
    pub v_icms: String,
    #[serde(rename = "vICMSDeson")]
    pub v_icms_deson: String,
    #[serde(rename = "vFCPUFDest")]
    pub v_fcpuf_dest: String,
    #[serde(rename = "vICMSUFDest")]
    pub v_icms_uf_dest: String,
    #[serde(rename = "vICMSUFRemet")]
    pub v_icms_uf_remet: String,
    #[serde(rename = "vFCP")]
    pub v_fcp: String,
    #[serde(rename = "vBCST")]
    pub v_bc_st: String,
    #[serde(rename = "vST")]
    pub v_st: String,
    // vFCPST
    #[serde(rename = "vFCPST")]
    pub v_fcpst: String,
    #[serde(rename = "vFCPSTRet")]
    pub v_fcpst_ret: String,
    // Monofásico de combustíveis (NT 2023.001): só saem quando algum item é 02/15/53/61.
    #[serde(rename = "qBCMono", skip_serializing_if = "Option::is_none")]
    pub q_bc_mono: Option<String>,
    #[serde(rename = "vICMSMono", skip_serializing_if = "Option::is_none")]
    pub v_icms_mono: Option<String>,
    #[serde(rename = "qBCMonoReten", skip_serializing_if = "Option::is_none")]
    pub q_bc_mono_reten: Option<String>,
    #[serde(rename = "vICMSMonoReten", skip_serializing_if = "Option::is_none")]
    pub v_icms_mono_reten: Option<String>,
    #[serde(rename = "qBCMonoRet", skip_serializing_if = "Option::is_none")]
    pub q_bc_mono_ret: Option<String>,
    #[serde(rename = "vICMSMonoRet", skip_serializing_if = "Option::is_none")]
    pub v_icms_mono_ret: Option<String>,
    #[serde(rename = "vProd")]
    pub v_prod: String,
    #[serde(rename = "vFrete")]
    pub v_frete: String,
    #[serde(rename = "vSeg")]
    pub v_seg: String,
    #[serde(rename = "vDesc")]
    pub v_desc: String,
    #[serde(rename = "vII")]
    pub v_ii: String,
    #[serde(rename = "vIPI")]
    pub v_ipi: String,
    // vIPIDevol
    #[serde(rename = "vIPIDevol")]
    pub v_ipi_devol: String,
    #[serde(rename = "vPIS")]
    pub v_pis: String,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: String,
    #[serde(rename = "vOutro")]
    pub v_outro: String,
    #[serde(rename = "vNF")]
    pub v_nf: String,
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: String,
}

pub fn total_process(
    total: Total,
    dets: Vec<DetProcess>,
    _ambiente: u8,
    _active_ibscbs: Option<String>,
) -> Result<TotalProcess> {
    // ── Totais calculados dos itens ───────────────────────────────────────────
    let mut v_bc          = 0.0_f64;
    let mut v_icms        = 0.0_f64;
    let mut v_icms_deson  = 0.0_f64;
    let mut v_bc_st_items = 0.0_f64;
    let mut v_st_items    = 0.0_f64;
    let mut v_fcp_items   = 0.0_f64;
    let mut v_fcpst_items = 0.0_f64;
    // vICMSDeson dos itens com indDeduzDeson = 1: sai do vNF (NT 2023.004)
    let mut v_deson_deduzido = 0.0_f64;
    let mut v_ipi_items   = 0.0_f64;
    let mut v_prod        = 0.0_f64;
    let mut v_desc        = Decimal::ZERO;
    let mut v_frete_items = Decimal::ZERO;
    let mut v_outro_items = Decimal::ZERO;
    let mut v_pis         = 0.0_f64;
    let mut v_cofins      = 0.0_f64;
    let mut v_tot_trib    = 0.0_f64;
    let mut v_fcpst_ret_items = 0.0_f64;
    // [qBCMono, vICMSMono, qBCMonoReten, vICMSMonoReten, qBCMonoRet, vICMSMonoRet]
    let mut mono: Option<[f64; 6]> = None;
    // PIS-ST/COFINS-ST com indSoma = 1 (NT 2020.005): entram no vNF.
    let mut v_pis_cofins_st_vnf = 0.0_f64;

    // ── Totais IBS/CBS ────────────────────────────────────────────────────────
    let mut v_bc_ibs_cbs_total    = 0.0_f64;
    let mut ibs_uf_total          = 0.0_f64;
    let mut ibs_uf_dif_total      = 0.0_f64;
    let mut ibs_uf_dev_trib_total = 0.0_f64;
    let mut ibs_mun_total         = 0.0_f64;
    let mut ibs_mun_dif_total     = 0.0_f64;
    let mut ibs_mun_dev_trib_total = 0.0_f64;
    let mut ibs_total             = 0.0_f64;
    let mut cbs_total             = 0.0_f64;
    let mut cbs_dif_total         = 0.0_f64;
    let mut cbs_dev_trib_total    = 0.0_f64;

    // Cada parcela é o valor IMPRESSO no item (arred2 = mesmo centavo do fmt_dec do item):
    // o total tem de ser exatamente Σ itens do XML (rejeições 531/532/602/603).
    for det in &dets {
        v_bc           += arred2(icms_v_bc(&det.imposto.icms));
        v_icms         += arred2(icms_v_icms(&det.imposto.icms));
        v_icms_deson   += arred2(icms_v_deson(&det.imposto.icms));
        v_bc_st_items  += arred2(icms_v_bcst(&det.imposto.icms));
        v_st_items     += arred2(icms_v_icmsst(&det.imposto.icms));
        v_fcp_items    += arred2(icms_v_fcp(&det.imposto.icms));
        v_fcpst_items  += arred2(icms_v_fcpst(&det.imposto.icms));
        if icms_deduz_deson(&det.imposto.icms) {
            v_deson_deduzido += arred2(icms_v_deson(&det.imposto.icms));
        }
        v_ipi_items    += arred2(ipi_v_ipi(&det.imposto.ipi));
        v_fcpst_ret_items += arred2(icms_v_fcpst_ret(&det.imposto.icms));
        if let Some(m) = icms_mono(&det.imposto.icms) {
            let acc = mono.get_or_insert([0.0; 6]);
            for (a, v) in acc.iter_mut().zip(m) {
                *a += v;
            }
        }
        v_prod       += det.prod.v_prod.parse::<f64>().unwrap_or(0.0);
        v_desc       += det.prod.v_desc.unwrap_or(Decimal::ZERO);
        v_frete_items += det.prod.v_frete.unwrap_or(Decimal::ZERO);
        v_outro_items += det.prod.v_outro.unwrap_or(Decimal::ZERO);
        v_pis        += arred2(pis_v_pis(&det.imposto.pis));
        v_cofins     += arred2(cofins_v_cofins(&det.imposto.cofins));
        v_pis_cofins_st_vnf += arred2(pis_st_soma_no_vnf(&det.imposto.pis))
            + arred2(cofins_st_soma_no_vnf(&det.imposto.cofins));
        v_tot_trib   += det.imposto.v_tot_trib.parse::<f64>().unwrap_or(0.0);

        if let Some(ibs_cbs) = det.imposto.ibs_cbs.as_ref() {
            v_bc_ibs_cbs_total += ibs_cbs.g_ibscbs.v_bc.parse::<f64>().unwrap_or(0.0);

            let v_ibs_uf = ibs_cbs.g_ibscbs.g_ibs_uf.v_ibs_uf.parse::<f64>().unwrap_or(0.0);
            ibs_uf_total += v_ibs_uf;
            if let Some(ref g) = ibs_cbs.g_ibscbs.g_ibs_uf.g_dif {
                ibs_uf_dif_total += g.v_dif.to_f64().unwrap_or(0.0);
            }
            if let Some(ref g) = ibs_cbs.g_ibscbs.g_ibs_uf.g_dev_trib {
                ibs_uf_dev_trib_total += g.v_dev_trib.to_f64().unwrap_or(0.0);
            }

            let v_ibs_mun = ibs_cbs.g_ibscbs.g_ibs_mun.v_ibs_mun.parse::<f64>().unwrap_or(0.0);
            ibs_mun_total += v_ibs_mun;
            if let Some(ref g) = ibs_cbs.g_ibscbs.g_ibs_mun.g_dif {
                ibs_mun_dif_total += g.v_dif.to_f64().unwrap_or(0.0);
            }
            if let Some(ref g) = ibs_cbs.g_ibscbs.g_ibs_mun.g_dev_trib {
                ibs_mun_dev_trib_total += g.v_dev_trib.to_f64().unwrap_or(0.0);
            }

            ibs_total += ibs_cbs.g_ibscbs.v_ibs.parse::<f64>().unwrap_or(0.0);
            cbs_total += ibs_cbs.g_ibscbs.g_cbs.v_cbs.parse::<f64>().unwrap_or(0.0);
            if let Some(ref g) = ibs_cbs.g_ibscbs.g_cbs.g_dif {
                cbs_dif_total += g.v_dif.to_f64().unwrap_or(0.0);
            }
            if let Some(ref g) = ibs_cbs.g_ibscbs.g_cbs.g_dev_trib {
                cbs_dev_trib_total += g.v_dev_trib.to_f64().unwrap_or(0.0);
            }
        }
    }

    let v_desc_f64 = v_desc.to_f64().unwrap_or(0.0);
    // vFrete efetivo: quando há frete rateado por item (det/prod/vFrete), o total DEVE ser
    // a soma dos itens (SEFAZ: ICMSTot/vFrete == Σ det/prod/vFrete). Sem rateio por item,
    // mantém o `total.v_frete` global informado pelo chamador (comportamento anterior — aditivo).
    let v_frete_items_f64 = v_frete_items.to_f64().unwrap_or(0.0);
    let v_frete_efetivo = if v_frete_items_f64 > 0.0 { v_frete_items_f64 } else { total.v_frete };
    // v_bc_st e v_st: auto-calculado dos itens + valor global informado em Total
    let total_v_bc_st = v_bc_st_items + total.v_bc_st;
    let total_v_st    = v_st_items    + total.v_st;
    // v_ipi: auto-calculado dos itens + valor global informado em Total
    let total_v_ipi = v_ipi_items + total.v_ipi;
    // vOutro efetivo: mesmo critério do frete — com acréscimo rateado por item
    // (det/prod/vOutro) o total é a soma dos itens; sem rateio, vale o `total.v_outro` global.
    let v_outro_items_f64 = v_outro_items.to_f64().unwrap_or(0.0);
    let v_outro_efetivo = if v_outro_items_f64 > 0.0 { v_outro_items_f64 } else { total.v_outro };
    let total_v_fcp   = v_fcp_items + total.v_fcp;
    let total_v_fcpst = v_fcpst_items + total.v_fcpst;
    // Regra 610 da SEFAZ: o ICMS-ST e o FCP-ST entram no total da nota, e o desonerado com
    // indDeduzDeson = 1 sai. Antes nenhum item chegava com ST (os consumidores achatavam
    // CST 10/30/70 em 00/90), então a falta das parcelas não aparecia.
    let v_nf = v_prod + v_frete_efetivo + total.v_seg - v_desc_f64
               + v_outro_efetivo + total.v_ii + total_v_ipi - total.v_ipi_devol
               + total_v_st + total_v_fcpst - v_deson_deduzido
               + v_pis_cofins_st_vnf
               // ICMS monofásico sujeito a retenção (CST 15) entra no vNF (NT 2023.001, 610).
               + mono.map(|m| arred2(m[3])).unwrap_or(0.0);

    // Só envia IBSCBSTot se algum item tiver IBS/CBS — enviar zerado causa rejeição 1118
    let send_ibs_cbs = if v_bc_ibs_cbs_total > 0.0 {
        Some(IBSCBSTot {
            v_bc_ibs_cbs: fmt_dec(v_bc_ibs_cbs_total, 2),
            g_ibs: GIBS {
                g_ibs_uf: GIBSUF {
                    v_dif: Decimal::from_str_exact(&fmt_dec(ibs_uf_dif_total, 2))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_dev_trib: Decimal::from_str_exact(&fmt_dec(ibs_uf_dev_trib_total, 2))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_ibs_uf: fmt_dec(ibs_uf_total, 2),
                },
                g_ibs_mun: GIBSMun {
                    v_dif: Decimal::from_str_exact(&fmt_dec(ibs_mun_dif_total, 2))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_dev_trib: Decimal::from_str_exact(&fmt_dec(ibs_mun_dev_trib_total, 2))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_ibs_mun: fmt_dec(ibs_mun_total, 2),
                },
                v_ibs: fmt_dec(ibs_total, 2),
                // Obrigatórios no XSD; a crate não monta gCredPres no item, então o total é 0.00
                // (`None` saía `<vCredPres/>` e reprovava no pattern).
                v_cred_pres: Decimal::new(0, 2),
                v_cred_pres_cond_sus: Decimal::new(0, 2),
            },
            g_cbs: GCBS {
                v_dif: Decimal::from_str_exact(&fmt_dec(cbs_dif_total, 2))
                    .unwrap_or(Decimal::new(0, 2)),
                v_dev_trib: Decimal::from_str_exact(&fmt_dec(cbs_dev_trib_total, 2))
                    .unwrap_or(Decimal::new(0, 2)),
                v_cbs: fmt_dec(cbs_total, 2),
                v_cred_pres: Decimal::new(0, 2),
                v_cred_pres_cond_sus: Decimal::new(0, 2),
            },
        })
    } else {
        None
    };

    let send_icms_tot = ICMSTot {
        v_bc:           fmt_dec(v_bc, 2),
        v_icms:         fmt_dec(v_icms, 2),
        v_icms_deson:   fmt_dec(v_icms_deson, 2),
        v_fcpuf_dest:   fmt_dec(total.v_fcpuf_dest, 2),
        v_icms_uf_dest: fmt_dec(total.v_icms_uf_dest, 2),
        v_icms_uf_remet:fmt_dec(total.v_icms_uf_remet, 2),
        v_fcp:          fmt_dec(total_v_fcp, 2),
        v_bc_st:        fmt_dec(total_v_bc_st, 2),
        v_st:           fmt_dec(total_v_st, 2),
        v_fcpst:        fmt_dec(total_v_fcpst, 2),
        v_fcpst_ret:    fmt_dec(v_fcpst_ret_items + total.v_fcpst_ret, 2),
        q_bc_mono:         mono.map(|m| fmt_dec(m[0], 4)),
        v_icms_mono:       mono.map(|m| fmt_dec(m[1], 2)),
        q_bc_mono_reten:   mono.map(|m| fmt_dec(m[2], 4)),
        v_icms_mono_reten: mono.map(|m| fmt_dec(m[3], 2)),
        q_bc_mono_ret:     mono.map(|m| fmt_dec(m[4], 4)),
        v_icms_mono_ret:   mono.map(|m| fmt_dec(m[5], 2)),
        v_prod:         fmt_dec(v_prod, 2),
        v_frete:        fmt_dec(v_frete_efetivo, 2),
        v_seg:          fmt_dec(total.v_seg, 2),
        v_desc:         fmt_dec(v_desc_f64, 2),
        v_ii:           fmt_dec(total.v_ii, 2),
        v_ipi:          fmt_dec(total_v_ipi, 2),
        v_ipi_devol:    fmt_dec(total.v_ipi_devol, 2),
        v_pis:          fmt_dec(v_pis, 2),
        v_cofins:       fmt_dec(v_cofins, 2),
        v_outro:        fmt_dec(v_outro_efetivo, 2),
        v_nf:           fmt_dec(v_nf, 2),
        v_tot_trib:     fmt_dec(v_tot_trib, 2),
    };

    Ok(TotalProcess {
        icms_tot: send_icms_tot,
        ibs_cbs_tot: send_ibs_cbs,
    })
}

// ── Extratores de valores dos itens ──────────────────────────────────────────

use super::det_process::entity::IpiProcess;

fn icms_v_bc(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS00(v) => v.v_bc,
        ICMSProcess::ICMS10(v) => v.v_bc,
        ICMSProcess::ICMS20(v) => v.v_bc,
        ICMSProcess::ICMS51(v) => v.v_bc.unwrap_or(0.0),
        ICMSProcess::ICMS70(v) => v.v_bc,
        ICMSProcess::ICMS90(v) => v.v_bc.unwrap_or(0.0),
        ICMSProcess::ICMSPart(v) => v.v_bc,
        ICMSProcess::ICMSSN900(v) => v.vbc.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        _ => 0.0,
    }
}

pub(super) fn icms_v_icms(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS00(v) => v.v_icms,
        ICMSProcess::ICMS10(v) => v.v_icms,
        ICMSProcess::ICMS20(v) => v.v_icms,
        ICMSProcess::ICMS51(v) => v.v_icms.unwrap_or(0.0),
        ICMSProcess::ICMS70(v) => v.v_icms,
        ICMSProcess::ICMS90(v) => v.v_icms.unwrap_or(0.0),
        ICMSProcess::ICMSPart(v) => v.v_icms,
        ICMSProcess::ICMSSN900(v) => v.vicms.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        _ => 0.0,
    }
}

fn icms_v_deson(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS40(v)  => v.vicmsdeson.unwrap_or(0.0),
        ICMSProcess::ICMS20(v)  => v.v_icms_deson.unwrap_or(0.0),
        ICMSProcess::ICMS30(v)  => v.v_icms_deson.unwrap_or(0.0),
        ICMSProcess::ICMS70(v)  => v.v_icms_deson.unwrap_or(0.0),
        ICMSProcess::ICMS90(v)  => v.v_icms_deson.unwrap_or(0.0),
        ICMSProcess::ICMSPart(v) => v.v_icms_deson.unwrap_or(0.0),
        _ => 0.0,
    }
}

fn icms_v_bcst(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS10(v) => v.v_bcst,
        ICMSProcess::ICMS30(v) => v.v_bcst,
        ICMSProcess::ICMS70(v) => v.v_bcst,
        ICMSProcess::ICMS90(v) => v.v_bcst.unwrap_or(0.0),
        ICMSProcess::ICMSPart(v) => v.v_bcst,
        ICMSProcess::ICMSSN201(v) => v.v_bcst,
        ICMSProcess::ICMSSN202(v) => v.v_bcst,
        ICMSProcess::ICMSSN900(v) => v.vbcst.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        _ => 0.0,
    }
}

fn icms_v_icmsst(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS10(v) => v.v_icmsst,
        ICMSProcess::ICMS30(v) => v.v_icmsst,
        ICMSProcess::ICMS70(v) => v.v_icmsst,
        ICMSProcess::ICMS90(v) => v.v_icmsst.unwrap_or(0.0),
        ICMSProcess::ICMSPart(v) => v.v_icmsst,
        ICMSProcess::ICMSSN201(v) => v.v_icmsst,
        ICMSProcess::ICMSSN202(v) => v.v_icmsst,
        ICMSProcess::ICMSSN900(v) => v.vicmsst.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0),
        _ => 0.0,
    }
}

pub(super) fn icms_v_fcp(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS00(v) => v.v_fcp,
        ICMSProcess::ICMS10(v) => v.v_fcp,
        ICMSProcess::ICMS20(v) => v.v_fcp,
        ICMSProcess::ICMS51(v) => v.v_fcp,
        ICMSProcess::ICMS70(v) => v.v_fcp,
        ICMSProcess::ICMS90(v) => v.v_fcp,
        _ => None,
    }
    .unwrap_or(0.0)
}

fn icms_v_fcpst(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS10(v) => v.v_fcpst,
        ICMSProcess::ICMS30(v) => v.v_fcpst,
        ICMSProcess::ICMS70(v) => v.v_fcpst,
        ICMSProcess::ICMS90(v) => v.v_fcpst,
        ICMSProcess::ICMSPart(v) => v.v_fcpst,
        ICMSProcess::ICMSSN201(v) => v.v_fcpst,
        ICMSProcess::ICMSSN202(v) => v.v_fcpst,
        ICMSProcess::ICMSSN900(v) => v.vfcpst.as_deref().and_then(|s| s.parse().ok()),
        _ => None,
    }
    .unwrap_or(0.0)
}

fn icms_deduz_deson(icms: &ICMSProcess) -> bool {
    let ind = match icms {
        ICMSProcess::ICMS20(v) => v.ind_deduz_deson,
        ICMSProcess::ICMS30(v) => v.ind_deduz_deson,
        ICMSProcess::ICMS40(v) => v.ind_deduz_deson,
        ICMSProcess::ICMS70(v) => v.ind_deduz_deson,
        ICMSProcess::ICMS90(v) => v.ind_deduz_deson,
        ICMSProcess::ICMSPart(v) => v.ind_deduz_deson,
        _ => None,
    };
    ind == Some(1)
}

/// vFCPSTRet do item (CST 60, CSOSN 500, ICMSST).
fn icms_v_fcpst_ret(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS60(v) => v.v_fcpst_ret,
        ICMSProcess::ICMSSN500(v) => v.v_fcpst_ret,
        ICMSProcess::ICMSST(v) => v.v_fcpst_ret,
        _ => None,
    }
    .unwrap_or(0.0)
}

/// Monofásico de combustíveis (NT 2023.001): (qBCMono, vICMSMono, qBCMonoReten,
/// vICMSMonoReten, qBCMonoRet, vICMSMonoRet) do item.
fn icms_mono(icms: &ICMSProcess) -> Option<[f64; 6]> {
    Some(match icms {
        ICMSProcess::ICMS02(v) => [v.q_bc_mono.unwrap_or(0.0), v.v_icms_mono, 0.0, 0.0, 0.0, 0.0],
        ICMSProcess::ICMS15(v) => [
            v.q_bc_mono.unwrap_or(0.0), v.v_icms_mono,
            v.q_bc_mono_reten.unwrap_or(0.0), v.v_icms_mono_reten, 0.0, 0.0,
        ],
        ICMSProcess::ICMS53(v) => [v.q_bc_mono.unwrap_or(0.0), v.v_icms_mono.unwrap_or(0.0), 0.0, 0.0, 0.0, 0.0],
        ICMSProcess::ICMS61(v) => [0.0, 0.0, 0.0, 0.0, v.q_bc_mono_ret.unwrap_or(0.0), v.v_icms_mono_ret],
        _ => return None,
    })
}

fn ipi_v_ipi(ipi: &Option<IpiProcess>) -> f64 {
    ipi.as_ref().filter(|p| p.tributado)
        // o XML inner do IPITrib contém vIPI serializado
        .and_then(|p| {
            let xml = &p.inner;
            let start = xml.find("<vIPI>")? + 6;
            let end   = xml.find("</vIPI>")?;
            xml[start..end].parse().ok()
        })
        .unwrap_or(0.0)
}

/// vPIS do grupo PIS do item (Aliq, Qtde ou Outr). O PISST é outro grupo: não entra no
/// ICMSTot/vPIS — só no vNF, e só com indSomaPISST = 1 (ver [`pis_st_soma_no_vnf`]).
pub(super) fn pis_v_pis(pis: &PISProcess) -> f64 {
    let num = |s: &Option<String>| s.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    if let Some(v) = &pis.pis_aliq  { return v.v_pis; }
    if let Some(v) = &pis.pis_qtde  { return v.vpis.parse().unwrap_or(0.0); }
    if let Some(v) = &pis.pis_outr  { return num(&v.vpis); }
    0.0
}

/// vPIS do PISST quando indSomaPISST = 1 (NT 2020.005, regra 610).
pub(super) fn pis_st_soma_no_vnf(pis: &PISProcess) -> f64 {
    pis.pis_st.as_ref()
        .filter(|v| v.ind_soma.as_deref() == Some("1"))
        .and_then(|v| v.vpis.as_deref().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

/// vCOFINS do COFINSST quando indSomaCOFINSST = 1.
pub(super) fn cofins_st_soma_no_vnf(cofins: &COFINSProcess) -> f64 {
    cofins.cofins_st.as_ref()
        .filter(|v| v.ind_soma.as_deref() == Some("1"))
        .and_then(|v| v.vcofins.as_deref().and_then(|s| s.parse().ok()))
        .unwrap_or(0.0)
}

/// vCOFINS do grupo COFINS do item — mesma regra de [`pis_v_pis`].
pub(super) fn cofins_v_cofins(cofins: &COFINSProcess) -> f64 {
    if let Some(v) = &cofins.cofins_aliq { return v.v_cofins; }
    if let Some(v) = &cofins.cofins_qtde { return v.vcofins.parse().unwrap_or(0.0); }
    if let Some(v) = &cofins.cofins_outr { return v.v_cofins.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0); }
    0.0
}
