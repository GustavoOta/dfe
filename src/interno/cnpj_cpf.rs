/// Remove a formatação de um CNPJ/CPF (pontos, barra, traço, espaços),
/// **preservando letras** e normalizando para maiúsculas.
///
/// A partir de julho/2026 o CNPJ passa a ser alfanumérico (12 posições
/// alfanuméricas + 2 dígitos verificadores numéricos), por isso a limpeza
/// não pode descartar letras como um `filter(is_ascii_digit)` faria.
///
/// # Exemplo
///
/// ```
/// use dfe::sanitize_cnpj;
///
/// assert_eq!(sanitize_cnpj("12.ABC.345/01DE-35"), "12ABC34501DE35");
/// assert_eq!(sanitize_cnpj("11.222.333/0001-81"), "11222333000181");
/// ```
pub fn sanitize_cnpj(cnpj: &str) -> String {
    cnpj.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Aplica a máscara `00.000.000/0000-00` a um CNPJ numérico **ou alfanumérico**.
///
/// Se a entrada, depois de limpa, não tiver exatamente 14 posições, devolve o
/// valor original inalterado.
///
/// # Exemplo
///
/// ```
/// use dfe::format_cnpj;
///
/// assert_eq!(format_cnpj("12ABC34501DE35"), "12.ABC.345/01DE-35");
/// assert_eq!(format_cnpj("11222333000181"), "11.222.333/0001-81");
/// ```
pub fn format_cnpj(cnpj: &str) -> String {
    let d = sanitize_cnpj(cnpj);
    if d.len() != 14 {
        return cnpj.to_string();
    }
    format!(
        "{}.{}.{}/{}-{}",
        &d[0..2],
        &d[2..5],
        &d[5..8],
        &d[8..12],
        &d[12..14]
    )
}

/// Formata um documento como CNPJ (14 posições, alfanumérico) ou CPF (11
/// dígitos). Helper interno compartilhado pelos geradores de DANFE/ESC-POS.
///
/// Mantém letras (CNPJ alfanumérico) e cai para o valor original quando o
/// comprimento não corresponde a um CPF ou CNPJ válido.
pub(crate) fn format_cnpj_cpf(doc: &str) -> String {
    let d = sanitize_cnpj(doc);
    match d.len() {
        14 => format!(
            "{}.{}.{}/{}-{}",
            &d[0..2],
            &d[2..5],
            &d[5..8],
            &d[8..12],
            &d[12..14]
        ),
        11 => format!("{}.{}.{}-{}", &d[0..3], &d[3..6], &d[6..9], &d[9..11]),
        _ => doc.to_string(),
    }
}

/// Valida o dígito verificador de um **CNPJ**, numérico ou **alfanumérico**.
///
/// Aceita o número com ou sem formatação (pontos, barras, traço) e com letras
/// em maiúsculas ou minúsculas. As 12 primeiras posições podem ser
/// alfanuméricas (`0-9`, `A-Z`); os 2 dígitos verificadores são sempre
/// numéricos. O valor de cada posição no cálculo do módulo 11 é o código ASCII
/// do caractere menos 48 (`'0'`→0 … `'9'`→9, `'A'`→17 … `'Z'`→42), conforme a
/// regra da Receita Federal — o que torna o cálculo **retrocompatível** com os
/// CNPJs puramente numéricos atuais.
///
/// Retorna `true` somente se os dois dígitos verificadores estiverem corretos
/// e o número não for formado por posições todas iguais (ex.: `00000000000000`).
///
/// # Exemplo
///
/// ```
/// use dfe::validate_cnpj;
///
/// assert!(validate_cnpj("11.222.333/0001-81"));    // numérico (legado)
/// assert!(validate_cnpj("11222333000181"));
/// assert!(validate_cnpj("12.ABC.345/01DE-35"));    // alfanumérico (novo)
/// assert!(!validate_cnpj("11222333000182"));       // DV errado
/// assert!(!validate_cnpj("00000000000000"));       // todos iguais
/// ```
pub fn validate_cnpj(cnpj: &str) -> bool {
    // Mantém apenas caracteres alfanuméricos, em maiúsculas.
    let chars: Vec<char> = cnpj
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if chars.len() != 14 {
        return false;
    }
    // Os dois dígitos verificadores (posições 12 e 13) são sempre numéricos.
    if !chars[12..].iter().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Valor de cada posição = código ASCII - 48
    // ('0'->0 .. '9'->9, 'A'->17 .. 'Z'->42), conforme regra da RFB.
    let values: Vec<u64> = chars.iter().map(|c| (*c as u64) - 48).collect();

    // CNPJ com todas as posições iguais é inválido (ex.: 00000000000000).
    if values.windows(2).all(|w| w[0] == w[1]) {
        return false;
    }

    // Pesos: ciclo 2-9 da direita para a esquerda.
    // (0..len).rev() mapeia índice i → peso (2 + i % 8), gerando a sequência
    // correta da esquerda para a direita (ex.: [5,4,3,2,9,8,7,6,5,4,3,2] para len=12).
    let check = |d: &[u64], len: usize| -> u64 {
        let weights: Vec<u64> = (0..len).rev().map(|i| 2 + (i % 8) as u64).collect();
        let sum: u64 = d[..len].iter().zip(weights.iter()).map(|(a, b)| a * b).sum();
        let rem = sum % 11;
        if rem < 2 {
            0
        } else {
            11 - rem
        }
    };
    values[12] == check(&values, 12) && values[13] == check(&values, 13)
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
    fn cnpj_alfanumerico_valido() {
        // Exemplo oficial da Receita Federal (CNPJ alfanumérico).
        assert!(validate_cnpj("12ABC34501DE35"));
        assert!(validate_cnpj("12.ABC.345/01DE-35"));
        // Letras minúsculas devem ser aceitas (normalizadas para maiúsculas).
        assert!(validate_cnpj("12abc34501de35"));
    }

    #[test]
    fn cnpj_invalido() {
        assert!(!validate_cnpj("11222333000182")); // DV errado
        assert!(!validate_cnpj("12ABC34501DE34"));  // DV alfanumérico errado
        assert!(!validate_cnpj("00000000000000")); // todos iguais
        assert!(!validate_cnpj("1234567"));        // tamanho errado
        assert!(!validate_cnpj("12ABC34501DEXY")); // DV não numérico
    }

    #[test]
    fn sanitize_e_format() {
        assert_eq!(sanitize_cnpj("12.ABC.345/01DE-35"), "12ABC34501DE35");
        assert_eq!(format_cnpj("12ABC34501DE35"), "12.ABC.345/01DE-35");
        assert_eq!(format_cnpj("11222333000181"), "11.222.333/0001-81");
        assert_eq!(format_cnpj_cpf("12ABC34501DE35"), "12.ABC.345/01DE-35");
        assert_eq!(format_cnpj_cpf("52998224725"), "529.982.247-25");
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
