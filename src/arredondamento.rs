//! Arredondamento fiscal único da crate — e do ecossistema (`gravisServer`, `gravis-pdv`,
//! `gravis-retaguarda` reusam `dfe::arred2`; o JS tem o espelho `arredondamento.js`).
//!
//! Regra: meio para cima, afastando do zero (`MidpointAwayFromZero`), sobre o valor
//! DECIMAL — o mesmo `round(..., 2, PHP_ROUND_HALF_UP)` do NFePHP.
//!
//! Por que não `format!("{:.2}", f64)` nem `(v * 100.0).round() / 100.0`: os dois operam no
//! binário. 1.005 é 1.00499999999999989… em f64, então os dois dão 1.00; e o `format!` ainda
//! desempata para o par. Aqui o f64 passa primeiro por 15 algarismos significativos (o que
//! o f64 garante), vira `Decimal` e só então é arredondado.
//!
//! Uso: todo campo derivado sai dos campos já arredondados que vão no XML (vBC do PIS =
//! arred2(vProd − vICMS)), e todo total é a soma dos itens arredondados.

use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, RoundingStrategy};
use std::borrow::Borrow;

/// f64 → Decimal sem o ruído binário (15 algarismos significativos). Não finito → 0.
fn to_decimal(v: f64) -> Decimal {
    if !v.is_finite() || v == 0.0 {
        return Decimal::ZERO;
    }
    Decimal::from_scientific(&format!("{:.14e}", v))
        .or_else(|_| Decimal::from_f64(v).ok_or(()))
        .unwrap_or(Decimal::ZERO)
}

/// Arredonda um `Decimal` para `casas` com meio para cima.
pub fn arred_decimal(v: Decimal, casas: u32) -> Decimal {
    v.round_dp_with_strategy(casas, RoundingStrategy::MidpointAwayFromZero)
}

/// Arredonda para `casas` decimais, meio para cima.
pub fn arred(v: f64, casas: u32) -> f64 {
    arred_decimal(to_decimal(v), casas).to_f64().unwrap_or(0.0)
}

/// Arredonda para 2 casas (centavo), meio para cima.
pub fn arred2(v: f64) -> f64 {
    arred(v, 2)
}

/// Formata com exatamente `casas` decimais, arredondando meio para cima (texto do XML).
pub fn fmt_dec<V: Borrow<f64>>(v: V, casas: u32) -> String {
    let d = arred_decimal(to_decimal(*v.borrow()), casas);
    format!("{:.*}", casas as usize, d)
}

/// Formata um `Decimal` com exatamente `casas` decimais, meio para cima.
pub fn fmt_decimal(v: Decimal, casas: u32) -> String {
    format!("{:.*}", casas as usize, arred_decimal(v, casas))
}

/// f64 → `Decimal` sem o ruído binário (15 algarismos significativos). Para rateios que
/// trabalham em `Decimal` a partir do vProd recebido em f64.
pub fn para_decimal(v: f64) -> Decimal {
    to_decimal(v)
}

/// Formata com no mínimo `min` e no máximo `max` decimais (zeros à direita cortados até `min`).
/// Para vUnCom/vUnTrib, que aceitam até 10 casas: 602.6097 não pode sair "602.61".
pub fn fmt_dec_ate<V: Borrow<f64>>(v: V, min: u32, max: u32) -> String {
    let s = fmt_dec(v, max);
    let Some(ponto) = s.find('.') else { return s };
    let minimo = ponto + 1 + min as usize;
    let mut fim = s.len();
    while fim > minimo && s.as_bytes()[fim - 1] == b'0' {
        fim -= 1;
    }
    s[..fim].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meio_centavo_sobe_sem_ruido_binario() {
        // Espelho de gravis-pdv/src/nfe/arredondamento.test.js — os dois lados dão o mesmo centavo.
        for (v, esperado) in [
            (1.005, 1.01),
            (2.675, 2.68),
            (0.125, 0.13),
            (8.345, 8.35),
            (1.004999, 1.0),
            (-1.005, -1.01),
            (0.1 + 0.2, 0.3),
            (2109.13395, 2109.13),
            (1856.0379, 1856.04),
            (1234567.895, 1234567.9),
            (1e-7, 0.0),
            (0.0, 0.0),
        ] {
            assert_eq!(arred2(v), esperado, "arred2({v})");
        }
    }

    #[test]
    fn nao_finito_vira_zero() {
        assert_eq!(arred2(f64::NAN), 0.0);
        assert_eq!(arred2(f64::INFINITY), 0.0);
    }

    #[test]
    fn format_padrao_do_rust_erra_e_fmt_dec_acerta() {
        assert_eq!(format!("{:.2}", 1.005_f64), "1.00");
        assert_eq!(format!("{:.2}", 0.125_f64), "0.12"); // desempate para o par
        assert_eq!(fmt_dec(1.005, 2), "1.01");
        assert_eq!(fmt_dec(0.125, 2), "0.13");
        assert_eq!(fmt_dec(12.0, 4), "12.0000");
        assert_eq!(fmt_dec(3.5, 3), "3.500");
        assert_eq!(fmt_dec(-0.001, 2), "0.00");
    }

    #[test]
    fn caso_pis_da_base_com_icms_excluido() {
        // 3,5 × 602,6097: vProd 2109.13, vICMS 12% 253.10 ⇒ vBC PIS 1856.03 (não 1856.04).
        let v_prod = arred2(3.5 * 602.6097);
        let v_icms = arred2(v_prod * 12.0 / 100.0);
        assert_eq!(v_prod, 2109.13);
        assert_eq!(v_icms, 253.10);
        assert_eq!(fmt_dec(v_prod - v_icms, 2), "1856.03");
    }

    #[test]
    fn valor_unitario_mantem_as_casas_informadas() {
        assert_eq!(fmt_dec_ate(602.6097, 2, 10), "602.6097");
        assert_eq!(fmt_dec_ate(10.0, 2, 10), "10.00");
        assert_eq!(fmt_dec_ate(0.0049, 2, 10), "0.0049");
        assert_eq!(fmt_dec_ate(1.5, 2, 10), "1.50");
    }

    #[test]
    fn decimal_meio_para_cima() {
        assert_eq!(arred_decimal(Decimal::new(125, 3), 2), Decimal::new(13, 2));
        // round_dp puro (bancário) daria 0.12 — era o que o rateio usava.
        assert_eq!(Decimal::new(125, 3).round_dp(2), Decimal::new(12, 2));
    }
}
