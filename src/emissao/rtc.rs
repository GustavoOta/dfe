use rust_decimal::{Decimal, RoundingStrategy};

/// Política numérica das calculadoras da RTC (ADR003 da Receita Federal).
pub(super) fn arred_rtc(valor: Decimal, casas: u32) -> Decimal {
    valor.round_dp_with_strategy(casas, RoundingStrategy::MidpointNearestEven)
}

pub(super) fn fmt_rtc(valor: Decimal, casas: u32) -> String {
    crate::arredondamento::fmt_decimal(arred_rtc(valor, casas), casas)
}
