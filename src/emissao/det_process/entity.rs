use rust_decimal::Decimal;
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
    #[serde(rename = "vDesc", skip_serializing_if = "Option::is_none")]
    pub v_desc: Option<Decimal>,
    #[serde(rename = "indTot")]
    pub ind_tot: String,
    #[serde(rename = "xPed", skip_serializing_if = "Option::is_none")]
    pub x_ped: Option<String>,
    #[serde(rename = "nItemPed", skip_serializing_if = "Option::is_none")]
    pub n_item_ped: Option<String>,
}

// ─── IPI ──────────────────────────────────────────────────────────────────────

/// IPITrib — CST de saída tributada (50, 99 ...)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPITrib {
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "vBC", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_bc: Option<f64>,
    #[serde(rename = "pIPI", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_ipi: Option<f64>,
    #[serde(rename = "qBCProd", skip_serializing_if = "Option::is_none")]
    pub q_bc_prod: Option<String>,
    #[serde(rename = "vAliqProd", skip_serializing_if = "Option::is_none")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vIPI", serialize_with = "serialize_f64_2_decimals")]
    pub v_ipi: f64,
}

/// IPINT — CST de saída não tributada
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPINT {
    #[serde(rename = "CST")]
    pub cst: String,
}

/// Container do IPI — serializado manualmente em to_xml() para controle das tags
#[derive(Debug, Clone)]
pub struct IpiProcess {
    pub c_enq: String,
    pub c_selo: Option<String>,
    pub q_selo: Option<u32>,
    pub tributado: bool, // true = IPITrib, false = IPINT
    pub inner: String,   // XML pré-serializado da variante interna
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

/// 164 N01 ICMS Informações do ICMS da Operação própria e ST CG M01  1-1  Informar apenas um dos grupos de tributação do ICMS (ICMS00, ICMS10, ...) (v2.0)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ICMSProcess {
    ICMS00(ICMS00),
    ICMS10(ICMS10),
    ICMS20(ICMS20),
    ICMS30(ICMS30),
    ICMS40(ICMS40),
    ICMS51(ICMS51),
    ICMS60(ICMS60),
    ICMS70(ICMS70),
    ICMS90(ICMS90),
    ICMSPart(ICMSPart),
    ICMSSN101(ICMSSN101),
    ICMSSN102(ICMSSN102),
    ICMSSN201(ICMSSN201),
    ICMSSN202(ICMSSN202),
    ICMSSN500(ICMSSN500),
    ICMSSN900(ICMSSN900),
    ICMSError(String),
}

// Defina os structs para cada tipo de ICMS aqui
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS00 {
    // Campos específicos para ICMS00
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBC")]
    pub mod_bc: u8,
    #[serde(rename = "vBC", serialize_with = "serialize_f64_2_decimals")]
    pub v_bc: f64,
    #[serde(rename = "pICMS", serialize_with = "serialize_f64_4_decimals")]
    pub p_icms: f64,
    #[serde(rename = "vICMS", serialize_with = "serialize_f64_2_decimals")]
    pub v_icms: f64,
}

/// CST 10 — Tributada e com cobrança do ICMS por substituição tributária
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS10 {
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBC")]
    pub mod_bc: u8,
    #[serde(rename = "vBC", serialize_with = "serialize_f64_2_decimals")]
    pub v_bc: f64,
    #[serde(rename = "pICMS", serialize_with = "serialize_f64_4_decimals")]
    pub p_icms: f64,
    #[serde(rename = "vICMS", serialize_with = "serialize_f64_2_decimals")]
    pub v_icms: f64,
    #[serde(rename = "modBCST")]
    pub mod_bcst: u8,
    #[serde(rename = "pMVAST", serialize_with = "serialize_f64_4_decimals")]
    pub p_mvast: f64,
    #[serde(rename = "pRedBCST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_red_bcst: Option<f64>,
    #[serde(rename = "vBCST", serialize_with = "serialize_f64_2_decimals")]
    pub v_bcst: f64,
    #[serde(rename = "pICMSST", serialize_with = "serialize_f64_4_decimals")]
    pub p_icmsst: f64,
    #[serde(rename = "vICMSST", serialize_with = "serialize_f64_2_decimals")]
    pub v_icmsst: f64,
}

