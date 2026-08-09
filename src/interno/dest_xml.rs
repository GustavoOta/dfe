use crate::error::{DfeError, Result};
use crate::tipos::{Dest, Ide};
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::io::Cursor;

pub struct DestTAG;
impl DestTAG {
    pub fn build(dest: &Option<Dest>, ide: &Ide) -> Result<String> {
        let mod_ = ide.mod_;
        let ambiente = ide.tp_amb;
        match dest {
            Some(d) => match mod_ {
                55 => dest55(d),
                65 => dest65(d, &ambiente),
                _ => Err(DfeError::Validacao(format!("Modalidade de NFe inválida: {}", mod_))),
            },
            None => Ok(String::new()),
        }
    }
}

fn dest55(dest: &Dest) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer
        .create_element("dest")
        .write_inner_content(|writer| {
            if let Some(cnpj) = &dest.cnpj {
                writer.create_element("CNPJ").write_text_content(BytesText::new(cnpj))?;
            }
            if let Some(cpf) = &dest.cpf {
                writer.create_element("CPF").write_text_content(BytesText::new(cpf))?;
            }
            if let Some(id_estrangeiro) = &dest.id_estrangeiro {
                writer.create_element("idEstrangeiro").write_text_content(BytesText::new(id_estrangeiro))?;
            }
            if let Some(x_nome) = &dest.x_nome {
                writer.create_element("xNome").write_text_content(BytesText::new(x_nome))?;
            }

            if dest.x_lgr.is_some() || dest.nro.is_some() || dest.x_bairro.is_some()
                || dest.c_mun.is_some() || dest.x_mun.is_some() || dest.uf.is_some() || dest.cep.is_some()
            {
                writer.create_element("enderDest").write_inner_content(|writer| {
                    if let Some(x_lgr) = &dest.x_lgr { writer.create_element("xLgr").write_text_content(BytesText::new(x_lgr))?; }
                    if let Some(nro) = &dest.nro { writer.create_element("nro").write_text_content(BytesText::new(nro))?; }
                    if let Some(x_bairro) = &dest.x_bairro { writer.create_element("xBairro").write_text_content(BytesText::new(x_bairro))?; }
                    if let Some(c_mun) = &dest.c_mun { writer.create_element("cMun").write_text_content(BytesText::new(c_mun))?; }
                    if let Some(x_mun) = &dest.x_mun { writer.create_element("xMun").write_text_content(BytesText::new(x_mun))?; }
                    if let Some(uf) = &dest.uf { writer.create_element("UF").write_text_content(BytesText::new(uf))?; }
                    if let Some(cep) = &dest.cep { writer.create_element("CEP").write_text_content(BytesText::new(cep))?; }
                    if let Some(c_pais) = &dest.c_pais { writer.create_element("cPais").write_text_content(BytesText::new(c_pais))?; }
                    if let Some(x_pais) = &dest.x_pais { writer.create_element("xPais").write_text_content(BytesText::new(x_pais))?; }
                    if let Some(fone) = &dest.fone { writer.create_element("fone").write_text_content(BytesText::new(fone))?; }
                    Ok(())
                })?;
            }

            if let Some(ind_ie_dest) = dest.ind_ie_dest {
                writer.create_element("indIEDest").write_text_content(BytesText::new(&ind_ie_dest.to_string()))?;
            }
            if let Some(ie) = &dest.ie {
                if dest.ind_ie_dest == Some(1) {
                    writer.create_element("IE").write_text_content(BytesText::new(ie))?;
                }
            }
            if let Some(isuf) = &dest.isuf { writer.create_element("ISUF").write_text_content(BytesText::new(isuf))?; }
            if let Some(im) = &dest.im { writer.create_element("IM").write_text_content(BytesText::new(im))?; }
            if let Some(email) = &dest.email { writer.create_element("email").write_text_content(BytesText::new(email))?; }

            Ok(())
        })?;
    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

fn dest65(dest: &Dest, ambiente: &u8) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer
        .create_element("dest")
        .write_inner_content(|writer| {
            if let Some(cnpj) = &dest.cnpj { writer.create_element("CNPJ").write_text_content(BytesText::new(cnpj))?; }
            if let Some(cpf) = &dest.cpf { writer.create_element("CPF").write_text_content(BytesText::new(cpf))?; }
            if let Some(id_estrangeiro) = &dest.id_estrangeiro { writer.create_element("idEstrangeiro").write_text_content(BytesText::new(id_estrangeiro))?; }
            if let Some(x_nome) = &dest.x_nome {
                let x_nome = x_nome_validate(x_nome, ambiente).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
                })?;
                writer.create_element("xNome").write_text_content(BytesText::new(x_nome.as_str()))?;
            }

            let ind_ie_dest = dest.ind_ie_dest.unwrap_or(9);
            writer.create_element("indIEDest").write_text_content(BytesText::new(&ind_ie_dest.to_string()))?;

            Ok(())
        })?;

    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

