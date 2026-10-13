//! Quais `cStat` de retorno da autorização significam **nota autorizada** (NF-e e NFC-e).
//!
//! - `100` — Autorizado o uso da NF-e.
//! - `120` — Autorizado o uso da NF-e, **com alerta** (NT 2026.002 §2; produção 05/10/2026,
//!   inicialmente só NFC-e). A nota está gravada na SEFAZ e **não** deve ser corrigida nem
//!   retransmitida; os alertas vêm no grupo PR13 (`cMsg`/`xMsg`) do `infProt`.
//! - `150` — Autorizado o uso da NF-e, autorização fora de prazo.
//!
//! Regra única do ecossistema: gravisServer, PDV e retaguarda decidem "autorizou?" por aqui
//! (ou pelo espelho JS `sefaz/cstat.js`), nunca por lista literal de códigos.

/// `cStat` que significam nota autorizada.
pub const CSTAT_AUTORIZADO: [i32; 3] = [100, 120, 150];

/// `cStat` de "autorizado com alerta" (NT 2026.002).
pub const CSTAT_AUTORIZADO_COM_ALERTA: i32 = 120;

/// `true` quando o `cStat` da autorização significa nota autorizada (100, 120 ou 150).
pub fn cstat_autorizado(c_stat: i32) -> bool {
    CSTAT_AUTORIZADO.contains(&c_stat)
}

/// Mesma regra de [`cstat_autorizado`] para o `cStat` em texto (como vem gravado no banco e na
/// consulta de situação). Espaços nas bordas são ignorados; texto não numérico é `false`.
pub fn cstat_autorizado_str(c_stat: &str) -> bool {
    c_stat.trim().parse::<i32>().map(cstat_autorizado).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autorizados_sao_100_120_150() {
        assert!(cstat_autorizado(100));
        assert!(cstat_autorizado(120));
        assert!(cstat_autorizado(150));
        for c in [0, 101, 110, 135, 204, 225, 301, 539] {
            assert!(!cstat_autorizado(c), "{c} não é autorização");
        }
    }

    #[test]
    fn versao_texto_ignora_bordas_e_lixo() {
        assert!(cstat_autorizado_str("120"));
        assert!(cstat_autorizado_str(" 150 "));
        assert!(!cstat_autorizado_str(""));
        assert!(!cstat_autorizado_str("abc"));
        assert!(!cstat_autorizado_str("539"));
    }
}