/// CST 20 — Com redução de base de cálculo
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS20 {
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBC")]
    pub mod_bc: u8,
    #[serde(rename = "pRedBC", serialize_with = "serialize_f64_4_decimals")]
    pub p_red_bc: f64,
    #[serde(rename = "vBC", serialize_with = "serialize_f64_2_decimals")]
    pub v_bc: f64,
    #[serde(rename = "pICMS", serialize_with = "serialize_f64_4_decimals")]
    pub p_icms: f64,
    #[serde(rename = "vICMS", serialize_with = "serialize_f64_2_decimals")]
    pub v_icms: f64,
    #[serde(rename = "vICMSDeson", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms_deson: Option<f64>,
    #[serde(rename = "motDesICMS", skip_serializing_if = "Option::is_none")]
    pub mot_des_icms: Option<u16>,
}

/// CST 30 — Isenta/NT para o emitente e com cobrança do ICMS por ST
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS30 {
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBCST")]
    pub mod_bcst: u8,
    #[serde(rename = "pMVAST", serialize_with = "serialize_f64_4_decimals")]
    pub p_mvast: f64,
    #[serde(rename = "pRedBCST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_red_bcst: Option<f64>,
    #[serde(rename = "vBCST", serialize_with = "serialize_f64_2_decimals")]
    pub v_bcst: f64,
    #[serde(rename = "pICMSST", serialize_with = "serialize_f64_4_decimals")]
    pub p_icmsst: f64,
    #[serde(rename = "vICMSST", serialize_with = "serialize_f64_2_decimals")]
    pub v_icmsst: f64,
    #[serde(rename = "vICMSDeson", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms_deson: Option<f64>,
    #[serde(rename = "motDesICMS", skip_serializing_if = "Option::is_none")]
    pub mot_des_icms: Option<u16>,
}

/// Campos específicos para ICMS40
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "ICMS40")]
pub struct ICMS40 {
    /// Origem da mercadoria Ex: 0 para Nacional
    pub orig: u8,
    /// Código de Situação Tributária
    /// Ex: 40 = Isenta
    /// Ex: 41 = Não tributada
    /// Ex: 50 = Suspensão
    #[serde(rename = "CST")]
    pub cst: u16,
    /// 204.00 N27.1 -x- Sequência XML G N06 0-1 Grupo opcional.
    ///
    /// Informar apenas nas operações:
    /// a) com produtos beneficiados com a desoneração condicional do ICMS.
    /// b) destinadas à SUFRAMA, informando-se o valor que seria devido se não houvesse isenção.
    /// c) de venda a órgão da administração pública direta e suas Nota Fiscal eletrônica fundações e autarquias com isenção do ICMS. (NT 2011/004)
    #[serde(rename = "vICMSDeson", skip_serializing_if = "Option::is_none")]
    pub vicmsdeson: Option<f64>,
    /// Campo será preenchido quando o campo anterior estiver preenchido.
    /// Informar o motivo da desoneração:
    /// 1 = Táxi;
    /// 3 = Produtor Agropecuário;
    /// 4 = Frotista/Locadora;
    /// 5 = Diplomático/Consular;
    /// 6 = Utilitários e Motocicletas da Amazônia Ocidental e Áreas de
    /// Livre Comércio (Resolução 714/88 e 790/94 – CONTRAN e suas alterações);
    /// 7 = SUFRAMA;
    /// 8 = Venda a Órgão Público;
    /// 9 = Outros. (NT 2011/004);
    /// 10 = Deficiente Condutor (Convênio ICMS 38/12);
    /// 11 = Deficiente Não Condutor (Convênio ICMS 38/12).
    /// Revogada a partir da versão 3.
    #[serde(rename = "motDesICMS", skip_serializing_if = "Option::is_none")]
    pub mot_des_icms: Option<u16>,
}

impl Default for ICMS40 {
    fn default() -> Self {
        ICMS40 {
            orig: 0,
            cst: 40,
            vicmsdeson: None,
            mot_des_icms: None,
        }
    }
}

