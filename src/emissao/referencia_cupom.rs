//! Vedação de NF-e de saída que referencia cupom fiscal (NFC-e modelo 65 ou CF-e SAT modelo 59).
//!
//! Base: Ajuste SINIEF 32/2025 (altera o Ajuste SINIEF 07/2005 — "É vedada a emissão de NF-e de
//! saída que faça referência a uma NFC-e, excetuando-se a emissão de NF-e complementar"), com o
//! início adiado para **14/12/2026** pelo Ajuste SINIEF 29/2026. A SEFAZ aplica pelas regras
//! BA02-35 (rejeição 679) e I08-180 (rejeição 375, CFOP 5.929/6.929) da NT 2026.002 v1.11; a
//! homologação já rejeita. Devolução (`finNFe = 4`) e complementar (`finNFe = 2`) continuam podendo
//! referenciar a NFC-e.
//!
//! A guarda roda antes de assinar, para o consumidor receber uma explicação em vez da rejeição.
//! As funções são públicas porque quem monta a nota (gravisServer, retaguarda) recusa antes de
//! juntar os dados — mesma regra, uma fonte só.

use chrono::NaiveDate;

use crate::error::{DfeError, Result};
use crate::tipos::Ide;

/// Primeiro dia em que a SEFAZ de produção rejeita a referência (Ajuste SINIEF 29/2026).
pub const VEDACAO_REFERENCIA_CUPOM_PRODUCAO: NaiveDate = match NaiveDate::from_ymd_opt(2026, 12, 14)
{
    Some(d) => d,
    None => panic!("data inválida"),
};

/// Texto de balcão da recusa — o mesmo para a crate, o servidor e as telas.
pub const MSG_REFERENCIA_CUPOM_VEDADA: &str = "Desde 14/12/2026 a SEFAZ não aceita NF-e que referencia \
NFC-e ou cupom SAT (Ajuste SINIEF 32/2025; rejeição 679). A venda já tem o documento fiscal dela: a \
cobrança segue como fatura, com a lista das notas. Para o cliente que precisa de NF-e, emita a NF-e \
(modelo 55) na hora da venda, no lugar da NFC-e.";

/// A chave é de NFC-e (65) ou CF-e SAT (59)? O modelo está nas posições 21–22 da chave.
pub fn chave_e_de_cupom(chave: &str) -> bool {
    let c = chave.trim();
    c.len() == 44 && c.is_ascii() && matches!(&c[20..22], "65" | "59")
}

/// A vedação já vale neste ambiente e nesta data? Homologação: sempre (a SEFAZ já rejeita).
/// Produção: a partir de [`VEDACAO_REFERENCIA_CUPOM_PRODUCAO`].
pub fn vedacao_referencia_cupom_ativa(tp_amb: u8, hoje: NaiveDate) -> bool {
    tp_amb != 1 || hoje >= VEDACAO_REFERENCIA_CUPOM_PRODUCAO
}

/// A finalidade/tipo da nota está sujeita à vedação? NF-e (55) de **saída** que não é
/// complementar (2) nem devolução (4).
pub fn finalidade_sujeita_a_vedacao(mod_: u32, tp_nf: u8, fin_nfe: u8) -> bool {
    mod_ == 55 && tp_nf == 1 && !matches!(fin_nfe, 2 | 4)
}

