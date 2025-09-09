pub mod models;
use crate::nfe::xml_rules::ide::models::Ide;
use anyhow::Error;
use models::Dest;

use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::io::Cursor;

pub struct DestTAG;
impl DestTAG {
    pub fn build(dest: &Option<Dest>, ide: &Ide) -> Result<String, Error> {
        let mod_ = ide.mod_;
        let ambiente = ide.tp_amb;
        match dest {
            Some(d) => match mod_ {
                55 => dest55(d),
                65 => dest65(d, &ambiente),
                _ => {
                    let error_message = format!("Modalidade de NFe inválida: {}", mod_);
                    Err(Error::msg(error_message))
                }
            },
            None => {
                // Se não houver destinatário, retornar uma string vazia
                Ok(String::new())
            }
        }
    }
}

fn dest55(dest: &Dest) -> Result<String, Error> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // writes <dest>...content...</dest>
    writer
        .create_element("dest")
        .write_inner_content(|writer| {
            if let Some(cnpj) = &dest.cnpj {
                writer
                    .create_element("CNPJ")
                    .write_text_content(BytesText::new(cnpj))?;
            }
            if let Some(cpf) = &dest.cpf {
                writer
                    .create_element("CPF")
                    .write_text_content(BytesText::new(cpf))?;
            }
            if let Some(id_estrangeiro) = &dest.id_estrangeiro {
                writer
                    .create_element("idEstrangeiro")
                    .write_text_content(BytesText::new(id_estrangeiro))?;
            }
            if let Some(x_nome) = &dest.x_nome {
                writer
                    .create_element("xNome")
                    .write_text_content(BytesText::new(x_nome))?;
            }

            // Para NFe (modelo 55), incluir enderDest se informado
            if dest.x_lgr.is_some()
                || dest.nro.is_some()
                || dest.x_bairro.is_some()
                || dest.c_mun.is_some()
                || dest.x_mun.is_some()
                || dest.uf.is_some()
                || dest.cep.is_some()
            {
                writer
                    .create_element("enderDest")
                    .write_inner_content(|writer| {
                        if let Some(x_lgr) = &dest.x_lgr {
                            writer
                                .create_element("xLgr")
                                .write_text_content(BytesText::new(x_lgr))?;
                        }
                        if let Some(nro) = &dest.nro {
                            writer
                                .create_element("nro")
                                .write_text_content(BytesText::new(nro))?;
                        }
                        if let Some(x_bairro) = &dest.x_bairro {
                            writer
                                .create_element("xBairro")
                                .write_text_content(BytesText::new(x_bairro))?;
                        }
                        if let Some(c_mun) = &dest.c_mun {
                            writer
                                .create_element("cMun")
                                .write_text_content(BytesText::new(c_mun))?;
                        }
                        if let Some(x_mun) = &dest.x_mun {
                            writer
                                .create_element("xMun")
                                .write_text_content(BytesText::new(x_mun))?;
                        }
                        if let Some(uf) = &dest.uf {
                            writer
                                .create_element("UF")
                                .write_text_content(BytesText::new(uf))?;
                        }
                        if let Some(cep) = &dest.cep {
                            writer
                                .create_element("CEP")
                                .write_text_content(BytesText::new(cep))?;
                        }
                        if let Some(c_pais) = &dest.c_pais {
                            writer
                                .create_element("cPais")
                                .write_text_content(BytesText::new(c_pais))?;
                        }
                        if let Some(x_pais) = &dest.x_pais {
                            writer
                                .create_element("xPais")
                                .write_text_content(BytesText::new(x_pais))?;
                        }
                        if let Some(fone) = &dest.fone {
                            writer
                                .create_element("fone")
                                .write_text_content(BytesText::new(fone))?;
                        }
                        Ok(())
                    })?;
            }

            // indIEDest é obrigatório para NFe
            if let Some(ind_ie_dest) = dest.ind_ie_dest {
                writer
                    .create_element("indIEDest")
                    .write_text_content(BytesText::new(&ind_ie_dest.to_string()))?;
            }

            // IE apenas se ind_ie_dest for 1 (Contribuinte ICMS)
            if let Some(ie) = &dest.ie {
                if dest.ind_ie_dest == Some(1) {
                    writer
                        .create_element("IE")
                        .write_text_content(BytesText::new(ie))?;
                }
            }

            // ISUF se informado
            if let Some(isuf) = &dest.isuf {
                writer
                    .create_element("ISUF")
                    .write_text_content(BytesText::new(isuf))?;
            }

            // IM se informado
            if let Some(im) = &dest.im {
                writer
                    .create_element("IM")
                    .write_text_content(BytesText::new(im))?;
            }

            // Email se informado
            if let Some(email) = &dest.email {
                writer
                    .create_element("email")
                    .write_text_content(BytesText::new(email))?;
            }

            Ok(())
        })
        .map_err(|e| Error::new(e))?;
    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

fn dest65(dest: &Dest, ambiente: &u8) -> Result<String, Error> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // writes <dest>...content...</dest>
    writer
        .create_element("dest")
        .write_inner_content(|writer| {
            if let Some(cnpj) = &dest.cnpj {
                writer
                    .create_element("CNPJ")
                    .write_text_content(BytesText::new(cnpj))?;
            }
            if let Some(cpf) = &dest.cpf {
                writer
                    .create_element("CPF")
                    .write_text_content(BytesText::new(cpf))?;
            }
            if let Some(id_estrangeiro) = &dest.id_estrangeiro {
                writer
                    .create_element("idEstrangeiro")
                    .write_text_content(BytesText::new(id_estrangeiro))?;
            }
            if let Some(x_nome) = &dest.x_nome {
                let x_nome = match x_nome_validate(&x_nome, &ambiente) {
                    Ok(validated) => validated,
                    Err(e) => {
                        let error = format!("Erro na validação do campo xNome: {}", e);
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, error));
                    }
                };
                writer
                    .create_element("xNome")
                    .write_text_content(BytesText::new(x_nome.as_str()))?;
            }

            // Para NFCe (modelo 65), o indIEDest é obrigatório
            // NFCe sempre usa indIEDest=9 (Não Contribuinte) conforme legislação
            let ind_ie_dest = dest.ind_ie_dest.unwrap_or(9);
            writer
                .create_element("indIEDest")
                .write_text_content(BytesText::new(&ind_ie_dest.to_string()))?;

            Ok(())
        })
        .map_err(|e| Error::new(e))?;

    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

// validações **************************
fn x_nome_validate(x_nome: &str, ambiente: &u8) -> Result<String, Error> {
    let mut x_nome = x_nome.trim();
    let len = x_nome.chars().count();
    if len < 2 || len > 60 {
        let error_message = format!(
            "O campo xNome deve ter entre 2 e 60 caracteres. Tamanho atual: {}",
            len
        );
        return Err(Error::msg(error_message));
    }

    // Em ambiente de produção, não permitir caracteres especiais
    if *ambiente == 2 {
        x_nome = "NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL";
    }

    Ok(x_nome.to_string())
}