/// CST 51 — Diferimento total ou parcial (todos os campos opcionais por definição SEFAZ)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS51 {
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBC", skip_serializing_if = "Option::is_none")]
    pub mod_bc: Option<u8>,
    #[serde(rename = "pRedBC", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_red_bc: Option<f64>,
    #[serde(rename = "vBC", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_bc: Option<f64>,
    #[serde(rename = "pICMS", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_icms: Option<f64>,
    #[serde(rename = "vICMSOp", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms_op: Option<f64>,
    #[serde(rename = "pDif", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_dif: Option<f64>,
    #[serde(rename = "vICMSDif", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms_dif: Option<f64>,
    #[serde(rename = "vICMS", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms: Option<f64>,
}

/// ICMS60 — ICMS cobrado anteriormente por substituição tributária
/// Usar quando o produto entrou no estoque com ICMS-ST já retido (CFOP 5403, 5405, 6403, 6405)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS60 {
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String, // sempre "60"
    // ** OPCIONAIS ** — xs:sequence minOccurs="0": todos presentes ou nenhum (NT 2011/004)
    /// Valor da BC do ICMS ST retido anteriormente 13v2
    #[serde(rename = "vBCSTRet", skip_serializing_if = "Option::is_none")]
    pub v_bcst_ret: Option<String>,
    /// Alíquota suportada pelo consumidor final 3v2-4 (TDec_0302a04Opc)
    #[serde(rename = "pST", skip_serializing_if = "Option::is_none")]
    pub p_st: Option<String>,
    /// Valor do ICMS Próprio do Substituto cobrado em operação anterior 13v2
    #[serde(rename = "vICMSSubstituto", skip_serializing_if = "Option::is_none")]
    pub v_icms_substituto: Option<String>,
    /// Valor do ICMS ST retido anteriormente 13v2
    #[serde(rename = "vICMSSTRet", skip_serializing_if = "Option::is_none")]
    pub v_icmsst_ret: Option<String>,
}

/// CST 70 — Com redução de BC e cobrança do ICMS por substituição tributária
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS70 {
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBC")]
    pub mod_bc: u8,
    #[serde(rename = "pRedBC", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_red_bc: Option<f64>,
    #[serde(rename = "vBC", serialize_with = "serialize_f64_2_decimals")]
    pub v_bc: f64,
    #[serde(rename = "pICMS", serialize_with = "serialize_f64_4_decimals")]
    pub p_icms: f64,
    #[serde(rename = "vICMS", serialize_with = "serialize_f64_2_decimals")]
    pub v_icms: f64,
    #[serde(rename = "modBCST")]
    pub mod_bcst: u8,
    #[serde(rename = "pMVAST", serialize_with = "serialize_f64_4_decimals")]
    pub p_mvast: f64,
    #[serde(rename = "pRedBCST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_red_bcst: Option<f64>,
    #[serde(rename = "vBCST", serialize_with = "serialize_f64_2_decimals")]
    pub v_bcst: f64,
    #[serde(rename = "pICMSST", serialize_with = "serialize_f64_4_decimals")]
    pub p_icmsst: f64,
    #[serde(rename = "vICMSST", serialize_with = "serialize_f64_2_decimals")]
    pub v_icmsst: f64,
    #[serde(rename = "vICMSDeson", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms_deson: Option<f64>,
    #[serde(rename = "motDesICMS", skip_serializing_if = "Option::is_none")]
    pub mot_des_icms: Option<u16>,
}

/// CST 90 — Outros (todos os campos opcionais, exceto orig e CST)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS90 {
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBC", skip_serializing_if = "Option::is_none")]
    pub mod_bc: Option<u8>,
    #[serde(rename = "pRedBC", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_red_bc: Option<f64>,
    #[serde(rename = "vBC", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_bc: Option<f64>,
    #[serde(rename = "pICMS", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_icms: Option<f64>,
    #[serde(rename = "vICMS", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms: Option<f64>,
    #[serde(rename = "modBCST", skip_serializing_if = "Option::is_none")]
    pub mod_bcst: Option<u8>,
    #[serde(rename = "pMVAST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_mvast: Option<f64>,
    #[serde(rename = "pRedBCST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_red_bcst: Option<f64>,
    #[serde(rename = "vBCST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_bcst: Option<f64>,
    #[serde(rename = "pICMSST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_4_decimals")]
    pub p_icmsst: Option<f64>,
    #[serde(rename = "vICMSST", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icmsst: Option<f64>,
    #[serde(rename = "vICMSDeson", skip_serializing_if = "Option::is_none",
            serialize_with = "serialize_option_f64_2_decimals")]
    pub v_icms_deson: Option<f64>,
    #[serde(rename = "motDesICMS", skip_serializing_if = "Option::is_none")]
    pub mot_des_icms: Option<u16>,
}

