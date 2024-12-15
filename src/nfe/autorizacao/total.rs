use super::Total;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "total")]
pub struct TotalProcess {
    #[serde(rename = "ICMSTot")]
    pub icms_tot: ICMSTot,
}

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

pub fn total_process(total: Total) -> Result<TotalProcess, Error> {
    let total_process = TotalProcess {
        icms_tot: ICMSTot {
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
        },
    };

    Ok(total_process)
}
