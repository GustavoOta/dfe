//! Structs de PIS e COFINS do XML de emissão (A7: extraído de entity.rs).

use serde::{Deserialize, Serialize};
use super::{serialize_f64_2_decimals, serialize_f64_4_decimals, serialize_option_f64_2_decimals};


/// Grupo PIS Informar apenas um dos grupos PIS (PISAliq, PISQtde, PISNT ... )
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISProcess {
    #[serde(rename = "PISAliq", skip_serializing_if = "Option::is_none")]
    pub pis_aliq: Option<PISAliq>,
    #[serde(rename = "PISQtde", skip_serializing_if = "Option::is_none")]
    pub pis_qtde: Option<PISQtde>,
    #[serde(rename = "PISNT", skip_serializing_if = "Option::is_none")]
    pub pis_nt: Option<PISNT>,
    #[serde(rename = "PISOutr", skip_serializing_if = "Option::is_none")]
    pub pis_outr: Option<PISOutr>,
    #[serde(rename = "PISST", skip_serializing_if = "Option::is_none")]
    pub pis_st: Option<PISST>,
    #[serde(rename = "PISInvalid", skip_serializing_if = "Option::is_none")]
    pub pis_invalid: Option<String>,
}

impl Default for PISProcess {
    fn default() -> Self {
        PISProcess {
            pis_aliq: None,
            pis_qtde: None,
            pis_nt: None,
            pis_outr: None,
            pis_st: None,
            pis_invalid: None,
        }
    }
}


/// Grupo PIS tributado pela alíquota
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISAliq {
    /// Código de Situação Tributária
    /// 01 = Operação Tributável (base de cálculo = valor da operação alíquota normal (cumulativo/não cumulativo));
    /// 02 = Operação Tributável (base de cálculo = valor da operação (alíquota diferenciada));
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "vBC", serialize_with = "serialize_f64_2_decimals")]
    pub v_bc: f64,
    #[serde(rename = "pPIS", serialize_with = "serialize_f64_4_decimals")]
    pub p_pis: f64,
    #[serde(rename = "vPIS", serialize_with = "serialize_f64_2_decimals")]
    pub v_pis: f64,
}


/// Grupo PIS tributado por Qtde
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISQtde {
    /// Código de Situação Tributária
    /// 03=Operação Tributável (base de cálculo = quantidade vendida x alíquota por unidade de produto);
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "qBCProd")]
    pub qbc_prod: String,
    #[serde(rename = "vAliqProd")]
    pub valiq_prod: String,
    #[serde(rename = "vPIS")]
    pub vpis: String,
}


