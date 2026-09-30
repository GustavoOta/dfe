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
    pub ie: Option<String>,

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
    pub c_mun: String,

    /// Nome do município Ex: Lages
    #[serde(rename = "xMun")]
    pub x_mun: String,

    /// Sigla da UF Ex: SC
    #[serde(rename = "UF")]
    pub uf: String,

    /// CEP do emitente Ex: 88509900
    #[serde(rename = "CEP")]
    pub cep: String,

    /// Código do país Ex: 1058 para Brasil
    #[serde(rename = "cPais")]
    pub c_pais: u16,

    /// Nome do país Ex: Brasil
    #[serde(rename = "xPais")]
    pub x_pais: String,
}

/// Tamanho máximo de `xNome` e `xFant` do emitente no XSD da NF-e/NFC-e (60).
pub(super) const EMIT_X_NOME_MAX: usize = 60;

/// Razão social (`xNome`) ou nome fantasia (`xFant`) do emitente cortado nos 60
/// caracteres que o XSD aceita.
/// Conta caracteres (não bytes), então acento não quebra no meio.
pub(super) fn limitar_x_nome(x_nome: &str) -> String {
    x_nome
        .trim()
        .chars()
        .take(EMIT_X_NOME_MAX)
        .collect::<String>()
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests_limitar_x_nome {
    use super::*;

    #[test]
    fn razao_social_de_62_caracteres_corta_nos_60() {
        let nome = "A".repeat(62);
        assert_eq!(limitar_x_nome(&nome).chars().count(), 60);
    }

    #[test]
    fn razao_social_curta_fica_intacta() {
        assert_eq!(limitar_x_nome("  Empresa Ltda "), "Empresa Ltda");
    }

    #[test]
    fn corte_conta_caracteres_e_nao_bytes() {
        let nome = "Ç".repeat(70);
        assert_eq!(limitar_x_nome(&nome), "Ç".repeat(60));
    }

    #[test]
    fn espaco_no_ponto_de_corte_nao_sobra_no_fim() {
        let nome = format!("{} RESTO", "B".repeat(59));
        assert_eq!(limitar_x_nome(&nome), "B".repeat(59));
    }

    #[test]
    fn nome_fantasia_longo_corta_nos_60_e_ausente_segue_ausente() {
        let fant = Some("F".repeat(75));
        assert_eq!(fant.as_deref().map(limitar_x_nome), Some("F".repeat(60)));
        let sem: Option<String> = None;
        assert_eq!(sem.as_deref().map(limitar_x_nome), None);
    }
}
