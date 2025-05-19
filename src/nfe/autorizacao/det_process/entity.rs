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
    #[serde(rename = "indTot")]
    pub ind_tot: String,
    #[serde(rename = "xPed", skip_serializing_if = "Option::is_none")]
    pub x_ped: Option<String>,
    #[serde(rename = "nItemPed", skip_serializing_if = "Option::is_none")]
    pub n_item_ped: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "imposto")]
pub struct ImpostoProcess {
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: String,
    #[serde(rename = "ICMS")]
    pub icms: ICMSProcess,
    #[serde(rename = "PIS")]
    pub pis: PISProcess,
    #[serde(rename = "COFINS")]
    pub cofins: COFINSProcess,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS10 {
    // Campos específicos para ICMS10
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS20 {
    // Campos específicos para ICMS20
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS30 {
    // Campos específicos para ICMS30
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS51 {
    // Campos específicos para ICMS51
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS60 {
    // Campos específicos para ICMS60
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS70 {
    // Campos específicos para ICMS70
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS90 {
    // Campos específicos para ICMS90
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
}

impl Default for ICMS90 {
    fn default() -> Self {
        ICMS90 {
            orig: 0,
            cst: "90".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSPart {
    // Campos específicos para ICMSPart
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN101 {
    // Campos específicos para ICMSSN101
    pub orig: u8,
    #[serde(rename = "CSOSN")]
    pub csosn: u16,
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
    pub csosn: u16,
    #[serde(rename = "pCredSN")]
    pub p_cred_sn: f64,
    #[serde(rename = "vCredICMSSN")]
    pub v_cred_icmssn: f64,
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN500 {
    #[serde(rename = "orig")]
    pub orig: u8, // Origem da mercadoria
    #[serde(rename = "CSOSN")]
    pub csosn: u16, // Código de Situação da Operação - Simples Nacional
    #[serde(rename = "vBCSTRet")]
    pub vbcst_ret: Option<String>, // Valor da BC do ICMS ST retido
    #[serde(rename = "vICMSSTRet")]
    pub vicmsst_ret: Option<String>, // Valor do ICMS ST retido
}

impl Default for ICMSSN500 {
    fn default() -> Self {
        ICMSSN500 {
            orig: 0,
            csosn: 500,
            vbcst_ret: None,
            vicmsst_ret: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN900 {
    // Campos específicos para ICMSSN900
    #[serde(rename = "orig")]
    pub orig: String, // Origem da mercadoria
    #[serde(rename = "CSOSN")]
    // 245.55 N12.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    pub csosn: String, // Código de Situação da Operação - Simples Nacional
    #[serde(rename = "modBC")]
    pub modbc: Option<String>, // Modalidade de determinação da BC do ICMS
    #[serde(rename = "vBC")]
    pub vbc: Option<String>, // Valor da BC do ICMS
    #[serde(rename = "pRedBC")]
    pub pred_bc: Option<String>, // Percentual de redução da BC
    #[serde(rename = "pICMS")]
    pub picms: Option<String>, // Alíquota do ICMS
    #[serde(rename = "vICMS")]
    pub vicms: Option<String>, // Valor do ICMS
    // 245.60 N17.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    #[serde(rename = "modBCST")]
    pub modbcst: Option<String>, // Modalidade de determinação da BC do ICMS ST
    #[serde(rename = "pMVAST")]
    pub pmvast: Option<String>, // Percentual da margem de valor Adicionado do ICMS ST
    #[serde(rename = "pRedBCST")]
    pub pred_bcst: Option<String>, // Percentual de redução da BC do ICMS ST
    #[serde(rename = "vBCST")]
    pub vbcst: Option<String>, // Valor da BC do ICMS ST
    #[serde(rename = "pICMSST")]
    pub picmsst: Option<String>, // Alíquota do ICMS ST
    #[serde(rename = "vICMSST")]
    pub vicmsst: Option<String>, // Valor do ICMS ST
    // 245.52 N27.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    #[serde(rename = "pCredSN")]
    pub pcred_sn: Option<String>, // Alíquota aplicável de cálculo do crédito (Simples Nacional)
    #[serde(rename = "vCredICMSSN")]
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
    /// -*-
    #[serde(
        rename = "vBC",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_2_decimals"
    )]
    pub vbc: Option<f64>,
    #[serde(
        rename = "pCOFINS",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_4_decimals"
    )]
    pub p_cofins: Option<f64>,
    /// -*-
    #[serde(
        rename = "qBCProd",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_4_decimals"
    )]
    pub qbc_prod: Option<f64>,
    #[serde(
        rename = "vAliqProd",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_4_decimals"
    )]
    pub valiq_prod: Option<f64>,
    #[serde(
        rename = "vCOFINS",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_f64_4_decimals"
    )]
    pub vcofins: Option<f64>,
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