/// Grupo PIS não tributado
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISNT {
    /// Código de Situação Tributária
    /// 04 = Operação Tributável (tributação monofásica - alíquota zero);
    /// 05 = Operação Tributável (Substituição Tributária);
    /// 06 = Operação Tributável (alíquota zero);
    /// 07 = Operação Isenta da Contribuição;
    /// 08 = Operação Sem Incidência da Contribuição;
    /// 09 = Operação com Suspensão da Contribuição;
    /// ...
    #[serde(rename = "CST")]
    pub cst: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISOutr {
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "qBCProd", skip_serializing_if = "Option::is_none")]
    pub qbc_prod: Option<String>,
    #[serde(rename = "vAliqProd", skip_serializing_if = "Option::is_none")]
    pub valiq_prod: Option<String>,
    #[serde(rename = "vPIS", skip_serializing_if = "Option::is_none")]
    pub vpis: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISST {
    /// -*-
    #[serde(rename = "vBC", skip_serializing_if = "Option::is_none")]
    pub v_bc: Option<String>,
    #[serde(rename = "pPIS", skip_serializing_if = "Option::is_none")]
    pub p_pis: Option<String>,
    /// -*-
    #[serde(rename = "qBCProd", skip_serializing_if = "Option::is_none")]
    pub qbc_prod: Option<String>,
    #[serde(rename = "vAliqProd", skip_serializing_if = "Option::is_none")]
    pub valiq_prod: Option<String>,
    #[serde(rename = "vPIS", skip_serializing_if = "Option::is_none")]
    pub vpis: Option<String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSProcess {
    #[serde(rename = "COFINSAliq", skip_serializing_if = "Option::is_none")]
    pub cofins_aliq: Option<COFINSAliq>,
    #[serde(rename = "COFINSQtde", skip_serializing_if = "Option::is_none")]
    pub cofins_qtde: Option<COFINSQtde>,
    #[serde(rename = "COFINSNT", skip_serializing_if = "Option::is_none")]
    pub cofins_nt: Option<COFINSNT>,
    #[serde(rename = "COFINSOutr", skip_serializing_if = "Option::is_none")]
    pub cofins_outr: Option<COFINSOutr>,
    #[serde(rename = "COFINSST", skip_serializing_if = "Option::is_none")]
    pub cofins_st: Option<COFINSST>,
    #[serde(rename = "COFINSInvalid", skip_serializing_if = "Option::is_none")]
    pub cofins_invalid: Option<String>,
}


impl Default for COFINSProcess {
    fn default() -> Self {
        COFINSProcess {
            cofins_aliq: None,
            cofins_qtde: None,
            cofins_nt: None,
            cofins_outr: None,
            cofins_st: None,
            cofins_invalid: None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSAliq {
    /// Código de Situação Tributária
    /// 01 = Operação Tributável (base de cálculo = valor da operação alíquota normal (cumulativo/não cumulativo));
    /// 02 = Operação Tributável (base de cálculo = valor da operação (alíquota diferenciada));
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "vBC", serialize_with = "serialize_f64_2_decimals")]
    pub v_bc: f64,
    #[serde(rename = "pCOFINS", serialize_with = "serialize_f64_4_decimals")]
    pub p_cofins: f64,
    #[serde(rename = "vCOFINS", serialize_with = "serialize_f64_2_decimals")]
    pub v_cofins: f64,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSQtde {
    /// Código de Situação Tributária
    /// 03=Operação Tributável (base de cálculo = quantidade vendida x alíquota por unidade de produto);
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "qBCProd")]
    pub qbc_prod: String,
    #[serde(rename = "vAliqProd")]
    pub valiq_prod: String,
    #[serde(rename = "vCOFINS")]
    pub vcofins: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSNT {
    /// Código de Situação Tributária
    /// 04 = Operação Tributável (tributação monofásica - alíquota zero);
    /// 05 = Operação Tributável (Substituição Tributária);
    /// 06 = Operação Tributável (alíquota zero);
    /// 07 = Operação Isenta da Contribuição;
    /// 08 = Operação Sem Incidência da Contribuição;
    /// 09 = Operação com Suspensão da Contribuição;
    /// ...
    #[serde(rename = "CST")]
    pub cst: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSOutr {
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(
        rename = "vBC",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_2_decimals"
    )]
    pub v_bc: Option<f64>,
    #[serde(
        rename = "pCOFINS",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_2_decimals"
    )]
    pub p_cofins: Option<f64>,
    #[serde(
        rename = "vCOFINS",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_2_decimals"
    )]
    pub v_cofins: Option<f64>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSST {
    /// -*-
    #[serde(rename = "vBC", skip_serializing_if = "Option::is_none")]
    pub v_bc: Option<String>,
    #[serde(rename = "pCOFINS", skip_serializing_if = "Option::is_none")]
    pub p_cofins: Option<String>,
    /// -*-
    #[serde(rename = "qBCProd", skip_serializing_if = "Option::is_none")]
    pub qbc_prod: Option<String>,
    #[serde(rename = "vAliqProd", skip_serializing_if = "Option::is_none")]
    pub valiq_prod: Option<String>,
    #[serde(rename = "vCOFINS", skip_serializing_if = "Option::is_none")]
    pub vcofins: Option<String>,
}
