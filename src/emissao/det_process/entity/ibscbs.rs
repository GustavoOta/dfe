//! Structs de IBS/CBS (reforma tributária) do XML de emissão (A7: extraído de entity.rs).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "IBSCBS")]
pub struct IBSCBSProcess {
    /// Código de Situação Tributária do IBS e CBS NUM 3
    #[serde(rename = "CST")]
    pub cst: String,
    /// Código de Classificação Tributária do IBS e da CBS CHAR 6
    #[serde(rename = "cClassTrib")]
    pub c_class_trib: String,
    /// Grupo de Informações do IBS e CBS
    #[serde(rename = "gIBSCBS")]
    pub g_ibscbs: GIBSCBS,
}


impl Default for IBSCBSProcess {
    fn default() -> Self {
        IBSCBSProcess {
            cst: "IBS CBS cst não infomado".to_string(),
            c_class_trib: "c_class_trib não infomado".to_string(),
            g_ibscbs: GIBSCBS {
                v_bc: "-0.01".to_string(),
                g_ibs_uf: GIBSUF {
                    p_ibs_uf: "p_ibs_uf valor incorreto".to_string(),
                    ..Default::default()
                },
                g_ibs_mun: GIBSMun {
                    p_ibs_mun: "p_ibs_mun valor incorreto".to_string(),
                    ..Default::default()
                },
                v_ibs: "v_ibs valor incorreto".to_string(),
                g_cbs: GCBS {
                    p_cbs: "p_cbs valor incorreto".to_string(),
                    ..Default::default()
                },
            },
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GIBSCBS {
    /// Base de cálculo do IBS e CBS 13v2
    #[serde(rename = "vBC")]
    pub v_bc: String,
    /// Grupo de Informações do IBS
    #[serde(rename = "gIBSUF")]
    pub g_ibs_uf: GIBSUF,
    /// Grupo de Informações do IBS para o município
    #[serde(rename = "gIBSMun")]
    pub g_ibs_mun: GIBSMun,
    /// Valor do IBS e CBS 13v2
    #[serde(rename = "vIBS")]
    pub v_ibs: String,
    /// Grupo de Informações da CBS
    #[serde(rename = "gCBS")]
    pub g_cbs: GCBS,
}

/// GIBSUF ************************************************************************************
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GIBSUF {
    /// Alíquota do IBS de competência das UF 3v2-4
    #[serde(rename = "pIBSUF")]
    pub p_ibs_uf: String,
    /// Valor do tributo considerando BC x Alq do IBS, sem considerar qualquer desoneração. 13v2
    /// DESCONTINUADO ? Vamos aguardar um pouco antes de remover
    #[serde(rename = "vTribOP", skip_serializing_if = "Option::is_none")]
    pub v_trib_op: Option<String>,
    /// Grupo de Informações do Diferimento
    #[serde(rename = "gDif", skip_serializing_if = "Option::is_none")]
    pub g_dif: Option<GDif>,
    /// Grupo de Informações da devolução de tributos
    #[serde(rename = "gDevTrib", skip_serializing_if = "Option::is_none")]
    pub g_dev_trib: Option<GDevTrib>,
    /// Grupo de informações da redução da alíquota
    #[serde(rename = "gRed", skip_serializing_if = "Option::is_none")]
    pub g_red: Option<GRed>,
    /// Grupo de informações da Tributação Regular
    #[serde(rename = "gTribRegular", skip_serializing_if = "Option::is_none")]
    pub g_trib_regular: Option<GTribRegular>,
    /// Valor do IBS de competência da UF 13v2
    #[serde(rename = "vIBSUF")]
    pub v_ibs_uf: String,
}


impl Default for GIBSUF {
    fn default() -> Self {
        GIBSUF {
            p_ibs_uf: "p_ibs_uf valor incorreto".to_string(),
            v_trib_op: None,
            g_dif: None,
            g_dev_trib: None,
            g_red: None,
            g_trib_regular: None,
            v_ibs_uf: "v_ibs_uf valor incorreto".to_string(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GDif {
    /// Percentual do diferimento 3v2-4
    #[serde(rename = "pDif")]
    pub p_dif: Decimal,
    /// Valor do Diferimento 13v2
    #[serde(rename = "vDif")]
    pub v_dif: Decimal,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GDevTrib {
    /// Valor do tributo devolvido 13v2
    #[serde(rename = "vDevTrib")]
    pub v_dev_trib: Decimal,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GRed {
    /// Percentual de redução da alíquota 3v2-4
    #[serde(rename = "pRedAliq")]
    pub p_red_aliq: Decimal,
    /// Alíquota Efetiva do IBS de competência das UF que será aplicada a Base de Cálculo
    /// Alíquota efetiva, após aplicação da redução de alíquota 3v2-4
    #[serde(rename = "pAliqEfet")]
    pub p_aliq_efet: Decimal,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GTribRegular {
    /// DESCONTINUADO ? Vamos aguardar um pouco antes de remover
    /// Código de Situação Tributária do IBS e CBS
    /// Informado como seria caso não cumprida a condição resolutória/suspensiva.
    /// Utilizar tabela CÓDIGO DE CLASSIFICAÇÃO TRIBUTÁRIA DO IBS E DA CBS NUM 3
    #[serde(rename = "CSTReg", skip_serializing_if = "Option::is_none")]
    pub cst_reg: Option<String>,
    /// Código de Classificação Tributária do IBS e CBS
    /// Informado como seria caso não cumprida a condição resolutória/suspensiva.
    /// Utilizar tabela CÓDIGO DE CLASSIFICAÇÃO TRIBUTÁRIA DO IBS E DA CBS CHAR 6
    #[serde(rename = "cClassTribReg", skip_serializing_if = "Option::is_none")]
    pub c_class_trib_reg: Option<String>,
    /// Valor da alíquota
    /// Informado como seria caso não cumprida a condição resolutória/suspensiva.
    /// Utilizar tabela CÓDIGO DE CLASSIFICAÇÃO TRIBUTÁRIA DO IBS E DA CBS 3v2-4
    #[serde(rename = "pAliqEfetReg", skip_serializing_if = "Option::is_none")]
    pub p_aliq_efet_reg: Option<Decimal>,
    /// Valor do Tributo (IBS)
    /// Informado como seria caso não cumprida a condição resolutória/suspensiva.
    /// Utilizar tabela CÓDIGO DE CLASSIFICAÇÃO TRIBUTÁRIA DO IBS E DA CBS 13v2
    #[serde(rename = "vTribReg", skip_serializing_if = "Option::is_none")]
    pub v_trib_reg: Option<Decimal>,
}


/// GIBSMun ************************************************************************************
/// Grupo de Informações do IBS para o município
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GIBSMun {
    /// Alíquota do IBS de competência do Município 3v2-4
    #[serde(rename = "pIBSMun")]
    pub p_ibs_mun: String,
    /// Valor do tributo considerando BC x Alq do IBS, sem considerar qualquer desoneração. 13v2
    #[serde(rename = "vTribOP", skip_serializing_if = "Option::is_none")]
    pub v_trib_op: Option<String>,
    /// Grupo de Informações do Diferimento
    #[serde(rename = "gDif", skip_serializing_if = "Option::is_none")]
    pub g_dif: Option<GDif>,
    /// Grupo de Informações da devolução de tributos
    #[serde(rename = "gDevTrib", skip_serializing_if = "Option::is_none")]
    pub g_dev_trib: Option<GDevTrib>,
    /// Grupo de informações da redução da alíquota
    #[serde(rename = "gRed", skip_serializing_if = "Option::is_none")]
    pub g_red: Option<GRed>,
    /// Grupo de informações da Tributação Regular
    /// DESCONTINUADO ? Vamos aguardar um pouco antes de remover
    #[serde(rename = "gTribRegular", skip_serializing_if = "Option::is_none")]
    pub g_trib_regular: Option<GTribRegular>,
    /// Valor do IBS de competência do Município 13v2
    #[serde(rename = "vIBSMun")]
    pub v_ibs_mun: String,
}


impl Default for GIBSMun {
    fn default() -> Self {
        GIBSMun {
            p_ibs_mun: "p_ibs_mun valor incorreto".to_string(),
            v_trib_op: None,
            g_dif: None,
            g_dev_trib: None,
            g_red: None,
            g_trib_regular: None,
            v_ibs_mun: "v_ibs_mun valor incorreto".to_string(),
        }
    }
}


/// GCBS ************************************************************************************
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GCBS {
    /// Alíquota da CBS 3v2-4
    #[serde(rename = "pCBS")]
    pub p_cbs: String,
    /// Valor bruto do tributo na operação
    /// Valor do tributo considerando BC x Alq da CBS, sem considerar qualquer desoneração.
    #[serde(rename = "vTribOp", skip_serializing_if = "Option::is_none")]
    pub v_trib_op: Option<Decimal>,
    /// Grupo de Informações do Diferimento
    #[serde(rename = "gDif", skip_serializing_if = "Option::is_none")]
    pub g_dif: Option<GDif>,
    /// Grupo de Informações da devolução de tributos
    #[serde(rename = "gDevTrib", skip_serializing_if = "Option::is_none")]
    pub g_dev_trib: Option<GDevTrib>,
    /// Grupo de informações da redução da alíquota
    #[serde(rename = "gRed", skip_serializing_if = "Option::is_none")]
    pub g_red: Option<GRed>,
    /// Grupo de informações da Tributação Regular
    /// Informado como seria caso não cumprida a condição resolutória/suspensiva.
    #[serde(rename = "gTribRegular", skip_serializing_if = "Option::is_none")]
    pub g_trib_regular: Option<GTribRegular>,
    /// Valor da CBS 13v2
    #[serde(rename = "vCBS")]
    pub v_cbs: String,
}


impl Default for GCBS {
    fn default() -> Self {
        GCBS {
            p_cbs: "p_cbs valor incorreto".to_string(),
            v_trib_op: None,
            g_dif: None,
            g_dev_trib: None,
            g_red: None,
            g_trib_regular: None,
            v_cbs: "v_cbs valor incorreto".to_string(),
        }
    }
}
