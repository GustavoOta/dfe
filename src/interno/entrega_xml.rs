use crate::error::Result;
use crate::tipos::Entrega;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::io::Cursor;

/// Monta o grupo `<entrega>` (local de entrega, TLocal). Emitido só quando informado
/// (delivery / `indPres = 4`); ausente → string vazia (mudança aditiva). A ordem dos
/// elementos segue o XSD; a validação de obrigatoriedade fica com o `is_xml_valid` (XSD).
pub struct EntregaTAG;

impl EntregaTAG {
    pub fn build(entrega: &Option<Entrega>) -> Result<String> {
        let e = match entrega {
            Some(e) => e,
            None => return Ok(String::new()),
        };

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        writer
            .create_element("entrega")
            .write_inner_content(|writer| {
                if let Some(cnpj) = &e.cnpj {
                    writer.create_element("CNPJ").write_text_content(BytesText::new(cnpj))?;
                }
                if let Some(cpf) = &e.cpf {
                    writer.create_element("CPF").write_text_content(BytesText::new(cpf))?;
                }
                if let Some(x_nome) = &e.x_nome {
                    writer.create_element("xNome").write_text_content(BytesText::new(x_nome))?;
                }
                if let Some(x_lgr) = &e.x_lgr {
                    writer.create_element("xLgr").write_text_content(BytesText::new(x_lgr))?;
                }
                if let Some(nro) = &e.nro {
                    writer.create_element("nro").write_text_content(BytesText::new(nro))?;
                }
                if let Some(x_cpl) = &e.x_cpl {
                    writer.create_element("xCpl").write_text_content(BytesText::new(x_cpl))?;
                }
                if let Some(x_bairro) = &e.x_bairro {
                    writer.create_element("xBairro").write_text_content(BytesText::new(x_bairro))?;
                }
                if let Some(c_mun) = &e.c_mun {
                    writer.create_element("cMun").write_text_content(BytesText::new(c_mun))?;
                }
                if let Some(x_mun) = &e.x_mun {
                    writer.create_element("xMun").write_text_content(BytesText::new(x_mun))?;
                }
                if let Some(uf) = &e.uf {
                    writer.create_element("UF").write_text_content(BytesText::new(uf))?;
                }
                if let Some(cep) = &e.cep {
                    writer.create_element("CEP").write_text_content(BytesText::new(cep))?;
                }
                if let Some(c_pais) = &e.c_pais {
                    writer.create_element("cPais").write_text_content(BytesText::new(c_pais))?;
                }
                if let Some(x_pais) = &e.x_pais {
                    writer.create_element("xPais").write_text_content(BytesText::new(x_pais))?;
                }
                if let Some(fone) = &e.fone {
                    writer.create_element("fone").write_text_content(BytesText::new(fone))?;
                }
                if let Some(email) = &e.email {
                    writer.create_element("email").write_text_content(BytesText::new(email))?;
                }
                if let Some(ie) = &e.ie {
                    writer.create_element("IE").write_text_content(BytesText::new(ie))?;
                }
                Ok(())
            })?;

        Ok(String::from_utf8(writer.into_inner().into_inner())?)
    }
}

#[cfg(test)]
mod tests {
    use super::EntregaTAG;
    use crate::tipos::Entrega;

    #[test]
    fn sem_entrega_produz_string_vazia() {
        assert_eq!(EntregaTAG::build(&None).unwrap(), "");
    }

    #[test]
    fn entrega_monta_grupo_na_ordem_do_xsd() {
        let e = Entrega {
            cpf: Some("12345678909".to_string()),
            x_lgr: Some("Rua A".to_string()),
            nro: Some("100".to_string()),
            x_bairro: Some("Centro".to_string()),
            c_mun: Some("3550308".to_string()),
            x_mun: Some("Sao Paulo".to_string()),
            uf: Some("SP".to_string()),
            cep: Some("01001000".to_string()),
            ..Default::default()
        };
        let xml = EntregaTAG::build(&Some(e)).unwrap();

        assert!(xml.starts_with("<entrega>"));
        assert!(xml.ends_with("</entrega>"));
        assert!(xml.contains("<CPF>12345678909</CPF>"));
        assert!(xml.contains("<cMun>3550308</cMun>"));
        assert!(xml.contains("<UF>SP</UF>"));
        // Ordem do XSD: CPF antes de xLgr; cMun antes de UF; UF antes de CEP.
        let pos = |t: &str| xml.find(t).unwrap();
        assert!(pos("<CPF>") < pos("<xLgr>"));
        assert!(pos("<xLgr>") < pos("<xBairro>"));
        assert!(pos("<cMun>") < pos("<UF>"));
        assert!(pos("<UF>") < pos("<CEP>"));
    }
}
