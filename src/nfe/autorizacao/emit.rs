use serde::{Deserialize, Serialize};

/// Emitente da NF-e (Nota Fiscal Eletrônica) ou NFC-e (Nota Fiscal de Consumidor Eletrônica)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "emit")]
pub struct EmitProcess {
    /// CNPJ do emitente Ex: 12345678000123
    #[serde(rename = "CNPJ", skip_serializing_if = "Option::is_none")]
    pub cnpj: Option<String>,
    /// CPF do emitente Ex: 12345678901
    #[serde(rename = "CPF", skip_serializing_if = "Option::is_none")]
    pub cpf: Option<String>,
    /// Razão social do emitente Ex: Empresa Ltda
    #[serde(rename = "xNome")]
    pub x_nome: String,
    /// Nome fantasia do emitente Ex: Empresa
    #[serde(rename = "xFant", skip_serializing_if = "Option::is_none")]
    pub x_fant: Option<String>,
    /// Bloco de endereço do emitente
    #[serde(rename = "enderEmit")]
    pub ender_emit: EnderEmitProcess,
    /// Inscrição estadual do emitente Ex: 123456789
    #[serde(rename = "IE", skip_serializing_if = "Option::is_none")]
    pub ie: Option<u64>,

    /// Código de Regime Tributário do emitente
    /// Ex: 1 para Simples Nacional
    /// Ex: 2 para Simples Nacional - excesso de sublimite de receita bruta
    /// Ex: 3 para Regime Normal (Lucro Presumido ou Lucro Real)
    /// Ex: 4 para MEI - Microempreendedor Individual
    #[serde(rename = "CRT")]
    pub crt: u8,
}

/// Endereço do emitente
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnderEmitProcess {
    /// Logradouro do emitente Ex: Rua das Flores
    #[serde(rename = "xLgr")]
    pub x_lgr: String,

    /// Número do endereço Ex: 1234 ou S/N
    pub nro: String,

    /// Bairro do emitente Ex: Centro
    #[serde(rename = "xBairro")]
    pub x_bairro: String,

    /// Código do município Ex: 4205407 para Lages
    #[serde(rename = "cMun")]
    pub c_mun: u32,

    /// Nome do município Ex: Lages
    #[serde(rename = "xMun")]
    pub x_mun: String,

    /// Sigla da UF Ex: SC
    #[serde(rename = "UF")]
    pub uf: String,

    /// CEP do emitente Ex: 88509900
    #[serde(rename = "CEP")]
    pub cep: u32,

    /// Código do país Ex: 1058 para Brasil
    #[serde(rename = "cPais")]
    pub c_pais: u16,

    /// Nome do país Ex: Brasil
    #[serde(rename = "xPais")]
    pub x_pais: String,
}
