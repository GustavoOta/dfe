use crate::error::{DfeError, Result};
use crate::interno::chave_acesso_props::{ChaveAcessoProps, ExtractComposition};
use crate::interno::dates::{get_current_month, get_current_year};
use rand::Rng;

pub struct ChaveAcesso {
    pub chave: String,
    pub dv: u8,
}

impl ChaveAcesso {
    pub fn gerar_chave_acesso(props: ChaveAcessoProps) -> ChaveAcesso {
        let serie = format!("{:0>3}", props.serie);
        let numero = format!("{:0>9}", props.numero);
        // CNPJ alfanumérico: remove a máscara e normaliza em maiúsculas preservando
        // as letras. CPF (numérico, 11 dígitos) continua preenchido à esquerda com zeros.
        let doc = format!(
            "{:0>14}",
            crate::interno::cnpj_cpf::sanitize_cnpj(&props.doc)
        );

        let mut chave = format!(
            "{}{}{}{}{}{}{}{}{}",
            props.uf,
            get_current_year(2),
            get_current_month(),
            doc,
            props.modelo,
            serie,
            numero,
            props.tp_emis,
            props.codigo_numerico,
        );

        let digito_verificador = ChaveAcesso::gerar_dv(&chave);
        match digito_verificador {
            Ok(dv) => {
                chave.push_str(&dv.to_string());
                ChaveAcesso { chave, dv }
            }
            Err(e) => panic!("Error generating DV: {}", e),
        }
    }

    pub fn gerar_dv(chave_sem_dv: &str) -> Result<u8> {
        let pesos = [2, 3, 4, 5, 6, 7, 8, 9];
        let mut soma = 0;

        for (i, c) in chave_sem_dv.chars().rev().enumerate() {
            let c = c.to_ascii_uppercase();
            if !c.is_ascii_alphanumeric() {
                return Err(DfeError::Xml(format!(
                    "Invalid character '{}' in chave_sem_dv",
                    c
                )));
            }
            // CNPJ alfanumérico (chave [0-9]{6}[0-9A-Z]{12}[0-9]{26}): o valor de cada
            // posição no módulo 11 é o código ASCII menos 48 ('0'->0..'9'->9,
            // 'A'->17..'Z'->42), conforme regra da Receita. Retrocompatível: para uma
            // chave totalmente numérica o resultado é idêntico ao cálculo anterior.
            let valor = (c as u32) - 48;
            soma += valor * pesos[i % 8];
        }

        let resto = soma % 11;
        let dv = if resto == 0 || resto == 1 { 0 } else { 11 - resto };

        Ok(dv as u8)
    }

    pub fn gerar_codigo_numerico(c_nf: Option<String>) -> String {
        if c_nf.is_some() {
            return c_nf.unwrap_or("Erro: c_nf não é String".to_string());
        }

        let mut rng = rand::thread_rng();
        let codigo_numerico: u32 = rng.gen_range(10000000..99999999);
        codigo_numerico.to_string()
    }

    pub fn extract_composition(chave: &str) -> Result<ExtractComposition> {
        let uf = &chave[0..2];
        let ano = &chave[2..4];
        let mes = &chave[4..6];
        let doc = &chave[6..20];
        let modelo = &chave[20..22];
        let serie = &chave[22..25];
        let numero = &chave[25..34];
        let tp_emis = &chave[34..35];
        let codigo_numerico = &chave[35..43];

        Ok(ExtractComposition {
            uf_code: uf.to_string(),
            ano: ano.to_string(),
            mes: mes.to_string(),
            doc: doc.to_string(),
            modelo: modelo.to_string(),
            serie: serie.to_string(),
            numero: numero.to_string(),
            tp_emis: tp_emis.to_string(),
            codigo_numerico: codigo_numerico.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DV de uma chave totalmente numérica deve coincidir com o cálculo antigo
    /// (dígito decimal × peso) — garante retrocompatibilidade.
    #[test]
    fn dv_numerico_retrocompat() {
        // 43 primeiras posições da chave do sample55 (sem o cDV).
        let chave = "3526030000000000019155001000000504100000000";
        assert_eq!(chave.len(), 43);

        let pesos = [2u32, 3, 4, 5, 6, 7, 8, 9];
        let mut soma = 0u32;
        for (i, c) in chave.chars().rev().enumerate() {
            soma += c.to_digit(10).unwrap() * pesos[i % 8];
        }
        let resto = soma % 11;
        let esperado = if resto == 0 || resto == 1 { 0u8 } else { (11 - resto) as u8 };

        assert_eq!(ChaveAcesso::gerar_dv(chave).unwrap(), esperado);
    }

    /// Chave com CNPJ alfanumérico nas posições 7-18 não deve mais gerar erro e
    /// produz um DV numérico válido (0-9).
    #[test]
    fn dv_alfanumerico_aceita_letras() {
        // cUF+AAMM(6) + CNPJ alfanumérico(14) + mod(2) + serie(3) + nNF(9) + tpEmis(1) + cNF(8)
        let chave = format!("{}{}{}{}{}{}{}", "352603", "12ABC34501DE35", "55", "001", "000000504", "1", "00000000");
        assert_eq!(chave.len(), 43);

        let dv = ChaveAcesso::gerar_dv(&chave).unwrap();
        assert!(dv <= 9);

        // Minúsculas devem ser normalizadas (mesmo DV que maiúsculas).
        let chave_min = chave.to_lowercase();
        assert_eq!(ChaveAcesso::gerar_dv(&chave_min).unwrap(), dv);
    }

    /// Caractere inválido (símbolo) ainda deve ser rejeitado.
    #[test]
    fn dv_rejeita_caractere_invalido() {
        assert!(ChaveAcesso::gerar_dv("3526031#ABC34501DE355500100000050410000000").is_err());
    }

    /// A geração completa da chave aceita CNPJ alfanumérico com máscara e produz
    /// 44 posições no padrão [0-9]{6}[0-9A-Z]{12}[0-9]{26}.
    #[test]
    fn gera_chave_com_cnpj_alfanumerico() {
        let ch = ChaveAcesso::gerar_chave_acesso(ChaveAcessoProps {
            uf: 35,
            doc: "12.ABC.345/01DE-35".to_string(),
            modelo: 55,
            serie: 1,
            numero: 504,
            tp_emis: 1,
            codigo_numerico: "00000000".to_string(),
        });
        assert_eq!(ch.chave.len(), 44);
        // posições 6-17 (12) = base do CNPJ alfanumérico
        assert_eq!(&ch.chave[6..18], "12ABC34501DE");
        // último caractere é o cDV (numérico)
        assert!(ch.chave.chars().last().unwrap().is_ascii_digit());
    }
}
