use super::Det;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "prod")]
pub struct ProdProcess {
    #[serde(rename = "cProd")]
    pub c_prod: String,
    #[serde(rename = "cEAN")]
    pub c_ean: String,
    #[serde(rename = "xProd")]
    pub x_prod: String,
    #[serde(rename = "NCM")]
    pub ncm: String,
    #[serde(rename = "CEST", skip_serializing_if = "Option::is_none")]
    pub cest: Option<String>,
    #[serde(rename = "CFOP")]
    pub cfop: String,
    #[serde(rename = "uCom")]
    pub u_com: String,
    #[serde(rename = "qCom")]
    pub q_com: String,
    #[serde(rename = "vUnCom")]
    pub v_un_com: String,
    #[serde(rename = "vProd")]
    pub v_prod: String,
    #[serde(rename = "cEANTrib")]
    pub c_ean_trib: String,
    #[serde(rename = "uTrib")]
    pub u_trib: String,
    #[serde(rename = "qTrib")]
    pub q_trib: String,
    #[serde(rename = "vUnTrib")]
    pub v_un_trib: String,
    #[serde(rename = "indTot")]
    pub ind_tot: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "imposto")]
pub struct ImpostoProcess {
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: String,
    #[serde(rename = "ICMS")]
    pub icms: ICMSProcess,
    #[serde(rename = "PIS")]
    pub pis: PISProcess,
    #[serde(rename = "COFINS")]
    pub cofins: COFINSProcess,
}

/// 164 N01 ICMS Informações do ICMS da Operação própria e ST CG M01  1-1  Informar apenas um dos grupos de tributação do ICMS (ICMS00, ICMS10, ...) (v2.0)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ICMSProcess {
    ICMS00(ICMS00),
    ICMS10(ICMS10),
    ICMS20(ICMS20),
    ICMS30(ICMS30),
    ICMS40(ICMS40),
    ICMS51(ICMS51),
    ICMS60(ICMS60),
    ICMS70(ICMS70),
    ICMS90(ICMS90),
    ICMSPart(ICMSPart),
    ICMSSN101(ICMSSN101),
    ICMSSN102(ICMSSN102),
    ICMSSN201(ICMSSN201),
    ICMSSN202(ICMSSN202),
    ICMSSN500(ICMSSN500),
    ICMSSN900(ICMSSN900),
}

// Defina os structs para cada tipo de ICMS aqui
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS00 {
    // Campos específicos para ICMS00
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "modBC")]
    pub mod_bc: u8,
    #[serde(rename = "vBC")]
    pub v_bc: f32,
    #[serde(rename = "pICMS")]
    pub p_icms: f32,
    #[serde(rename = "vICMS")]
    pub v_icms: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS10 {
    // Campos específicos para ICMS10
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS20 {
    // Campos específicos para ICMS20
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS30 {
    // Campos específicos para ICMS30
}

/// Campos específicos para ICMS40
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "ICMS40")]
pub struct ICMS40 {
    /// Origem da mercadoria Ex: 0 para Nacional
    pub orig: u8,
    /// Código de Situação Tributária
    /// Ex: 40 = Isenta
    /// Ex: 41 = Não tributada
    /// Ex: 50 = Suspensão
    #[serde(rename = "CST")]
    pub cst: u16,
    /// 204.00 N27.1 -x- Sequência XML G N06 0-1 Grupo opcional.
    ///
    /// Informar apenas nas operações:
    /// a) com produtos beneficiados com a desoneração condicional do ICMS.
    /// b) destinadas à SUFRAMA, informando-se o valor que seria devido se não houvesse isenção.
    /// c) de venda a órgão da administração pública direta e suas Nota Fiscal eletrônica fundações e autarquias com isenção do ICMS. (NT 2011/004)
    #[serde(rename = "vICMSDeson", skip_serializing_if = "Option::is_none")]
    pub vicmsdeson: Option<f32>,
    /// Campo será preenchido quando o campo anterior estiver preenchido.
    /// Informar o motivo da desoneração:
    /// 1 = Táxi;
    /// 3 = Produtor Agropecuário;
    /// 4 = Frotista/Locadora;
    /// 5 = Diplomático/Consular;
    /// 6 = Utilitários e Motocicletas da Amazônia Ocidental e Áreas de
    /// Livre Comércio (Resolução 714/88 e 790/94 – CONTRAN e suas alterações);
    /// 7 = SUFRAMA;
    /// 8 = Venda a Órgão Público;
    /// 9 = Outros. (NT 2011/004);
    /// 10 = Deficiente Condutor (Convênio ICMS 38/12);
    /// 11 = Deficiente Não Condutor (Convênio ICMS 38/12).
    /// Revogada a partir da versão 3.
    #[serde(rename = "motDesICMS", skip_serializing_if = "Option::is_none")]
    pub mot_des_icms: Option<u16>,
}

