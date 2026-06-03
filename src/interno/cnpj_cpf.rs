/// Valida o dígito verificador de um **CNPJ**.
///
/// Aceita o número com ou sem formatação (pontos, barras, traço).
/// Retorna `true` somente se os dois dígitos verificadores estiverem corretos
/// e o número não for formado por dígitos todos iguais (ex.: `00000000000000`).
///
/// # Exemplo
///
/// ```
/// use dfe::validate_cnpj;
///
/// assert!(validate_cnpj("11.222.333/0001-81"));
/// assert!(validate_cnpj("11222333000181"));
/// assert!(!validate_cnpj("11222333000182")); // DV errado
/// assert!(!validate_cnpj("00000000000000")); // todos iguais
/// ```
pub fn validate_cnpj(cnpj: &str) -> bool {
    let digits: Vec<u64> = cnpj.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| (c as u64) - 48)
        .collect();
    if digits.len() != 14 { return false; }
    // CNPJ com todos os dígitos iguais é inválido (ex.: 00000000000000)
    if digits.windows(2).all(|w| w[0] == w[1]) { return false; }

    // Pesos: ciclo 2-9 da direita para a esquerda.
    // (0..len).rev() mapeia índice i → peso (2 + i % 8), gerando a sequência
    // correta da esquerda para a direita (ex.: [5,4,3,2,9,8,7,6,5,4,3,2] para len=12).
    let check = |d: &[u64], len: usize| -> u64 {
        let weights: Vec<u64> = (0..len).rev().map(|i| 2 + (i % 8) as u64).collect();
        let sum: u64 = d[..len].iter().zip(weights.iter()).map(|(a, b)| a * b).sum();
        let rem = sum % 11;
        if rem < 2 { 0 } else { 11 - rem }
    };
    digits[12] == check(&digits, 12) && digits[13] == check(&digits, 13)
}

/// Valida o dígito verificador de um **CPF**.
///
/// Aceita o número com ou sem formatação.
/// Retorna `true` somente se os dois dígitos verificadores estiverem corretos
/// e o número não for formado por dígitos todos iguais (ex.: `11111111111`).
///
/// # Exemplo
///
/// ```
/// use dfe::validate_cpf;
///
/// assert!(validate_cpf("529.982.247-25"));
/// assert!(validate_cpf("52998224725"));
/// assert!(!validate_cpf("52998224726")); // DV errado
/// assert!(!validate_cpf("11111111111")); // todos iguais
/// ```
pub fn validate_cpf(cpf: &str) -> bool {
    let digits: Vec<u64> = cpf.chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| (c as u64) - 48)
        .collect();
    if digits.len() != 11 {
        return false;
    }
    if digits.windows(2).all(|w| w[0] == w[1]) {
        return false;
    }
    let check = |d: &[u64], len: usize| -> u64 {
        let sum: u64 = d[..len].iter().enumerate().map(|(i, &v)| v * (len as u64 + 1 - i as u64)).sum();
        let rem = sum % 11;
        if rem < 2 { 0 } else { 11 - rem }
    };
    digits[9] == check(&digits, 9) && digits[10] == check(&digits, 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cnpj_valido() {
        assert!(validate_cnpj("11222333000181"));
        assert!(validate_cnpj("11.222.333/0001-81"));
    }

    #[test]
    fn cnpj_invalido() {
        assert!(!validate_cnpj("11222333000182")); // DV errado
        assert!(!validate_cnpj("00000000000000")); // todos iguais
        assert!(!validate_cnpj("1234567"));        // tamanho errado
    }

    #[test]
    fn cpf_valido() {
        assert!(validate_cpf("52998224725"));
        assert!(validate_cpf("529.982.247-25"));
    }

    #[test]
    fn cpf_invalido() {
        assert!(!validate_cpf("52998224726")); // DV errado
        assert!(!validate_cpf("11111111111")); // todos iguais
        assert!(!validate_cpf("123456"));      // tamanho errado
    }
}