fn x_nome_validate(x_nome: &str, ambiente: &u8) -> Result<String> {
    let mut x_nome = x_nome.trim();
    let len = x_nome.chars().count();
    if len < 2 || len > 60 {
        return Err(DfeError::Validacao(format!(
            "O campo xNome deve ter entre 2 e 60 caracteres. Tamanho atual: {}",
            len
        )));
    }
    if *ambiente == 2 {
        x_nome = "NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL";
    }
    Ok(x_nome.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tipos::{Dest, Ide};

    /// `Ide` mínimo para o `DestTAG::build` — só `mod_` e `tp_amb` importam aqui.
    fn ide(mod_: u32, tp_amb: u8) -> Ide {
        Ide {
            mod_,
            tp_amb,
            ..Default::default()
        }
    }

    // ─── NF-e (modelo 55) ────────────────────────────────────────────────────

    /// Destinatário **pessoa física (CPF)** não contribuinte: deve sair `<CPF>` e
    /// `<indIEDest>9`, **sem** `<CNPJ>` e **sem** `<IE>`.
    #[test]
    fn dest55_cpf_gera_tag_cpf_sem_cnpj_nem_ie() {
        let dest = Dest {
            cpf: Some("52998224725".to_string()),
            x_nome: Some("Fulano de Tal".to_string()),
            ind_ie_dest: Some(9),
            // IE preenchida "por engano" NÃO deve ser emitida p/ não contribuinte.
            ie: Some("123456789".to_string()),
            ..Default::default()
        };
        let xml = DestTAG::build(&Some(dest), &ide(55, 2)).unwrap();

        assert!(xml.contains("<CPF>52998224725</CPF>"), "esperava <CPF>: {xml}");
        assert!(!xml.contains("<CNPJ>"), "não deveria ter <CNPJ>: {xml}");
        assert!(xml.contains("<indIEDest>9</indIEDest>"), "esperava indIEDest 9: {xml}");
        assert!(!xml.contains("<IE>"), "não deveria emitir <IE> p/ indIEDest != 1: {xml}");
    }

    /// Regressão: destinatário **pessoa jurídica (CNPJ)** contribuinte de ICMS deve
    /// continuar saindo com `<CNPJ>`, `<indIEDest>1` e `<IE>`.
    #[test]
    fn dest55_cnpj_contribuinte_mantem_cnpj_ie() {
        let dest = Dest {
            cnpj: Some("11222333000181".to_string()),
            x_nome: Some("Empresa Ltda".to_string()),
            ind_ie_dest: Some(1),
            ie: Some("110042490114".to_string()),
            ..Default::default()
        };
        let xml = DestTAG::build(&Some(dest), &ide(55, 2)).unwrap();

        assert!(xml.contains("<CNPJ>11222333000181</CNPJ>"), "esperava <CNPJ>: {xml}");
        assert!(!xml.contains("<CPF>"), "não deveria ter <CPF>: {xml}");
        assert!(xml.contains("<indIEDest>1</indIEDest>"), "esperava indIEDest 1: {xml}");
        assert!(xml.contains("<IE>110042490114</IE>"), "esperava <IE> p/ contribuinte: {xml}");
    }

    // ─── NFC-e (modelo 65) ───────────────────────────────────────────────────

    /// NFC-e para **CPF**: deve sair `<CPF>`; em homologação (tp_amb 2) o `xNome`
    /// é substituído pela frase padrão sem valor fiscal.
    #[test]
    fn dest65_cpf_gera_cpf_e_xnome_homologacao() {
        let dest = Dest {
            cpf: Some("52998224725".to_string()),
            x_nome: Some("Consumidor Final".to_string()),
            ind_ie_dest: Some(9),
            ..Default::default()
        };
        let xml = DestTAG::build(&Some(dest), &ide(65, 2)).unwrap();

        assert!(xml.contains("<CPF>52998224725</CPF>"), "esperava <CPF>: {xml}");
        assert!(!xml.contains("<CNPJ>"), "não deveria ter <CNPJ>: {xml}");
        assert!(
            xml.contains("HOMOLOGACAO"),
            "xNome deveria virar a frase de homologação: {xml}"
        );
        assert!(xml.contains("<indIEDest>9</indIEDest>"), "esperava indIEDest 9: {xml}");
    }
}
