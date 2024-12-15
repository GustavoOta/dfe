use super::Dest;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

/// Destinatário da NF-e
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename = "dest")]
pub struct DestProcess {
    #[serde(rename = "CPF", skip_serializing_if = "Option::is_none")]
    pub cpf: Option<CPFProcess>,
    #[serde(rename = "CNPJ", skip_serializing_if = "Option::is_none")]
    pub cnpj: Option<CNPJProcess>,
    #[serde(rename = "idEstrangeiro", skip_serializing_if = "Option::is_none")]
    pub id_estrangeiro: Option<IdEstrangeiroProcess>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CPFProcess {
    #[serde(rename = "CPF")]
    pub cpf: String,
    #[serde(rename = "xNome")]
    pub x_nome: String,
    #[serde(rename = "enderDest")]
    pub ender_dest: EnderDestProcess,
    /// Indicador da IE do Destinatário
    /// 1=Contribuinte ICMS (informar a IE do destinatário);
    /// 2=Contribuinte isento de Inscrição no cadastro de Contribuintes do ICMS;
    /// 9=Não Contribuinte, que pode ou não possuir Inscrição Estadual no Cadastro de Contribuintes do ICMS.
    /// Nota 1: No caso de NFC-e informar indIEDest=9 e não informar a tag IE do destinatário;
    /// Nota 2: No caso de operação com o Exterior informar indIEDest=9 e não informar a tag IE do destinatário;
    /// Nota 3: No caso de Contribuinte Isento de Inscrição (indIEDest=2), não informar a tag IE do destinatário.
    #[serde(rename = "indIEDest", skip_serializing_if = "Option::is_none")]
    pub ind_ie_dest: Option<u8>,
}

impl Default for CPFProcess {
    fn default() -> Self {
        CPFProcess {
            cpf: "".to_string(),
            x_nome: "".to_string(),
            ender_dest: EnderDestProcess {
                x_lgr: "".to_string(),
                nro: "".to_string(),
                x_bairro: "".to_string(),
                c_mun: "".to_string(),
                x_mun: "".to_string(),
                uf: "".to_string(),
                cep: "".to_string(),
                c_pais: "".to_string(),
                x_pais: "".to_string(),
                fone: None,
            },
            ind_ie_dest: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CNPJProcess {
    #[serde(rename = "CNPJ")]
    pub cnpj: String,
    #[serde(rename = "xNome")]
    pub x_nome: String,
    #[serde(rename = "enderDest")]
    pub ender_dest: EnderDestProcess,
    #[serde(rename = "indIEDest")]
    pub ind_ie_dest: Option<u8>,
    #[serde(rename = "IE", skip_serializing_if = "Option::is_none")]
    pub ie: Option<String>,
    #[serde(rename = "ISUF", skip_serializing_if = "Option::is_none")]
    pub isuf: Option<String>,
    #[serde(rename = "IM", skip_serializing_if = "Option::is_none")]
    pub im: Option<String>,
    #[serde(rename = "email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdEstrangeiroProcess {
    #[serde(rename = "idEstrangeiro")]
    pub id_estrangeiro: String,
    #[serde(rename = "xNome")]
    pub x_nome: String,
    #[serde(rename = "enderDest")]
    pub ender_dest: EnderDestProcess,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnderDestProcess {
    #[serde(rename = "xLgr")]
    pub x_lgr: String,
    pub nro: String,
    #[serde(rename = "xBairro")]
    pub x_bairro: String,
    #[serde(rename = "cMun")]
    pub c_mun: String,
    #[serde(rename = "xMun")]
    pub x_mun: String,
    #[serde(rename = "UF")]
    pub uf: String,
    #[serde(rename = "CEP")]
    pub cep: String,
    #[serde(rename = "cPais")]
    pub c_pais: String,
    #[serde(rename = "xPais")]
    pub x_pais: String,
    #[serde(rename = "fone", skip_serializing_if = "Option::is_none")]
    pub fone: Option<String>,
}

impl Default for EnderDestProcess {
    fn default() -> Self {
        EnderDestProcess {
            x_lgr: "".to_string(),
            nro: "".to_string(),
            x_bairro: "".to_string(),
            c_mun: "".to_string(),
            x_mun: "".to_string(),
            uf: "".to_string(),
            cep: "".to_string(),
            c_pais: "".to_string(),
            x_pais: "".to_string(),
            fone: None,
        }
    }
}

pub fn dest_process(dest: Dest) -> Result<DestProcess, Error> {
    // Se nenhuma das alternativas for verdadeira, é uma NFC-e
    let is_cpf = dest.cpf.is_some();
    let is_cnpj = dest.cnpj.is_some();
    let is_id_estrangeiro = dest.id_estrangeiro.is_some();

    // Se for CPF
    if is_cpf {
        let cpf = dest.cpf.unwrap();
        let cpf_process = CPFProcess {
            cpf,
            x_nome: dest
                .x_nome
                .ok_or_else(|| Error::msg("Campo xNome é obrigatório"))?,
            ender_dest: EnderDestProcess {
                x_lgr: dest
                    .x_lgr
                    .ok_or_else(|| Error::msg("Campo xLgr é obrigatório"))?,
                nro: dest
                    .nro
                    .ok_or_else(|| Error::msg("Campo nro é obrigatório"))?,
                x_bairro: dest
                    .x_bairro
                    .ok_or_else(|| Error::msg("Campo xBairro é obrigatório"))?,
                c_mun: dest
                    .c_mun
                    .ok_or_else(|| Error::msg("Campo cMun é obrigatório"))?
                    .to_string(),
                x_mun: dest
                    .x_mun
                    .ok_or_else(|| Error::msg("Campo xMun é obrigatório"))?,
                uf: dest
                    .uf
                    .ok_or_else(|| Error::msg("Campo UF é obrigatório"))?,
                cep: dest
                    .cep
                    .ok_or_else(|| Error::msg("Campo CEP é obrigatório"))?
                    .to_string(),
                c_pais: dest.c_pais.unwrap_or("1058".to_string()),
                x_pais: dest.x_pais.unwrap_or("BRASIL".to_string()),
                fone: dest.fone,
            },
            ind_ie_dest: dest.ind_ie_dest,
        };
        return Ok(DestProcess {
            cpf: Some(cpf_process),
            cnpj: None,
            id_estrangeiro: None,
        });
    }

    // Se for CNPJ
    if is_cnpj {
        let cnpj_process = CNPJProcess {
            cnpj: dest
                .cnpj
                .ok_or_else(|| Error::msg("Campo CNPJ é obrigatório"))?,
            x_nome: dest
                .x_nome
                .ok_or_else(|| Error::msg("Campo xNome é obrigatório"))?,
            ender_dest: EnderDestProcess {
                x_lgr: dest
                    .x_lgr
                    .ok_or_else(|| Error::msg("Campo xLgr é obrigatório"))?,
                nro: dest
                    .nro
                    .ok_or_else(|| Error::msg("Campo nro é obrigatório"))?,
                x_bairro: dest
                    .x_bairro
                    .ok_or_else(|| Error::msg("Campo xBairro é obrigatório"))?,
                c_mun: dest
                    .c_mun
                    .ok_or_else(|| Error::msg("Campo cMun é obrigatório"))?
                    .to_string(),
                x_mun: dest
                    .x_mun
                    .ok_or_else(|| Error::msg("Campo xMun é obrigatório"))?,
                uf: dest
                    .uf
                    .ok_or_else(|| Error::msg("Campo UF é obrigatório"))?,
                cep: dest
                    .cep
                    .ok_or_else(|| Error::msg("Campo CEP é obrigatório"))?
                    .to_string(),
                c_pais: dest.c_pais.unwrap_or("".to_string()),
                x_pais: dest.x_pais.unwrap_or("".to_string()),
                fone: dest.fone,
            },
            ind_ie_dest: dest.ind_ie_dest,
            ie: dest.ie,
            isuf: dest.isuf,
            im: dest.im,
            email: dest.email,
        };
        return Ok(DestProcess {
            cpf: None,
            cnpj: Some(cnpj_process),
            id_estrangeiro: None,
        });
    }

    // Se for ID Estrangeiro
    if is_id_estrangeiro {
        let id_estrangeiro_process = IdEstrangeiroProcess {
            id_estrangeiro: dest
                .id_estrangeiro
                .ok_or_else(|| Error::msg("Campo idEstrangeiro é obrigatório"))?,
            x_nome: dest
                .x_nome
                .ok_or_else(|| Error::msg("Campo xNome é obrigatório"))?,
            ender_dest: EnderDestProcess {
                x_lgr: dest
                    .x_lgr
                    .ok_or_else(|| Error::msg("Campo xLgr é obrigatório"))?,
                nro: dest
                    .nro
                    .ok_or_else(|| Error::msg("Campo nro é obrigatório"))?,
                x_bairro: dest
                    .x_bairro
                    .ok_or_else(|| Error::msg("Campo xBairro é obrigatório"))?,
                c_mun: dest
                    .c_mun
                    .ok_or_else(|| Error::msg("Campo cMun é obrigatório"))?
                    .to_string(),
                x_mun: dest
                    .x_mun
                    .ok_or_else(|| Error::msg("Campo xMun é obrigatório"))?,
                uf: dest
                    .uf
                    .ok_or_else(|| Error::msg("Campo UF é obrigatório"))?,
                cep: dest
                    .cep
                    .ok_or_else(|| Error::msg("Campo CEP é obrigatório"))?
                    .to_string(),
                c_pais: dest.c_pais.unwrap_or("".to_string()),
                x_pais: dest.x_pais.unwrap_or("".to_string()),
                fone: dest.fone,
            },
        };
        return Ok(DestProcess {
            cpf: None,
            cnpj: None,
            id_estrangeiro: Some(id_estrangeiro_process),
        });
    }

    // Se nenhuma das alternativas for verdadeira, é uma NFC-e
    Ok(DestProcess {
        cpf: None,
        cnpj: None,
        id_estrangeiro: None,
    })
}
