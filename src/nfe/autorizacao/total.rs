use crate::nfe::autorizacao::det_process::entity::DetProcess;

use super::Total;
use anyhow::{Error, Result};
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
    ambiente: u8,
    active_ibscbs: Option<String>,
) -> Result<TotalProcess, Error> {
    let mut v_bc_ibs_cbs_total = 0.0;
    for det in &dets {
        if let Some(ibs_cbs) = det.imposto.ibs_cbs.as_ref() {
            let v_bc_ibs_cbs = ibs_cbs.g_ibscbs.v_bc.clone();
            let v_bc_ibs_cbs = v_bc_ibs_cbs.to_string().parse::<f64>().unwrap_or(0.0);
            v_bc_ibs_cbs_total += v_bc_ibs_cbs;
        }
    }

    let ibs_uf_total = v_bc_ibs_cbs_total * 0.001; // 0.1%
    let ibs_mun_total = v_bc_ibs_cbs_total * 0.000; // 0.0%
    let ibs_total = ibs_uf_total + ibs_mun_total;
    let cbs_total = v_bc_ibs_cbs_total * 0.009; // 0.9%

    let send_ibs_cbs = Some(IBSCBSTot {
        v_bc_ibs_cbs: format!("{:.2}", v_bc_ibs_cbs_total),
        g_ibs: GIBS {
            g_ibs_uf: GIBSUF {
                v_dif: Decimal::new(0, 2),
                v_dev_trib: Decimal::new(0, 2),
                v_ibs_uf: format!("{:.2}", ibs_uf_total),
            },
            g_ibs_mun: GIBSMun {
                v_dif: Decimal::new(0, 2),
                v_dev_trib: Decimal::new(0, 2),
                v_ibs_mun: format!("{:.2}", ibs_mun_total),
            },
            v_ibs: format!("{:.2}", ibs_total),
            v_cred_pres: Some(Decimal::new(0, 2)),
            v_cred_pres_cond_sus: Some(Decimal::new(0, 2)),
        },
        g_cbs: GCBS {
            v_dif: Decimal::new(0, 2),
            v_dev_trib: Decimal::new(0, 2),
            v_cbs: format!("{:.2}", cbs_total),
            v_cred_pres: Some(Decimal::new(0, 2)),
            v_cred_pres_cond_sus: Some(Decimal::new(0, 2)),
        },
    });

    let send_icms_tot = ICMSTot {
        v_bc: format!("{:.2}", total.v_bc),
        v_icms: format!("{:.2}", total.v_icms),
        v_icms_deson: format!("{:.2}", total.v_icms_deson),
        v_fcpuf_dest: format!("{:.2}", total.v_fcpuf_dest),
        v_icms_uf_dest: format!("{:.2}", total.v_icms_uf_dest),
        v_icms_uf_remet: format!("{:.2}", total.v_icms_uf_remet),
        v_fcp: format!("{:.2}", total.v_fcp),
        v_bc_st: format!("{:.2}", total.v_bc_st),
        v_st: format!("{:.2}", total.v_st),
        v_fcpst: format!("{:.2}", total.v_fcpst),
        v_fcpst_ret: format!("{:.2}", total.v_fcpst_ret),
        v_prod: format!("{:.2}", total.v_prod),
        v_frete: format!("{:.2}", total.v_frete),
        v_seg: format!("{:.2}", total.v_seg),
        v_desc: format!("{:.2}", total.v_desc),
        v_ii: format!("{:.2}", total.v_ii),
        v_ipi: format!("{:.2}", total.v_ipi),
        v_ipi_devol: format!("{:.2}", total.v_ipi_devol),
        v_pis: format!("{:.2}", total.v_pis),
        v_cofins: format!("{:.2}", total.v_cofins),
        v_outro: format!("{:.2}", total.v_outro),
        v_nf: format!("{:.2}", total.v_nf),
        v_tot_trib: format!("{:.2}", total.v_tot_trib),
    };

    // active_ibscbs onde None = ativado por padrão e Some('quarquer coisa') = desativado
    //
    // decidir se envia ou não o IBSCBS conforme ambiente e data
    // verificar se a data do sistema é maior ou igual a  que 2026-01-01
    let data_limite =
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("Data limite inválida para IBSCBS");
    let data_atual = chrono::Local::now().naive_local().date();
    if data_atual >= data_limite {
        return Ok(TotalProcess {
            icms_tot: send_icms_tot,
            ibs_cbs_tot: send_ibs_cbs,
        });
    }

    // active_ibscbs onde None = ativado por padrão e Some(false) ou Some(true) = desativado
    if let Some(_) = active_ibscbs {
        return Ok(TotalProcess {
            icms_tot: send_icms_tot,
            ibs_cbs_tot: None,
        });
    } else {
        // se for None, ativa por padrão
        return Ok(TotalProcess {
            icms_tot: send_icms_tot,
            ibs_cbs_tot: send_ibs_cbs,
        });
    }
}