impl Default for ICMS40 {
    fn default() -> Self {
        ICMS40 {
            orig: 0,
            cst: 40,
            vicmsdeson: None,
            mot_des_icms: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS51 {
    // Campos específicos para ICMS51
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS60 {
    // Campos específicos para ICMS60
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS70 {
    // Campos específicos para ICMS70
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS90 {
    // Campos específicos para ICMS90
    pub orig: u8,
    #[serde(rename = "CST")]
    pub cst: String,
}

impl Default for ICMS90 {
    fn default() -> Self {
        ICMS90 {
            orig: 0,
            cst: "90".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSPart {
    // Campos específicos para ICMSPart
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN101 {
    // Campos específicos para ICMSSN101
    pub orig: u8,
    #[serde(rename = "CSOSN")]
    pub csosn: u16,
}

/// 245.46 N10f ICMSSN102 Grupo CRT=1 – Simples Nacional e CSOSN = 102 CG N01  1-1  Tributação ICMS pelo Simples Nacional, CSOSN=102 (v2.0)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN102 {
    /// Origem da mercadoria Ex: 0 para Nacional
    pub orig: u8,
    /// Código de Situação da Operação - Simples Nacional
    /// Ex: 102 = Tributada pelo Simples Nacional sem permissão de crédito
    /// Ex: 103 = Isenção do ICMS no Simples Nacional para faixa de receita bruta
    /// Ex: 300 = Imune
    /// Ex: 400 = Não tributada pelo Simples Nacional
    #[serde(rename = "CSOSN")]
    pub csosn: u16,
    #[serde(rename = "pCredSN")]
    pub p_cred_sn: f32,
    #[serde(rename = "vCredICMSSN")]
    pub v_cred_icmssn: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN201 {
    // Campos específicos para ICMSSN201
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN202 {
    // Campos específicos para ICMSSN202
}

/// 245.47 N10g ICMSSN500 Grupo CRT=1 – Simples Nacional e CSOSN = 500 CG N01  1-1  Tributação ICMS pelo Simples Nacional, CSOSN=500 (v2.0)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN500 {
    #[serde(rename = "orig")]
    pub orig: u8, // Origem da mercadoria
    #[serde(rename = "CSOSN")]
    pub csosn: u16, // Código de Situação da Operação - Simples Nacional
    #[serde(rename = "vBCSTRet")]
    pub vbcst_ret: Option<String>, // Valor da BC do ICMS ST retido
    #[serde(rename = "vICMSSTRet")]
    pub vicmsst_ret: Option<String>, // Valor do ICMS ST retido
}

impl Default for ICMSSN500 {
    fn default() -> Self {
        ICMSSN500 {
            orig: 0,
            csosn: 500,
            vbcst_ret: None,
            vicmsst_ret: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSSN900 {
    // Campos específicos para ICMSSN900
    #[serde(rename = "orig")]
    pub orig: String, // Origem da mercadoria
    #[serde(rename = "CSOSN")]
    // 245.55 N12.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    pub csosn: String, // Código de Situação da Operação - Simples Nacional
    #[serde(rename = "modBC")]
    pub modbc: Option<String>, // Modalidade de determinação da BC do ICMS
    #[serde(rename = "vBC")]
    pub vbc: Option<String>, // Valor da BC do ICMS
    #[serde(rename = "pRedBC")]
    pub pred_bc: Option<String>, // Percentual de redução da BC
    #[serde(rename = "pICMS")]
    pub picms: Option<String>, // Alíquota do ICMS
    #[serde(rename = "vICMS")]
    pub vicms: Option<String>, // Valor do ICMS
    // 245.60 N17.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    #[serde(rename = "modBCST")]
    pub modbcst: Option<String>, // Modalidade de determinação da BC do ICMS ST
    #[serde(rename = "pMVAST")]
    pub pmvast: Option<String>, // Percentual da margem de valor Adicionado do ICMS ST
    #[serde(rename = "pRedBCST")]
    pub pred_bcst: Option<String>, // Percentual de redução da BC do ICMS ST
    #[serde(rename = "vBCST")]
    pub vbcst: Option<String>, // Valor da BC do ICMS ST
    #[serde(rename = "pICMSST")]
    pub picmsst: Option<String>, // Alíquota do ICMS ST
    #[serde(rename = "vICMSST")]
    pub vicmsst: Option<String>, // Valor do ICMS ST
    // 245.52 N27.1 -x- Sequência XML G N10h  0-1  Grupo opcional.
    #[serde(rename = "pCredSN")]
    pub pcred_sn: Option<String>, // Alíquota aplicável de cálculo do crédito (Simples Nacional)
    #[serde(rename = "vCredICMSSN")]
    pub vcred_icmssn: Option<String>, // Valor crédito do ICMS que pode ser aproveitado nos termos do art. 23 da LC 123 (Simples Nacional)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISProcess {
    #[serde(rename = "PISOutr")]
    pub pis_outr: PISOutr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISOutr {
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "qBCProd", skip_serializing_if = "Option::is_none")]
    pub qbc_prod: Option<String>,
    #[serde(rename = "vAliqProd", skip_serializing_if = "Option::is_none")]
    pub valiq_prod: Option<String>,
    #[serde(rename = "vPIS", skip_serializing_if = "Option::is_none")]
    pub vpis: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSProcess {
    #[serde(rename = "COFINSOutr")]
    pub cofins_outr: COFINSOutr,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSOutr {
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "qBCProd")]
    pub qbc_prod: String,
    #[serde(rename = "vAliqProd")]
    pub valiq_prod: String,
    #[serde(rename = "vCOFINS")]
    pub vcofins: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "det")]
pub struct DetProcess {
    #[serde(rename = "prod")]
    pub prod: ProdProcess,
    #[serde(rename = "imposto")]
    pub imposto: ImpostoProcess,
}
pub fn det_process(prod: Vec<Det>) -> Result<Vec<DetProcess>, Error> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    for d in &prod {
        let icms = match d.icms.as_str() {
            "ICMSSN101" => {
                // Tributada pelo Simples Nacional com permissão de crédito. (v2.0)
                let orig = match d.orig {
                    Some(orig) => orig,
                    None => {
                        return Err(Error::msg("Origem (var orig) da mercadoria não informada"))
                    }
                };

                let csosn = match d.csosn {
                    Some(csosn) => csosn,
                    None => {
                        return Err(Error::msg("CSOSN (var csosn) não informado para ICMSSN101"))
                    }
                };
                ICMSProcess::ICMSSN101(ICMSSN101 { orig, csosn })
            }
            "ICMSSN102" => {
                // Tributada pelo Simples Nacional sem permissão de crédito e com cobrança do ICMS por substituição tributária. (v2.0)
                let orig = match d.orig {
                    Some(orig) => orig,
                    None => {
                        return Err(Error::msg("Origem (var orig) da mercadoria não informada"))
                    }
                };
                let csosn = match d.csosn {
                    Some(csosn) => csosn,
                    None => {
                        return Err(Error::msg("CSOSN (var csosn) não informado para ICMSSN102"))
                    }
                };
                let p_cred_sn = match d.p_cred_sn {
                    Some(p_cred_sn) => p_cred_sn,
                    None => {
                        return Err(Error::msg(
                            "Percentual de crédito (var p_cred_sn) não informado",
                        ))
                    }
                };
                let v_cred_icmssn = match d.v_cred_icmssn {
                    Some(v_cred_icmssn) => v_cred_icmssn,
                    None => {
                        return Err(Error::msg(
                            "Valor do crédito do ICMS (var v_cred_icmssn) não informado",
                        ))
                    }
                };
                ICMSProcess::ICMSSN102(ICMSSN102 {
                    orig,
                    csosn,
                    p_cred_sn,
                    v_cred_icmssn,
                })
            }
            "ICMS00" => {
                let orig = match d.orig {
                    Some(orig) => orig,
                    None => return Err(Error::msg("Origem da mercadoria não informada")),
                };
                let cst = match d.cst.clone() {
                    Some(cst) => cst,
                    None => return Err(Error::msg("CST não informado")),
                };
                let mod_bc = match d.mod_bc {
                    Some(mod_bc) => mod_bc,
                    None => {
                        return Err(Error::msg(
                            "Modalidade de determinação da BC do ICMS não informada",
                        ))
                    }
                };
                let v_bc = match d.v_bc {
                    Some(v_bc) => v_bc,
                    None => return Err(Error::msg("Valor da BC do ICMS não informado")),
                };
                let p_icms = match d.p_icms {
                    Some(p_icms) => p_icms,
                    None => return Err(Error::msg("Alíquota do ICMS não informada")),
                };
                let v_icms = match d.v_icms {
                    Some(v_icms) => v_icms,
                    None => return Err(Error::msg("Valor do ICMS não informado")),
                };
                ICMSProcess::ICMS00(ICMS00 {
                    orig,
                    cst,
                    mod_bc,
                    v_bc,
                    p_icms,
                    v_icms,
                })
            }
            "ICMS40" => ICMSProcess::ICMS40(ICMS40 {
                orig: 0,
                cst: 41,
                ..Default::default()
            }),
            "ICMS90" => {
                let orig = match d.orig {
                    Some(orig) => orig,
                    None => return Err(Error::msg("Origem da mercadoria não informada")),
                };
                let cst = match d.cst.clone() {
                    Some(cst) => cst,
                    None => return Err(Error::msg("CST não informado")),
                };
                ICMSProcess::ICMS90(ICMS90 { orig, cst })
            }
            _ => return Err(Error::msg("Unsupported ICMS type")),
        };

        det_process_values.push(DetProcess {
            prod: ProdProcess {
                c_prod: d.c_prod.to_string(),
                c_ean: d.c_ean.to_string(),
                x_prod: d.x_prod.to_string(),
                ncm: d.ncm.to_string(),
                cfop: d.cfop.to_string(),
                cest: d.cest.clone(),
                u_com: d.u_com.to_string(),
                q_com: format!("{:.2}", d.q_com),
                v_un_com: format!("{:.2}", d.v_un_com),
                v_prod: format!("{:.2}", d.v_prod),
                c_ean_trib: d.c_ean_trib.to_string(),
                u_trib: d.u_trib.to_string(),
                q_trib: format!("{:.2}", d.q_trib),
                v_un_trib: format!("{:.2}", d.v_un_trib),
                ind_tot: d.ind_tot.to_string(),
            },
            imposto: ImpostoProcess {
                v_tot_trib: format!("{:.2}", d.v_tot_trib),
                icms: icms.clone(),
                pis: PISProcess {
                    pis_outr: PISOutr {
                        cst: 99.to_string(),
                        qbc_prod: Some("0.00".to_string()),
                        valiq_prod: Some("0.00".to_string()),
                        vpis: Some("0.00".to_string()),
                    },
                },
                cofins: COFINSProcess {
                    cofins_outr: COFINSOutr {
                        cst: 99.to_string(),
                        qbc_prod: "0.00".to_string(),
                        valiq_prod: "0.00".to_string(),
                        vcofins: "0.00".to_string(),
                    },
                },
            },
        });
    }
    Ok(det_process_values)
}
/*

ERro: duplicando vetor por extend
pub fn det_process(prod: Vec<Det>) -> Result<Vec<DetProcess>, Error> {
    let mut det_process_values: Vec<DetProcess> = Vec::new();
    for d in &prod {
        let icms = match d.icms.as_str() {
            "ICMSSN102" => ICMSProcess::ICMSSN102(ICMSSN102 {
                orig: 0,
                csosn: 102,
            }),
            "ICMS40" => ICMSProcess::ICMS40(ICMS40 {
                orig: 0,
                cst: 41,
                ..Default::default()
            }),
            _ => return Err(Error::msg("Unsupported ICMS type")),
        };

        det_process_values.extend(
            prod.iter()
                .map(|d| DetProcess {
                    prod: ProdProcess {
                        c_prod: d.c_prod.to_string(),
                        c_ean: d.c_ean.to_string(),
                        x_prod: d.x_prod.to_string(),
                        ncm: d.ncm.to_string(),
                        cfop: d.cfop.to_string(),
                        cest: d.cest.clone(),
                        u_com: d.u_com.to_string(),
                        q_com: format!("{:.2}", d.q_com),
                        v_un_com: format!("{:.2}", d.v_un_com),
                        v_prod: format!("{:.2}", d.v_prod),
                        c_ean_trib: d.c_ean_trib.to_string(),
                        u_trib: d.u_trib.to_string(),
                        q_trib: format!("{:.2}", d.q_trib),
                        v_un_trib: format!("{:.2}", d.v_un_trib),
                        ind_tot: d.ind_tot.to_string(),
                    },
                    imposto: ImpostoProcess {
                        v_tot_trib: format!("{:.2}", d.v_tot_trib),
                        icms: icms.clone(),
                        pis: PISProcess {
                            pis_outr: PISOutr {
                                cst: 99.to_string(),
                                qbc_prod: Some("0.00".to_string()),
                                valiq_prod: Some("0.00".to_string()),
                                vpis: Some("0.00".to_string()),
                            },
                        },
                        cofins: COFINSProcess {
                            cofins_outr: COFINSOutr {
                                cst: 99.to_string(),
                                qbc_prod: "0.00".to_string(),
                                valiq_prod: "0.00".to_string(),
                                vcofins: "0.00".to_string(),
                            },
                        },
                    },
                })
                .collect::<Vec<_>>(),
        );
    }
    Ok(det_process_values)
} */
