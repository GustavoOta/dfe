//! Structs de ICMS (todas as variantes + enum ICMSProcess) do XML de emissão (A7: extraído de entity.rs).

use serde::{Deserialize, Serialize};
use super::{serialize_f64_2_decimals, serialize_f64_4_decimals, serialize_option_f64_2_decimals, serialize_option_f64_4_decimals};


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
    /// Alíquota suportada pelo consumidor final 3v2-4 (TDec_0302a04Opc — não aceita zero)
    #[serde(rename = "pST", skip_serializing_if = "Option::is_none")]
    pub p_st: Option<String>,
    /// Valor do ICMS Próprio do Substituto cobrado em operação anterior 13v2 (opcional no grupo)
    #[serde(rename = "vICMSSubstituto", skip_serializing_if = "Option::is_none")]
    pub v_icms_substituto: Option<String>,
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
