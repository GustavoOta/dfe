use crate::nfe::common::dates::{get_current_month, get_current_year};
use crate::nfe::types::chave_acesso_props::{ChaveAcessoProps, ExtractComposition};
use anyhow::{Error, Result};
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
        chave.push_str(&digito_verificador.to_string());
        ChaveAcesso {
            chave,
            dv: digito_verificador,
        }
    }

    pub fn gerar_dv(chave_sem_dv: &str) -> u8 {
        let pesos = [2, 3, 4, 5, 6, 7, 8, 9];
        let mut soma = 0;

        for (i, c) in chave_sem_dv.chars().rev().enumerate() {
            let digito = c.to_digit(10).unwrap();
            soma += digito * pesos[i % 8];
        }

        let resto = soma % 11;
        let dv = if resto == 0 || resto == 1 {
            0
        } else {
            11 - resto
        };

        dv as u8
    }

    /// Gera um código numérico de 8 dígitos
    pub fn gerar_codigo_numerico() -> String {
        let mut rng = rand::thread_rng();
        let codigo_numerico: u32 = rng.gen_range(10000000..99999999);
        codigo_numerico.to_string()
    }

    pub fn extract_composition(chave: &str) -> Result<ExtractComposition, Error> {
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
    //use super::*;

    #[test]
    fn test_chave_acesso() {
        /*  let init = Init {
            uf: "35".to_string(),
            ano: "24".to_string(),
            mes: "12".to_string(),
            cnpj: "54515633000161".to_string(),
            modelo: "55".to_string(),
            serie: "1".to_string(),
            numero: "1".to_string(),
            tp_emis: "1".to_string(),
            codigo_numerico: "00000001".to_string(),
        };

        let chave_acesso = ChaveAcesso::gerar_chave_acesso(init);
        println!("Chave de Acesso: {}", chave_acesso.chave);
        // asset num digts
        assert_eq!(chave_acesso.chave.len(), 44); */
    }
}
