//! Structs de item (`det`) do XML de emissão. A7 (§2.1): as famílias de imposto foram movidas
//! para submódulos temáticos (icms/pis_cofins/ibscbs/ipi); aqui ficam `ProdProcess`,
//! `ImpostoProcess` (+ montagem `to_xml`), `DetProcess` e os helpers de serialização.

mod ibscbs;
mod icms;
mod ipi;
mod pis_cofins;

pub use ibscbs::*;
pub use icms::*;
pub use ipi::*;
pub use pis_cofins::*;

use rust_decimal::Decimal;
use crate::arredondamento::fmt_dec;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "prod")]
pub struct ProdProcess {
    #[serde(rename = "cProd")]
    pub c_prod: String,
    #[serde(rename = "cEAN")]
    pub c_ean: String,
    #[serde(rename = "xProd")]
    pub x_prod: String,
    #[serde(rename = "NCM")]
    pub ncm: String,
    #[serde(rename = "CEST", skip_serializing_if = "Option::is_none")]
    pub cest: Option<String>,
    /// Código de benefício fiscal na UF. No XSD vem depois de CEST/indEscala/CNPJFab e
    /// antes de gCred/EXTIPI/CFOP.
    #[serde(rename = "cBenef", skip_serializing_if = "Option::is_none")]
    pub c_benef: Option<String>,
    /// Crédito presumido (até 4) — entre cBenef e EXTIPI/CFOP.
    #[serde(rename = "gCred", skip_serializing_if = "Vec::is_empty", default)]
    pub g_cred: Vec<GCredProcess>,
    #[serde(rename = "CFOP")]
    pub cfop: String,
    #[serde(rename = "uCom")]
    pub u_com: String,
    #[serde(rename = "qCom")]
    pub q_com: String,
    #[serde(rename = "vUnCom")]
    pub v_un_com: String,
    #[serde(rename = "vProd")]
    pub v_prod: String,
    #[serde(rename = "cEANTrib")]
    pub c_ean_trib: String,
    #[serde(rename = "uTrib")]
    pub u_trib: String,
    #[serde(rename = "qTrib")]
    pub q_trib: String,
    #[serde(rename = "vUnTrib")]
    pub v_un_trib: String,
    // vFrete vem ANTES de vDesc na ordem do XSD (prod). Só emitido quando há frete rateado
    // (skip_serializing_if) → item sem frete produz XML idêntico ao anterior (mudança aditiva).
    #[serde(rename = "vFrete", skip_serializing_if = "Option::is_none")]
    pub v_frete: Option<Decimal>,
    #[serde(rename = "vDesc", skip_serializing_if = "Option::is_none")]
    pub v_desc: Option<Decimal>,
    // vOutro vem logo DEPOIS de vDesc e antes de indTot (ordem do XSD). Só emitido quando há
    // acréscimo rateado → item sem acréscimo produz XML idêntico ao anterior (mudança aditiva).
    #[serde(rename = "vOutro", skip_serializing_if = "Option::is_none")]
    pub v_outro: Option<Decimal>,
    #[serde(rename = "indTot")]
    pub ind_tot: String,
    #[serde(rename = "xPed", skip_serializing_if = "Option::is_none")]
    pub x_ped: Option<String>,
    #[serde(rename = "nItemPed", skip_serializing_if = "Option::is_none")]
    pub n_item_ped: Option<String>,
    /// Combustível — último elemento do <prod> (choice veicProd/med/arma/comb/nRECOPI).
    #[serde(rename = "comb", skip_serializing_if = "Option::is_none", default)]
    pub comb: Option<CombProcess>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GCredProcess {
    #[serde(rename = "cCredPresumido")]
    pub c_cred_presumido: String,
    #[serde(rename = "pCredPresumido")]
    pub p_cred_presumido: String,
    #[serde(rename = "vCredPresumido")]
    pub v_cred_presumido: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CombProcess {
    #[serde(rename = "cProdANP")]
    pub c_prod_anp: String,
    #[serde(rename = "descANP")]
    pub desc_anp: String,
    #[serde(rename = "pGLP", skip_serializing_if = "Option::is_none")]
    pub p_glp: Option<String>,
    #[serde(rename = "pGNn", skip_serializing_if = "Option::is_none")]
    pub p_gnn: Option<String>,
    #[serde(rename = "pGNi", skip_serializing_if = "Option::is_none")]
    pub p_gni: Option<String>,
    #[serde(rename = "vPart", skip_serializing_if = "Option::is_none")]
    pub v_part: Option<String>,
    #[serde(rename = "CODIF", skip_serializing_if = "Option::is_none")]
    pub codif: Option<String>,
    #[serde(rename = "qTemp", skip_serializing_if = "Option::is_none")]
    pub q_temp: Option<String>,
    #[serde(rename = "UFCons")]
    pub uf_cons: String,
    #[serde(rename = "CIDE", skip_serializing_if = "Option::is_none")]
    pub cide: Option<CideProcess>,
    #[serde(rename = "encerrante", skip_serializing_if = "Option::is_none")]
    pub encerrante: Option<EncerranteProcess>,
    #[serde(rename = "pBio", skip_serializing_if = "Option::is_none")]
    pub p_bio: Option<String>,
    #[serde(rename = "origComb", skip_serializing_if = "Vec::is_empty", default)]
    pub orig_comb: Vec<OrigCombProcess>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CideProcess {
    #[serde(rename = "qBCProd")]
    pub q_bc_prod: String,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: String,
    #[serde(rename = "vCIDE")]
    pub v_cide: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncerranteProcess {
    #[serde(rename = "nBico")]
    pub n_bico: String,
    #[serde(rename = "nBomba", skip_serializing_if = "Option::is_none")]
    pub n_bomba: Option<String>,
    #[serde(rename = "nTanque")]
    pub n_tanque: String,
    #[serde(rename = "vEncIni")]
    pub v_enc_ini: String,
    #[serde(rename = "vEncFin")]
    pub v_enc_fin: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrigCombProcess {
    #[serde(rename = "indImport")]
    pub ind_import: String,
    #[serde(rename = "cUFOrig")]
    pub c_uf_orig: String,
    #[serde(rename = "pOrig")]
    pub p_orig: String,
}


// ─── ImpostoProcess ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "imposto")]
pub struct ImpostoProcess {
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: String,
    #[serde(rename = "ICMS")]
    pub icms: ICMSProcess,
    #[serde(skip)]
    pub ipi: Option<IpiProcess>,
    #[serde(rename = "PIS")]
    pub pis: PISProcess,
    #[serde(rename = "COFINS")]
    pub cofins: COFINSProcess,
    #[serde(rename = "IBSCBS", skip_serializing_if = "Option::is_none")]
    pub ibs_cbs: Option<IBSCBSProcess>,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "det")]
pub struct DetProcess {
    #[serde(rename = "prod")]
    pub prod: ProdProcess,
    #[serde(rename = "imposto")]
    pub imposto: ImpostoProcess,
    #[serde(rename = "infAdProd", skip_serializing_if = "Option::is_none")]
    pub inf_ad_prod: Option<String>,
}


// ─── Serialização manual do imposto ──────────────────────────────────────────
//
// quick_xml não suporta enum externamente tagueado como campo de struct.
// Cada variante é serializada diretamente no struct concreto (que quick_xml
// serializa corretamente), e as tags <ICMS>, <PIS>, <COFINS> são montadas aqui.

impl ImpostoProcess {
    pub fn to_xml(&self) -> String {
        let icms    = icms_xml(&self.icms);
        let ipi     = self.ipi.as_ref().map(ipi_xml).unwrap_or_default();
        let pis     = pis_xml(&self.pis);
        let cofins  = cofins_xml(&self.cofins);
        let ibs_cbs = self.ibs_cbs.as_ref()
            .map(|v| quick_xml::se::to_string(v).unwrap_or_default())
            .unwrap_or_default();
        // Ordem do XSD: ICMS, IPI, PIS, PISST, COFINS, COFINSST, …, IBSCBS.
        let pis_st = self.pis.pis_st.as_ref()
            .map(|v| quick_xml::se::to_string(v).unwrap_or_default())
            .unwrap_or_default();
        let cofins_st = self.cofins.cofins_st.as_ref()
            .map(|v| quick_xml::se::to_string(v).unwrap_or_default())
            .unwrap_or_default();
        format!(
            "<imposto><vTotTrib>{}</vTotTrib>{}{}{}{}{}{}{}</imposto>",
            self.v_tot_trib, icms, ipi, pis, pis_st, cofins, cofins_st, ibs_cbs
        )
    }
}


fn icms_xml(icms: &ICMSProcess) -> String {
    use quick_xml::se::to_string;
    let inner = match icms {
        ICMSProcess::ICMS00(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS10(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS20(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS30(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS40(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS51(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS60(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS70(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS90(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS02(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS15(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS53(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMS61(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSPart(v)  => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSST(v)    => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN101(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN102(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN201(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN202(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN500(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN900(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSError(e) => format!("<!-- ICMSError: {} -->", e),
    };
    format!("<ICMS>{}</ICMS>", inner)
}


fn ipi_xml(ipi: &IpiProcess) -> String {
    let selo = match (&ipi.c_selo, ipi.q_selo) {
        (Some(s), Some(q)) => format!("<cSelo>{}</cSelo><qSelo>{}</qSelo>", s, q),
        _ => String::new(),
    };
    let inner = &ipi.inner;
    format!("<IPI>{}<cEnq>{}</cEnq>{}</IPI>", selo, ipi.c_enq, inner)
}


fn pis_xml(pis: &PISProcess) -> String {
    use quick_xml::se::to_string;
    let inner = if let Some(v) = &pis.pis_aliq  { to_string(v).unwrap_or_default() }
    else if let Some(v) = &pis.pis_outr          { to_string(v).unwrap_or_default() }
    else if let Some(v) = &pis.pis_nt            { to_string(v).unwrap_or_default() }
    else if let Some(v) = &pis.pis_qtde          { to_string(v).unwrap_or_default() }
    else if let Some(e) = &pis.pis_invalid       { format!("<!-- PISInvalid: {} -->", e) }
    else { String::new() };
    format!("<PIS>{}</PIS>", inner)
}


fn cofins_xml(cofins: &COFINSProcess) -> String {
    use quick_xml::se::to_string;
    let inner = if let Some(v) = &cofins.cofins_aliq  { to_string(v).unwrap_or_default() }
    else if let Some(v) = &cofins.cofins_outr          { to_string(v).unwrap_or_default() }
    else if let Some(v) = &cofins.cofins_nt            { to_string(v).unwrap_or_default() }
    else if let Some(v) = &cofins.cofins_qtde          { to_string(v).unwrap_or_default() }
    else if let Some(e) = &cofins.cofins_invalid       { format!("<!-- COFINSInvalid: {} -->", e) }
    else { String::new() };
    format!("<COFINS>{}</COFINS>", inner)
}


fn serialize_f64_2_decimals<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>

where
    S: Serializer,
{
    s.serialize_str(&fmt_dec(*x, 2))
}

fn serialize_f64_4_decimals<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>

where
    S: Serializer,
{
    s.serialize_str(&fmt_dec(*x, 4))
}

fn serialize_option_f64_2_decimals<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>

where
    S: Serializer,
{
    match x {
        Some(val) => s.serialize_str(&fmt_dec(*val, 2)),
        None => s.serialize_none(),
    }
}

fn serialize_option_f64_4_decimals<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>

where
    S: Serializer,
{
    match x {
        Some(val) => s.serialize_str(&fmt_dec(*val, 4)),
        None => s.serialize_none(),
    }
}

/* fn serialize_option_f64_4_decimals<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(val) => s.serialize_str(&fmt_dec(*val, 4)),
        None => s.serialize_none(),
    }
}
 */