impl Default for ICMS90 {
    fn default() -> Self {
        ICMS90 {
            orig: 0, cst: "90".to_string(),
            mod_bc: None, p_red_bc: None, v_bc: None, p_icms: None, v_icms: None,
            mod_bcst: None, p_mvast: None, p_red_bcst: None, v_bcst: None,
            p_icmsst: None, v_icmsst: None, v_icms_deson: None, mot_des_icms: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSPart {
    // TODO: implementar ICMSPart — Partilha do ICMS entre UF de origem e UF de destino
    // pub orig: u8,
    // #[serde(rename = "CST")] pub cst: String,         // "10" ou "90"
    // #[serde(rename = "modBC")] pub mod_bc: u8,
    // #[serde(rename = "vBC")] pub v_bc: f64,
    // #[serde(rename = "pRedBC", skip_serializing_if = "Option::is_none")] pub p_red_bc: Option<f64>,
    // #[serde(rename = "pICMS")] pub p_icms: f64,
    // #[serde(rename = "vICMS")] pub v_icms: f64,
    // #[serde(rename = "modBCST")] pub mod_bcst: u8,
    // #[serde(rename = "pMVAST", skip_serializing_if = "Option::is_none")] pub p_mvast: Option<f64>,
    // #[serde(rename = "pRedBCST", skip_serializing_if = "Option::is_none")] pub p_red_bcst: Option<f64>,
    // #[serde(rename = "vBCST")] pub v_bcst: f64,
    // #[serde(rename = "pICMSST")] pub p_icmsst: f64,
    // #[serde(rename = "vICMSST")] pub v_icmsst: f64,
    // #[serde(rename = "pBCOp")] pub p_bcop: f64,       // percentual da BC operação própria
    // #[serde(rename = "UFST")] pub ufst: String,        // UF para qual é devido o ICMS ST
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN101 {
    // Campos específicos para ICMSSN101
    pub orig: u8,
    #[serde(rename = "CSOSN")]
    pub csosn: String,
    #[serde(rename = "pCredSN")]
    pub p_cred_sn: String,
    #[serde(rename = "vCredICMSSN")]
    pub v_cred_icmssn: String,
}

/// 245.46 N10f ICMSSN102 Grupo CRT=1 – Simples Nacional e CSOSN = 102 CG N01  1-1  Tributação ICMS pelo Simples Nacional, CSOSN=102 (v2.0)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN102 {
    /// Origem da mercadoria Ex: 0 para Nacional
    pub orig: u8,
    /// Código de Situação da Operação - Simples Nacional
    /// Ex: 102 = Tributada pelo Simples Nacional sem permissão de crédito
    /// Ex: 103 = Isenção do ICMS no Simples Nacional para faixa de receita bruta
    /// Ex: 300 = Imune
    /// Ex: 400 = Não tributada pelo Simples Nacional
    #[serde(rename = "CSOSN")]
    pub csosn: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN201 {
    // Campos específicos para ICMSSN201
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN202 {
    // Campos específicos para ICMSSN202
}

/// 245.47 N10g ICMSSN500 Grupo CRT=1 – Simples Nacional e CSOSN = 500 CG N01  1-1  Tributação ICMS pelo Simples Nacional, CSOSN=500 (v2.0)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ICMSSN500 {
    #[serde(rename = "orig")]
    pub orig: u8, // Origem da mercadoria
    #[serde(rename = "CSOSN")]
    pub csosn: String, // Código de Situação da Operação - Simples Nacional
    #[serde(rename = "vBCSTRet", skip_serializing_if = "Option::is_none")]
    pub vbcst_ret: Option<String>, // Valor da BC do ICMS ST retido
    #[serde(rename = "vICMSSTRet", skip_serializing_if = "Option::is_none")]
    pub vicmsst_ret: Option<String>, // Valor do ICMS ST retido
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ICMSSN900 {
    // Campos específicos para ICMSSN900
    #[serde(rename = "orig")]
    pub orig: u8, // Origem da mercadoria
    #[serde(rename = "CSOSN")]
    // 245.55 N12.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    pub csosn: String, // Código de Situação da Operação - Simples Nacional
    #[serde(rename = "modBC", skip_serializing_if = "Option::is_none")]
    pub modbc: Option<String>, // Modalidade de determinação da BC do ICMS
    #[serde(rename = "vBC", skip_serializing_if = "Option::is_none")]
    pub vbc: Option<String>, // Valor da BC do ICMS
    #[serde(rename = "pRedBC", skip_serializing_if = "Option::is_none")]
    pub pred_bc: Option<String>, // Percentual de redução da BC
    #[serde(rename = "pICMS", skip_serializing_if = "Option::is_none")]
    pub picms: Option<String>, // Alíquota do ICMS
    #[serde(rename = "vICMS", skip_serializing_if = "Option::is_none")]
    pub vicms: Option<String>, // Valor do ICMS
    // 245.60 N17.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    #[serde(rename = "modBCST", skip_serializing_if = "Option::is_none")]
    pub modbcst: Option<String>, // Modalidade de determinação da BC do ICMS ST
    #[serde(rename = "pMVAST", skip_serializing_if = "Option::is_none")]
    pub pmvast: Option<String>, // Percentual da margem de valor Adicionado do ICMS ST
    #[serde(rename = "pRedBCST", skip_serializing_if = "Option::is_none")]
    pub pred_bcst: Option<String>, // Percentual de redução da BC do ICMS ST
    #[serde(rename = "vBCST", skip_serializing_if = "Option::is_none")]
    pub vbcst: Option<String>, // Valor da BC do ICMS ST
    #[serde(rename = "pICMSST", skip_serializing_if = "Option::is_none")]
    pub picmsst: Option<String>, // Alíquota do ICMS ST
    #[serde(rename = "vICMSST", skip_serializing_if = "Option::is_none")]
    pub vicmsst: Option<String>, // Valor do ICMS ST
    // 245.52 N27.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    #[serde(rename = "pCredSN", skip_serializing_if = "Option::is_none")]
    pub pcred_sn: Option<String>, // Alíquota aplicável de cálculo do crédito (Simples Nacional)
    #[serde(rename = "vCredICMSSN", skip_serializing_if = "Option::is_none")]
    pub vcred_icmssn: Option<String>, // Valor crédito do ICMS que pode ser aproveitado nos termos do art. 23 da LC 123 (Simples Nacional)
}

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
        format!(
            "<imposto><vTotTrib>{}</vTotTrib>{}{}{}{}{}</imposto>",
            self.v_tot_trib, icms, ipi, pis, cofins, ibs_cbs
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
        ICMSProcess::ICMSSN101(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN102(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN500(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSSN900(v) => to_string(v).unwrap_or_default(),
        ICMSProcess::ICMSError(e) => format!("<!-- ICMSError: {} -->", e),
        _ => String::new(),
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
    else if let Some(v) = &pis.pis_st            { to_string(v).unwrap_or_default() }
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
    else if let Some(v) = &cofins.cofins_st            { to_string(v).unwrap_or_default() }
    else if let Some(e) = &cofins.cofins_invalid       { format!("<!-- COFINSInvalid: {} -->", e) }
    else { String::new() };
    format!("<COFINS>{}</COFINS>", inner)
}

fn serialize_f64_2_decimals<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{:.2}", x))
}

fn serialize_f64_4_decimals<S>(x: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{:.4}", x))
}

fn serialize_option_f64_2_decimals<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(val) => s.serialize_str(&format!("{:.2}", val)),
        None => s.serialize_none(),
    }
}

fn serialize_option_f64_4_decimals<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(val) => s.serialize_str(&format!("{:.4}", val)),
        None => s.serialize_none(),
    }
}

/* fn serialize_option_f64_4_decimals<S>(x: &Option<f64>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(val) => s.serialize_str(&format!("{:.4}", val)),
        None => s.serialize_none(),
    }
}
 */
