//! Preenchimento da NF-e/NFC-e conforme a NT 2026.005, seção 2.3.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::det_process::entity::DetProcess;
use super::rtc::{arred_rtc, fmt_rtc};
use crate::error::{DfeError, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimplesNacional {
    pub tp_regime_ibs_cbs: Option<u8>,
    /// Um elemento por item da nota, na ordem enviada ao builder.
    pub itens: Vec<ItemSimplesNacional>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemSimplesNacional {
    pub c_ativ_sn: String,
    /// Tipo de receita vigente do cClassTrib, obtido da tabela fiscal do emissor.
    pub tipo_receita_bruta: Option<u8>,
    /// Insumos de gTribSN, quando a matriz de regimes prevê o grupo.
    pub tributacao: Option<TributacaoSimplesNacional>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TributacaoSimplesNacional {
    /// Alíquotas efetivas aplicáveis ao item, em pontos percentuais, obtidas do PGDAS-D.
    pub p_ibs_sn: Option<Decimal>,
    pub p_cbs_sn: Option<Decimal>,
    /// Receita bruta resolvida pelo emissor. Ausente = produto + frete + acréscimo − desconto.
    pub receita_bruta: Option<Decimal>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegimeSimplesNacional {
    DentroDoDas,
    ExcessoSublimite,
    OpcaoRegimeRegular,
    OpcaoPendente,
    OpcaoPendenteExcessoSublimite,
    OpcaoPendenteRegimeRegular,
}

impl RegimeSimplesNacional {
    pub fn usa_grupo_regular(self) -> bool {
        self != Self::DentroDoDas
    }

    pub fn zera_cbs_regular(self) -> bool {
        self == Self::ExcessoSublimite
    }

    fn tributos_simples(self) -> Option<bool> {
        match self {
            Self::DentroDoDas | Self::OpcaoPendente => Some(true),
            Self::ExcessoSublimite | Self::OpcaoPendenteExcessoSublimite => Some(false),
            Self::OpcaoRegimeRegular | Self::OpcaoPendenteRegimeRegular => None,
        }
    }

    fn opcao_pendente(self) -> bool {
        matches!(
            self,
            Self::OpcaoPendente | Self::OpcaoPendenteExcessoSublimite
        )
    }
}

impl SimplesNacional {
    pub fn regime(&self, crt: u8) -> Result<RegimeSimplesNacional> {
        use RegimeSimplesNacional::*;
        match (crt, self.tp_regime_ibs_cbs) {
            (1, None) => Ok(DentroDoDas),
            (2, None) => Ok(ExcessoSublimite),
            (1 | 2, Some(1)) => Ok(OpcaoRegimeRegular),
            (3, Some(2)) => Ok(OpcaoPendente),
            (3, Some(3)) => Ok(OpcaoPendenteExcessoSublimite),
            (3, Some(4)) => Ok(OpcaoPendenteRegimeRegular),
            _ => Err(DfeError::Validacao(format!(
                "Simples Nacional: CRT {crt} incompatível com tpRegimeIBSCBS {:?}.",
                self.tp_regime_ibs_cbs
            ))),
        }
    }

    pub(super) fn aplicar(&self, crt: u8, fin_nfe: u8, dets: &mut [DetProcess]) -> Result<()> {
        let regime = self.regime(crt)?;
        if !matches!(fin_nfe, 1 | 2) {
            return Err(DfeError::Validacao(
                "Simples Nacional: esta finalidade requer tratamento próprio de IBS/CBS. Na devolução ou no retorno por recusa, espelhe o grupo da nota referenciada.".into(),
            ));
        }
        if self.itens.len() != dets.len() {
            return Err(DfeError::Validacao(
                "Simples Nacional: informe os dados de todos os itens da nota, na mesma ordem."
                    .into(),
            ));
        }
        for (indice, (entrada, det)) in self.itens.iter().zip(dets).enumerate() {
            let erro = |msg: &str| {
                DfeError::Validacao(format!("Simples Nacional, item {}: {msg}", indice + 1))
            };
            if !matches!(entrada.c_ativ_sn.as_str(), "01" | "02" | "03" | "04" | "90") {
                return Err(erro("cAtivSN inválido para NF-e/NFC-e."));
            }
            if entrada.c_ativ_sn == "90" && matches!(entrada.tipo_receita_bruta, Some(1..=5)) {
                return Err(erro(
                    "cAtivSN 90 é incompatível com o tipo de receita do cClassTrib.",
                ));
            }
            let ibs = det
                .imposto
                .ibs_cbs
                .as_mut()
                .ok_or_else(|| erro("informe CST e cClassTrib de IBS/CBS."))?;
            if ibs.cst == "620" {
                return Err(erro("a monofasia exige gIBSCBSMono."));
            }
            det.prod.c_ativ_sn = Some(entrada.c_ativ_sn.clone());
            if !regime.usa_grupo_regular()
                || super::CSTS_IBSCBS_SEM_GRUPO_REGULAR.contains(&ibs.cst.as_str())
            {
                ibs.g_ibscbs = None;
            } else if regime.zera_cbs_regular() {
                if let Some(grupo) = ibs.g_ibscbs.as_mut() {
                    grupo.g_cbs.p_cbs = "0.0000".into();
                    grupo.g_cbs.v_cbs = "0.00".into();
                    if let Some(reducao) = grupo.g_cbs.g_red.as_mut() {
                        reducao.p_aliq_efet = "0.0000".into();
                    }
                }
            }

            match (regime.tributos_simples(), entrada.tributacao.as_ref()) {
                (Some(com_ibs), Some(tributacao)) => {
                    ibs.g_trib_sn = Some(
                        tributacao
                            .calcular(&det.prod, entrada.tipo_receita_bruta, regime, com_ibs)
                            .map_err(|msg| erro(&msg))?,
                    );
                }
                (Some(_), None) => return Err(erro("informe os insumos de gTribSN.")),
                (None, Some(_)) => {
                    return Err(erro(
                        "gTribSN é incompatível com a opção pelo regime regular.",
                    ))
                }
                (None, None) => {}
            }
        }
        Ok(())
    }
}

impl TributacaoSimplesNacional {
    fn calcular(
        &self,
        prod: &super::det_process::entity::ProdProcess,
        tipo_receita_bruta: Option<u8>,
        regime: RegimeSimplesNacional,
        com_ibs: bool,
    ) -> std::result::Result<TribSimplesXml, String> {
        let fator_receita = match tipo_receita_bruta {
            Some(1 | 4) => Decimal::ONE,
            Some(5) => Decimal::new(5, 1),
            Some(0 | 2 | 3 | 9) => Decimal::ZERO,
            _ => return Err("informe o tipo de receita bruta vigente do cClassTrib.".into()),
        };
        let receita = match self.receita_bruta {
            Some(receita) => receita,
            None => {
                Decimal::from_str_exact(&prod.v_prod).map_err(|e| e.to_string())?
                    + prod.v_frete.unwrap_or_default()
                    + prod.v_outro.unwrap_or_default()
                    - prod.v_desc.unwrap_or_default()
            }
        };
        if receita.is_sign_negative() {
            return Err("receita bruta negativa.".into());
        }
        let suspenso = tipo_receita_bruta == Some(4) || regime.opcao_pendente();
        let base_devida = if suspenso {
            Decimal::ZERO
        } else {
            arred_rtc(receita, 2) * fator_receita
        };
        let calcular = |aliquota: Option<Decimal>, tributo| {
            let aliquota = match aliquota {
                Some(aliquota) => aliquota,
                None if fator_receita.is_zero() => Decimal::ZERO,
                None => {
                    return Err(format!(
                        "informe a alíquota efetiva de {tributo} do Simples Nacional."
                    ))
                }
            };
            if !(Decimal::ZERO..=Decimal::ONE_HUNDRED).contains(&aliquota) {
                return Err(
                    "alíquota efetiva do Simples Nacional fora do intervalo de 0 a 100."
                        .to_string(),
                );
            }
            base_devida
                .checked_mul(aliquota / Decimal::ONE_HUNDRED)
                .map(|valor| (fmt_rtc(aliquota, 4), fmt_rtc(valor, 2)))
                .ok_or_else(|| {
                    "valor do tributo do Simples Nacional excede o limite decimal.".into()
                })
        };
        let g_ibs = if com_ibs {
            let (p_ibs, v_ibs) = calcular(self.p_ibs_sn, "IBS")?;
            Some(IbsSimplesXml { p_ibs, v_ibs })
        } else {
            if self.p_ibs_sn.is_some() {
                return Err("o IBS do excesso de sublimite pertence ao regime regular.".into());
            }
            None
        };
        let (p_cbs, v_cbs) = calcular(self.p_cbs_sn, "CBS")?;
        Ok(TribSimplesXml {
            g_ibs,
            g_cbs: CbsSimplesXml { p_cbs, v_cbs },
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TribSimplesXml {
    #[serde(rename = "gIBSSN", skip_serializing_if = "Option::is_none")]
    pub g_ibs: Option<IbsSimplesXml>,
    #[serde(rename = "gCBSSN")]
    pub g_cbs: CbsSimplesXml,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct IbsSimplesXml {
    #[serde(rename = "pIBSSN")]
    p_ibs: String,
    #[serde(rename = "vIBSSN")]
    pub v_ibs: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct CbsSimplesXml {
    #[serde(rename = "pCBSSN")]
    p_cbs: String,
    #[serde(rename = "vCBSSN")]
    pub v_cbs: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tipos::{Dest, Det, Emit, IbsCbs, Ide, Pag, Total, Transp};
    use crate::NFeBuilder;

    fn dados(tp_regime: Option<u8>, aliquota_ibs: Option<Decimal>) -> SimplesNacional {
        let item = ItemSimplesNacional {
            c_ativ_sn: "01".into(),
            tipo_receita_bruta: Some(1),
            tributacao: (tp_regime != Some(1) && tp_regime != Some(4)).then_some(
                TributacaoSimplesNacional {
                    p_ibs_sn: aliquota_ibs,
                    p_cbs_sn: Some(Decimal::new(5, 2)),
                    receita_bruta: None,
                },
            ),
        };
        SimplesNacional {
            tp_regime_ibs_cbs: tp_regime,
            itens: vec![item.clone(), item],
        }
    }

    fn builder(crt: u8, modelo: u32, simples: SimplesNacional) -> NFeBuilder {
        let emit = Emit {
            crt,
            cnpj: Some("12345678000195".into()),
            ie: Some("110042490114".into()),
            x_lgr: "Rua de Teste".into(),
            nro: "1".into(),
            x_bairro: "Centro".into(),
            c_mun: "3550308".into(),
            x_mun: "São Paulo".into(),
            uf: "SP".into(),
            cep: "01001000".into(),
            ..Default::default()
        };
        let item = Det {
            c_prod: "1".into(),
            c_ean: "SEM GTIN".into(),
            x_prod: "Item teste".into(),
            ncm: "10063021".into(),
            cfop: 5102,
            u_com: "UN".into(),
            u_trib: "UN".into(),
            q_com: 1.0,
            v_un_com: 14.0,
            v_prod: 14.0,
            c_ean_trib: "SEM GTIN".into(),
            q_trib: 1.0,
            v_un_trib: 14.0,
            ind_tot: 1,
            ibs_cbs: Some(IbsCbs {
                cst: "000".into(),
                class_trib: "000001".into(),
                calcular: true,
                v_bc: Decimal::ZERO,
                p_ibs_uf: Decimal::new(5, 2),
                p_ibs_mun: Decimal::new(5, 2),
                p_cbs: Decimal::new(9, 1),
                v_ibs_uf: Decimal::ZERO,
                v_ibs_mun: Decimal::ZERO,
                v_cbs: Decimal::ZERO,
                p_red_ibs: None,
                p_red_cbs: None,
            }),
            ..Default::default()
        };
        let mut builder = NFeBuilder::new()
            .cert("tests/fixtures/cert/sintetico_aes.pfx", "senha-teste-dfe")
            .ide(Ide {
                mod_: modelo,
                tp_imp: if modelo == 65 { 4 } else { 1 },
                ..Default::default()
            })
            .emitente(emit)
            .itens(vec![item.clone(), item])
            .pagamento(Pag {
                t_pag: "01".into(),
                v_pag: 28.0,
                ..Default::default()
            })
            .total(Total::default())
            .transporte(Transp::default())
            .simples_nacional(simples);
        if modelo == 55 {
            builder = builder.destinatario(Dest {
                cpf: Some("12345678909".into()),
                x_nome: Some("Consumidor".into()),
                x_lgr: Some("Rua Teste".into()),
                nro: Some("1".into()),
                x_bairro: Some("Centro".into()),
                c_mun: Some("3550308".into()),
                x_mun: Some("São Paulo".into()),
                uf: Some("SP".into()),
                ..Default::default()
            });
        }
        builder
    }

    #[tokio::test]
    async fn crt1_no_das_ou_com_opcao_regular_gera_xml_valido() {
        let xml = builder(1, 55, dados(None, Some(Decimal::new(5, 2))))
            .gerar_xml()
            .await
            .unwrap();
        assert_eq!(xml.matches("<cAtivSN>01</cAtivSN>").count(), 2);
        assert_eq!(xml.matches("<vIBSSN>0.01</vIBSSN>").count(), 2);
        assert!(xml.contains("<IBSCBSTot><vBCIBSCBS>0.00</vBCIBSCBS><gTribSN><vIBSSN>0.02</vIBSSN><vCBSSN>0.02</vCBSSN></gTribSN></IBSCBSTot>"));
        assert!(!xml.contains("<gIBSCBS>"));
        let xml = builder(1, 55, dados(Some(1), None))
            .gerar_xml()
            .await
            .unwrap();
        assert!(xml.contains("<CRT>1</CRT><tpRegimeIBSCBS>1</tpRegimeIBSCBS>"));
        assert!(xml.contains("<gIBSCBS>"));
        assert!(!xml.contains("<gTribSN>"));
    }

    #[tokio::test]
    async fn excesso_sublimite_distingue_ibs_regular_e_cbs_simples_na_nfce() {
        let xml = builder(2, 65, dados(None, None))
            .csc("csc-teste")
            .id_csc("1")
            .gerar_xml()
            .await
            .unwrap();
        assert_eq!(
            xml.matches("<gCBS><pCBS>0.0000</pCBS><vCBS>0.00</vCBS></gCBS>")
                .count(),
            2
        );
        assert_eq!(
            xml.matches("<gIBSUF><pIBSUF>0.0500</pIBSUF><vIBSUF>0.01</vIBSUF></gIBSUF>")
                .count(),
            2
        );
        assert!(xml.contains("<gTribSN><vIBSSN>0.00</vIBSSN><vCBSSN>0.02</vCBSSN></gTribSN>"));
        assert!(!xml.contains("<gIBSSN>"));
    }

    #[tokio::test]
    async fn crt_incompativel_e_atividade_invalida_sao_recusados() {
        let erro = builder(4, 55, dados(Some(1), None))
            .gerar_xml()
            .await
            .unwrap_err();
        assert!(erro.to_string().contains("CRT 4"));
        let mut sn = dados(None, None);
        sn.itens[0].c_ativ_sn = "05".into();
        let erro = builder(2, 55, sn).gerar_xml().await.unwrap_err();
        assert!(erro.to_string().contains("cAtivSN"));
    }

    #[tokio::test]
    async fn opcao_pendente_mantem_regular_e_nao_destaca_valor_devido_no_simples() {
        let xml = builder(3, 55, dados(Some(2), Some(Decimal::new(125, 2))))
            .gerar_xml()
            .await
            .unwrap();
        assert!(xml.contains("<CRT>3</CRT><tpRegimeIBSCBS>2</tpRegimeIBSCBS>"));
        assert_eq!(
            xml.matches("<gCBS><pCBS>0.9000</pCBS><vCBS>0.13</vCBS></gCBS>")
                .count(),
            2
        );
        assert!(xml.contains("<gTribSN><vIBSSN>0.00</vIBSSN><vCBSSN>0.00</vCBSSN></gTribSN>"));
    }

    #[tokio::test]
    async fn atividade_nao_tributavel_exige_tipo_compativel_e_dispensa_aliquota() {
        let mut sn = dados(None, None);
        for item in &mut sn.itens {
            item.c_ativ_sn = "90".into();
            item.tributacao.as_mut().unwrap().p_cbs_sn = None;
        }
        let erro = builder(1, 55, sn.clone()).gerar_xml().await.unwrap_err();
        assert!(erro.to_string().contains("cAtivSN 90"));
        for item in &mut sn.itens {
            item.tipo_receita_bruta = Some(0);
        }
        let xml = builder(1, 55, sn).gerar_xml().await.unwrap();
        assert!(xml.contains("<gTribSN><vIBSSN>0.00</vIBSSN><vCBSSN>0.00</vCBSSN></gTribSN>"));
    }
}
