//! Mapeamento entre **código IBGE da UF** (2 dígitos — como aparece em `cUF` e nos 2 primeiros
//! dígitos da chave de acesso) e a **sigla da UF** (como o `webservices.json` indexa os endpoints).
//!
//! Estado atual → alvo: antes cada operação passava `"SP"` fixo ao roteamento; agora a UF real é
//! derivada do documento (chave/`cUF`) e convertida em sigla aqui, num único lugar (fase A6).

use crate::error::{DfeError, Result};

/// Tabela canônica `(código IBGE, sigla)` das 27 UFs.
/// `pub(crate)` para que testes de roteamento iterem as 27 UFs sem duplicar a tabela.
pub(crate) const UFS: &[(&str, &str)] = &[
    ("11", "RO"), ("12", "AC"), ("13", "AM"), ("14", "RR"), ("15", "PA"), ("16", "AP"),
    ("17", "TO"), ("21", "MA"), ("22", "PI"), ("23", "CE"), ("24", "RN"), ("25", "PB"),
    ("26", "PE"), ("27", "AL"), ("28", "SE"), ("29", "BA"), ("31", "MG"), ("32", "ES"),
    ("33", "RJ"), ("35", "SP"), ("41", "PR"), ("42", "SC"), ("43", "RS"), ("50", "MS"),
    ("51", "MT"), ("52", "GO"), ("53", "DF"),
];

/// Sigla da UF a partir do código IBGE (ex.: `"35"` → `"SP"`).
pub fn sigla_por_codigo(codigo: &str) -> Result<&'static str> {
    UFS.iter()
        .find(|(c, _)| *c == codigo)
        .map(|(_, s)| *s)
        .ok_or_else(|| DfeError::Validacao(format!("código de UF inválido: {codigo}")))
}

/// Código IBGE a partir da sigla (ex.: `"SP"` → `"35"`).
pub fn codigo_por_sigla(sigla: &str) -> Result<&'static str> {
    UFS.iter()
        .find(|(_, s)| *s == sigla)
        .map(|(c, _)| *c)
        .ok_or_else(|| DfeError::Validacao(format!("sigla de UF inválida: {sigla}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ida_e_volta() {
        assert_eq!(sigla_por_codigo("35").unwrap(), "SP");
        assert_eq!(sigla_por_codigo("12").unwrap(), "AC");
        assert_eq!(codigo_por_sigla("SP").unwrap(), "35");
        assert_eq!(codigo_por_sigla("DF").unwrap(), "53");
        // Round-trip completo das 27.
        for (cod, sig) in UFS {
            assert_eq!(sigla_por_codigo(cod).unwrap(), *sig);
            assert_eq!(codigo_por_sigla(sig).unwrap(), *cod);
        }
    }

    #[test]
    fn invalidos() {
        assert!(sigla_por_codigo("99").is_err());
        assert!(codigo_por_sigla("XX").is_err());
    }
}
