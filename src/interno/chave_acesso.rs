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
        let doc = format!("{:0>14}", props.doc);

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
            let digito = c
                .to_digit(10)
                .ok_or_else(|| DfeError::Xml(format!("Invalid character '{}' in chave_sem_dv", c)))?;
            soma += digito * pesos[i % 8];
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
    #[test]
    fn test_chave_acesso() {}
}
