use super::det_process::entity::{COFINSProcess, DetProcess, ICMSProcess, PISProcess};
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
    pub v_cred_pres: Option<Decimal>,
    /// Valor total do crédito presumido em condição suspensiva 13v2
    #[serde(rename = "vCredPresCondSus")]
    pub v_cred_pres_cond_sus: Option<Decimal>,
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
    pub v_cred_pres: Option<Decimal>,
    /// Valor total do crédito presumido em condição suspensiva 13v2
    #[serde(rename = "vCredPresCondSus")]
    pub v_cred_pres_cond_sus: Option<Decimal>,
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
    let mut v_prod        = 0.0_f64;
    let mut v_desc        = Decimal::ZERO;
    let mut v_pis         = 0.0_f64;
    let mut v_cofins      = 0.0_f64;
    let mut v_tot_trib    = 0.0_f64;

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

    for det in &dets {
        v_bc         += icms_v_bc(&det.imposto.icms);
        v_icms       += icms_v_icms(&det.imposto.icms);
        v_icms_deson += icms_v_deson(&det.imposto.icms);
        v_prod       += det.prod.v_prod.parse::<f64>().unwrap_or(0.0);
        v_desc       += det.prod.v_desc.unwrap_or(Decimal::ZERO);
        v_pis        += pis_v_pis(&det.imposto.pis);
        v_cofins     += cofins_v_cofins(&det.imposto.cofins);
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
    let v_nf = v_prod + total.v_frete + total.v_seg - v_desc_f64
               + total.v_outro + total.v_ii + total.v_ipi - total.v_ipi_devol;

    // Só envia IBSCBSTot se algum item tiver IBS/CBS — enviar zerado causa rejeição 1118
    let send_ibs_cbs = if v_bc_ibs_cbs_total > 0.0 {
        Some(IBSCBSTot {
            v_bc_ibs_cbs: format!("{:.2}", v_bc_ibs_cbs_total),
            g_ibs: GIBS {
                g_ibs_uf: GIBSUF {
                    v_dif: Decimal::from_str_exact(&format!("{:.2}", ibs_uf_dif_total))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_dev_trib: Decimal::from_str_exact(&format!("{:.2}", ibs_uf_dev_trib_total))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_ibs_uf: format!("{:.2}", ibs_uf_total),
                },
                g_ibs_mun: GIBSMun {
                    v_dif: Decimal::from_str_exact(&format!("{:.2}", ibs_mun_dif_total))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_dev_trib: Decimal::from_str_exact(&format!("{:.2}", ibs_mun_dev_trib_total))
                        .unwrap_or(Decimal::new(0, 2)),
                    v_ibs_mun: format!("{:.2}", ibs_mun_total),
                },
                v_ibs: format!("{:.2}", ibs_total),
                v_cred_pres: None,
                v_cred_pres_cond_sus: None,
            },
            g_cbs: GCBS {
                v_dif: Decimal::from_str_exact(&format!("{:.2}", cbs_dif_total))
                    .unwrap_or(Decimal::new(0, 2)),
                v_dev_trib: Decimal::from_str_exact(&format!("{:.2}", cbs_dev_trib_total))
                    .unwrap_or(Decimal::new(0, 2)),
                v_cbs: format!("{:.2}", cbs_total),
                v_cred_pres: None,
                v_cred_pres_cond_sus: None,
            },
        })
    } else {
        None
    };

    let send_icms_tot = ICMSTot {
        v_bc:           format!("{:.2}", v_bc),
        v_icms:         format!("{:.2}", v_icms),
        v_icms_deson:   format!("{:.2}", v_icms_deson),
        v_fcpuf_dest:   format!("{:.2}", total.v_fcpuf_dest),
        v_icms_uf_dest: format!("{:.2}", total.v_icms_uf_dest),
        v_icms_uf_remet:format!("{:.2}", total.v_icms_uf_remet),
        v_fcp:          format!("{:.2}", total.v_fcp),
        v_bc_st:        format!("{:.2}", total.v_bc_st),
        v_st:           format!("{:.2}", total.v_st),
        v_fcpst:        format!("{:.2}", total.v_fcpst),
        v_fcpst_ret:    format!("{:.2}", total.v_fcpst_ret),
        v_prod:         format!("{:.2}", v_prod),
        v_frete:        format!("{:.2}", total.v_frete),
        v_seg:          format!("{:.2}", total.v_seg),
        v_desc:         format!("{:.2}", v_desc_f64),
        v_ii:           format!("{:.2}", total.v_ii),
        v_ipi:          format!("{:.2}", total.v_ipi),
        v_ipi_devol:    format!("{:.2}", total.v_ipi_devol),
        v_pis:          format!("{:.2}", v_pis),
        v_cofins:       format!("{:.2}", v_cofins),
        v_outro:        format!("{:.2}", total.v_outro),
        v_nf:           format!("{:.2}", v_nf),
        v_tot_trib:     format!("{:.2}", v_tot_trib),
    };

    Ok(TotalProcess {
        icms_tot: send_icms_tot,
        ibs_cbs_tot: send_ibs_cbs,
    })
}

// ── Extratores de valores dos itens ──────────────────────────────────────────

fn icms_v_bc(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS00(v) => v.v_bc,
        _ => 0.0,
    }
}

fn icms_v_icms(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS00(v) => v.v_icms,
        _ => 0.0,
    }
}

fn icms_v_deson(icms: &ICMSProcess) -> f64 {
    match icms {
        ICMSProcess::ICMS40(v) => v.vicmsdeson.unwrap_or(0.0),
        _ => 0.0,
    }
}

fn pis_v_pis(pis: &PISProcess) -> f64 {
    if let Some(v) = &pis.pis_aliq  { return v.v_pis; }
    if let Some(v) = &pis.pis_qtde  { return v.vpis.parse().unwrap_or(0.0); }
    0.0
}

fn cofins_v_cofins(cofins: &COFINSProcess) -> f64 {
    if let Some(v) = &cofins.cofins_aliq { return v.v_cofins; }
    if let Some(v) = &cofins.cofins_qtde { return v.vcofins.parse().unwrap_or(0.0); }
    0.0
}