/// Recusa a nota que a SEFAZ rejeitaria por referenciar cupom. Pura: `hoje` vem de quem chama.
pub fn validar_referencias(ide: &Ide, referencias: &[String], hoje: NaiveDate) -> Result<()> {
    if !finalidade_sujeita_a_vedacao(ide.mod_, ide.tp_nf, ide.fin_nfe)
        || !vedacao_referencia_cupom_ativa(ide.tp_amb, hoje)
    {
        return Ok(());
    }
    let cupons: Vec<&str> = referencias
        .iter()
        .map(|r| r.trim())
        .filter(|r| chave_e_de_cupom(r))
        .collect();
    if cupons.is_empty() {
        return Ok(());
    }
    Err(DfeError::Validacao(format!(
        "{} Chave(s) referenciada(s): {}.",
        MSG_REFERENCIA_CUPOM_VEDADA,
        cupons.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NFCE: &str = "35261148510510000135650010000002781297742703";
    const SAT: &str = "35261148510510000135590010000002781297742703";
    const NFE: &str = "35261148510510000135550010000002781297742703";
    /// Emitente com CNPJ alfanumérico: o modelo continua nas posições 21–22.
    const NFCE_ALFANUM: &str = "352611KNB52691000160650010000002781297742703";

    fn d(a: i32, m: u32, dia: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(a, m, dia).unwrap()
    }

    fn ide(tp_amb: u8, fin_nfe: u8, tp_nf: u8, mod_: u32) -> Ide {
        Ide {
            tp_amb,
            fin_nfe,
            tp_nf,
            mod_,
            ..Default::default()
        }
    }

    fn refs(chaves: &[&str]) -> Vec<String> {
        chaves.iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn reconhece_cupom_pelo_modelo_da_chave() {
        assert!(chave_e_de_cupom(NFCE));
        assert!(chave_e_de_cupom(SAT));
        assert!(chave_e_de_cupom(NFCE_ALFANUM));
        assert!(!chave_e_de_cupom(NFE));
        assert!(!chave_e_de_cupom("65"));
        assert!(!chave_e_de_cupom(""));
    }

    #[test]
    fn producao_so_a_partir_de_14_12_2026() {
        assert!(!vedacao_referencia_cupom_ativa(1, d(2026, 12, 13)));
        assert!(vedacao_referencia_cupom_ativa(1, d(2026, 12, 14)));
        assert!(vedacao_referencia_cupom_ativa(1, d(2027, 1, 5)));
        // homologação já rejeita
        assert!(vedacao_referencia_cupom_ativa(2, d(2026, 11, 17)));
    }

    #[test]
    fn nfe_normal_de_saida_com_nfce_e_recusada_com_texto_de_balcao() {
        let err = validar_referencias(&ide(1, 1, 1, 55), &refs(&[NFE, NFCE]), d(2026, 12, 14))
            .unwrap_err();
        let DfeError::Validacao(msg) = err else {
            panic!("esperava Validacao")
        };
        assert!(msg.contains("Ajuste SINIEF 32/2025"));
        assert!(msg.contains(NFCE));
        assert!(!msg.contains(NFE), "só lista as chaves de cupom");
    }

    #[test]
    fn cupom_sat_tambem_e_recusado() {
        assert!(validar_referencias(&ide(2, 1, 1, 55), &refs(&[SAT]), d(2026, 11, 17)).is_err());
    }

    #[test]
    fn devolucao_e_complementar_continuam_referenciando_nfce() {
        for fin in [2, 4] {
            assert!(
                validar_referencias(&ide(1, fin, 1, 55), &refs(&[NFCE]), d(2027, 1, 5)).is_ok(),
                "finNFe {fin}"
            );
        }
    }

    #[test]
    fn referencia_a_nfe_entrada_e_nfce_antes_da_data_passam() {
        assert!(validar_referencias(&ide(1, 1, 1, 55), &refs(&[NFE]), d(2027, 1, 5)).is_ok());
        // nota de entrada não é "NF-e de saída"
        assert!(validar_referencias(&ide(1, 1, 0, 55), &refs(&[NFCE]), d(2027, 1, 5)).is_ok());
        // produção antes de 14/12/2026
        assert!(validar_referencias(&ide(1, 1, 1, 55), &refs(&[NFCE]), d(2026, 12, 13)).is_ok());
        // sem referência nenhuma
        assert!(validar_referencias(&ide(1, 1, 1, 55), &[], d(2027, 1, 5)).is_ok());
    }
}
