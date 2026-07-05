//! Montagem genérica do `<infEvento>` de eventos da NF-e (cancelamento, manifestação, e
//! futuros: substituição, CC-e…).
//!
//! Unifica (fase A4) o `infEvento` antes duplicado em `cancelar` e `manifestacao`. A parte
//! **comum** (cOrgao, tpAmb, CNPJ, chNFe, dhEvento, tpEvento, nSeqEvento, verEvento e o
//! esqueleto do `detEvento`) mora aqui; a parte **variável** do `detEvento` (descEvento + campos
//! específicos como nProt/xJust) vem por parâmetro.
//!
//! Usa `quick_xml::Writer` (escapa o conteúdo de texto) e produz saída **byte-a-byte idêntica**
//! às implementações anteriores — travado por golden em `cancelar::tests` e `manifestacao::tests`.
//! O `infEvento` é o nó cujo digest é assinado, então qualquer mudança aqui afeta a assinatura.

use crate::error::{DfeError, Result};
use crate::interno::dates::get_current_date_time;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::io::Cursor;

/// Parâmetros do `<infEvento>`. O `dhEvento` é gerado internamente (data/hora atual).
pub struct InfEvento<'a> {
    /// Órgão: UF da chave (cancelamento) ou `"91"` = Ambiente Nacional (manifestação).
    pub c_orgao: &'a str,
    pub tp_amb: u8,
    /// Documento do autor do evento (CNPJ) — emitido no elemento `<CNPJ>`.
    pub cnpj: &'a str,
    pub chave: &'a str,
    pub tp_evento: &'a str,
    pub n_seq_evento: u32,
    pub ver_evento: &'a str,
    /// Descrição do evento (`<descEvento>`), ex.: "Cancelamento", "Confirmacao da Operacao".
    pub desc_evento: &'a str,
    /// Campos do `detEvento` **após** `descEvento`, em ordem (ex.: `[("nProt", …), ("xJust", …)]`).
    pub det_campos: &'a [(&'a str, &'a str)],
}

/// Monta o `<infEvento>` completo (com `Id` = `ID{tpEvento}{chave}{nSeqEvento:02}`).
pub fn inf_evento_xml(ev: &InfEvento) -> Result<String> {
    let inf_evento_id = format!("ID{}{}{:>02}", ev.tp_evento, ev.chave, ev.n_seq_evento);
    let tp_amb = ev.tp_amb.to_string();
    let dh_evento = get_current_date_time();
    let n_seq_evento = ev.n_seq_evento.to_string();

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .create_element("infEvento")
        .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
        .with_attribute(("Id", inf_evento_id.as_str()))
        .write_inner_content(|w| {
            w.create_element("cOrgao").write_text_content(BytesText::new(ev.c_orgao))?;
            w.create_element("tpAmb").write_text_content(BytesText::new(&tp_amb))?;
            w.create_element("CNPJ").write_text_content(BytesText::new(ev.cnpj))?;
            w.create_element("chNFe").write_text_content(BytesText::new(ev.chave))?;
            w.create_element("dhEvento").write_text_content(BytesText::new(&dh_evento))?;
            w.create_element("tpEvento").write_text_content(BytesText::new(ev.tp_evento))?;
            w.create_element("nSeqEvento").write_text_content(BytesText::new(&n_seq_evento))?;
            w.create_element("verEvento").write_text_content(BytesText::new(ev.ver_evento))?;
            w.create_element("detEvento")
                .with_attribute(("versao", ev.ver_evento))
                .write_inner_content(|w| {
                    w.create_element("descEvento")
                        .write_text_content(BytesText::new(ev.desc_evento))?;
                    for (tag, val) in ev.det_campos {
                        w.create_element(*tag).write_text_content(BytesText::new(val))?;
                    }
                    Ok(())
                })?;
            Ok(())
        })?;

    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

/// Monta o envelope SOAP 1.2 `envEvento` (nó `nfeDadosMsg` do `NFeRecepcaoEvento4`) que
/// transporta o `<infEvento>` assinado até a SEFAZ.
///
/// A4b/A5: unifica (fase de eventos) o envelope antes **duplicado byte-a-byte** em `cancelar` e
/// `manifestacao`. O `idLote` é gerado internamente (data/hora + dígito aleatório) — é o único
/// campo volátil e não faz parte do nó assinado. `inf_evento`/`signature` são inseridos como XML
/// já pronto (placeholders substituídos), pois são conteúdo canônico e não devem ser re-escapados.
pub fn env_evento_xml(inf_evento: &str, signature: &str) -> Result<String> {
    let lote_id = id_lote_generate();
    let lote_id = lote_id.as_str();
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer
        .create_element("soap12:Envelope")
        .with_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"))
        .with_attribute(("xmlns:xsd", "http://www.w3.org/2001/XMLSchema"))
        .with_attribute(("xmlns:soap12", "http://www.w3.org/2003/05/soap-envelope"))
        .write_inner_content(|writer| {
            writer
                .create_element("soap12:Body")
                .write_inner_content(|writer| {
                    writer
                        .create_element("nfeDadosMsg")
                        .with_attribute((
                            "xmlns",
                            "http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4",
                        ))
                        .write_inner_content(|writer| {
                            writer
                                .create_element("envEvento")
                                .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
                                .with_attribute(("versao", "1.00"))
                                .write_inner_content(|writer| {
                                    writer
                                        .create_element("idLote")
                                        .write_text_content(BytesText::new(lote_id))?;
                                    writer
                                        .create_element("evento")
                                        .with_attribute((
                                            "xmlns",
                                            "http://www.portalfiscal.inf.br/nfe",
                                        ))
                                        .with_attribute(("versao", "1.00"))
                                        .write_inner_content(|writer| {
                                            writer
                                                .create_element("REPLACER_INF_EVENTO")
                                                .write_empty()?;
                                            writer
                                                .create_element("REPLACER_SIGNATURE")
                                                .write_empty()?;
                                            Ok(())
                                        })?;
                                    Ok(())
                                })?;
                            Ok(())
                        })?;
                    Ok(())
                })?;
            Ok(())
        })?;

    let string = writer.into_inner().into_inner();
    let string = String::from_utf8(string)?;
    let string = string.replace("<REPLACER_INF_EVENTO/>", inf_evento);
    let string = string.replace("<REPLACER_SIGNATURE/>", signature);
    let string = "<?xml version=\"1.0\" encoding=\"utf-8\"?>".to_string() + &string;
    Ok(string)
}

fn id_lote_generate() -> String {
    let date = chrono::Local::now();
    let date = date.format("%Y%m%d%H%M%S").to_string();
    let random = rand::random::<u8>() % 10;
    format!("{}{}", date, random)
}

/// Wrapper tipado do `<retEnvEvento>` (resposta do `NFeRecepcaoEvento4`). Só os campos que
/// interessam são declarados; os demais (`idLote`, `verAplic`, …) são ignorados pelo serde.
#[derive(serde::Deserialize)]
struct RetEnvEvento<T> {
    #[serde(rename = "retEvento")]
    ret_evento: Option<RetEvento<T>>,
    #[serde(rename = "cStat", default)]
    c_stat: String,
    #[serde(rename = "xMotivo", default)]
    x_motivo: String,
}

/// `<retEvento>` — envelopa o `<infEvento>` de retorno (o `T` de cada família de evento).
#[derive(serde::Deserialize)]
struct RetEvento<T> {
    #[serde(rename = "infEvento")]
    inf_evento: T,
}

/// Extrai o subtree `<local_name …>…</local_name>` (nome local, ignora atributos na abertura).
///
/// Usado para isolar o `retEnvEvento` — **elemento de nome estável** — de dentro do envelope SOAP,
/// cujo **wrapper varia** entre implantações da SEFAZ (`nfeRecepcaoEventoResult`, `nfeResultMsg`, …),
/// tornando a desserialização do envelope inteiro frágil. Assume nome sem prefixo de namespace,
/// como a SEFAZ retorna (default namespace) — mesma premissa que o parse anterior já usava.
fn extract_subtree<'a>(xml: &'a str, local_name: &str) -> Option<&'a str> {
    let start = xml.find(&format!("<{local_name}"))?;
    let close = format!("</{local_name}>");
    let end = xml[start..].find(&close)? + start + close.len();
    Some(&xml[start..end])
}

/// Extrai e desserializa o `<retEvento>/<infEvento>` da resposta SOAP da SEFAZ para o tipo `T`.
///
/// Unifica (fase de eventos) o parse antes duplicado em `cancelar`/`manifestacao`. `T` é o
/// `InfEvento` de retorno de cada família de evento (os campos variam: cancelamento não traz
/// `CNPJDest`, manifestação sim).
///
/// **A5:** o recorte por regex do `<infEvento>` (frágil e dependente de ordem) foi trocado por
/// **desserialização tipada** do `retEnvEvento` via `quick_xml::de`. O único passo textual é
/// isolar o subtree `retEnvEvento` (nome estável) do envelope SOAP — cujo wrapper varia. Quando o
/// **lote** é rejeitado (sem `retEvento`), o erro carrega o `cStat`/`xMotivo` de lote.
pub fn parse_ret_evento<T>(response: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let subtree = extract_subtree(response, "retEnvEvento").ok_or_else(|| {
        DfeError::Xml(format!(
            "retEnvEvento não encontrado na resposta. Início={}",
            response.chars().take(220).collect::<String>()
        ))
    })?;

    let ret: RetEnvEvento<T> = quick_xml::de::from_str(subtree)
        .map_err(|e| DfeError::Xml(format!("Erro ao desserializar retEnvEvento: {e}")))?;

    match ret.ret_evento {
        Some(re) => Ok(re.inf_evento),
        // Sem <retEvento>: o lote inteiro foi rejeitado — o motivo vem no nível do retEnvEvento.
        None => Err(DfeError::Xml(format!(
            "Lote de evento rejeitado (cStat {}): {}",
            ret.c_stat, ret.x_motivo
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Golden do envelope envEvento — capturado da impl anterior (idêntica em cancelar/manifestacao,
    // agora unificada aqui). idLote (volátil) é redigido. Deve casar byte-a-byte com o antigo
    // golden `cancelar::tests::golden_envelope_xml`.
    #[test]
    fn golden_env_evento_xml() {
        let out = env_evento_xml("<infEvento>FIXO</infEvento>", "<Signature>FIXO</Signature>").unwrap();
        insta::with_settings!({filters => vec![
            (r"<idLote>.*?</idLote>", "<idLote>[LOTE]</idLote>"),
        ]}, {
            insta::assert_snapshot!(out);
        });
    }

    use crate::tipos::cancelar::InfEvento as CancelarInfEvento;
    use crate::tipos::manifestacao::InfEvento as ManifestacaoInfEvento;

    // Resposta SOAP completa e representativa do NFeRecepcaoEvento4 (soap12 + wrapper +
    // retEnvEvento + retEvento + infEvento). O wrapper aqui é `nfeResultMsg`; o parse deve
    // funcionar independentemente do nome do wrapper (isola o retEnvEvento por nome estável).
    fn resposta_cancelamento_ok() -> String {
        r#"<?xml version="1.0" encoding="utf-8"?><soap12:Envelope xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeResultMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4"><retEnvEvento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00"><idLote>202600000000001</idLote><tpAmb>2</tpAmb><verAplic>SP_EVENTOS</verAplic><cOrgao>35</cOrgao><cStat>128</cStat><xMotivo>Lote de Evento Processado</xMotivo><retEvento versao="1.00"><infEvento Id="ID110111350000000000000000005500100000000010000000010001"><tpAmb>2</tpAmb><verAplic>SP_EVENTOS</verAplic><cOrgao>35</cOrgao><cStat>135</cStat><xMotivo>Evento registrado e vinculado a NF-e</xMotivo><chNFe>35000000000000000000550010000000001000000001</chNFe><tpEvento>110111</tpEvento><xEvento>Cancelamento</xEvento><nSeqEvento>1</nSeqEvento><dhRegEvento>2026-07-05T10:00:00-03:00</dhRegEvento><nProt>135260000000001</nProt></infEvento></retEvento></retEnvEvento></nfeResultMsg></soap12:Body></soap12:Envelope>"#.to_string()
    }

    #[test]
    fn parse_ret_evento_cancelamento_ok() {
        let inf: CancelarInfEvento = parse_ret_evento(&resposta_cancelamento_ok()).unwrap();
        assert_eq!(inf.c_stat, "135");
        assert_eq!(inf.tp_evento, "110111");
        assert_eq!(inf.ch_nfe, "35000000000000000000550010000000001000000001");
        assert_eq!(inf.n_seq_evento, "1");
        assert_eq!(inf.x_motivo, "Evento registrado e vinculado a NF-e");
    }

    // Manifestação: o infEvento de retorno traz CNPJDest/nProt (campos a mais) — o parse tipado
    // deve preenchê-los; prova que `T` genérico funciona para a outra família de evento.
    #[test]
    fn parse_ret_evento_manifestacao_ok() {
        let resp = r#"<soap12:Envelope xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeResultMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4"><retEnvEvento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00"><idLote>1</idLote><tpAmb>2</tpAmb><verAplic>AN</verAplic><cOrgao>91</cOrgao><cStat>128</cStat><xMotivo>Lote processado</xMotivo><retEvento versao="1.00"><infEvento Id="ID210210x"><tpAmb>2</tpAmb><verAplic>AN</verAplic><cOrgao>91</cOrgao><cStat>135</cStat><xMotivo>Evento registrado e vinculado a NF-e</xMotivo><chNFe>35000000000000000000550010000000001000000001</chNFe><tpEvento>210210</tpEvento><xEvento>Ciencia da Operacao</xEvento><nSeqEvento>1</nSeqEvento><CNPJDest>11222333000181</CNPJDest><dhRegEvento>2026-07-05T10:00:00-03:00</dhRegEvento><nProt>135260000000009</nProt></infEvento></retEvento></retEnvEvento></nfeResultMsg></soap12:Body></soap12:Envelope>"#;
        let inf: ManifestacaoInfEvento = parse_ret_evento(resp).unwrap();
        assert_eq!(inf.c_stat, "135");
        assert_eq!(inf.tp_evento, "210210");
        assert_eq!(inf.cnpj_dest, "11222333000181");
        assert_eq!(inf.n_prot, "135260000000009");
    }

    // Lote rejeitado: retEnvEvento com cStat/xMotivo mas SEM retEvento → erro com o motivo do lote.
    #[test]
    fn parse_ret_evento_lote_rejeitado() {
        let resp = r#"<soap12:Envelope xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeResultMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4"><retEnvEvento xmlns="http://www.portalfiscal.inf.br/nfe" versao="1.00"><idLote>1</idLote><tpAmb>2</tpAmb><verAplic>SP</verAplic><cOrgao>35</cOrgao><cStat>215</cStat><xMotivo>Rejeicao: Falha no schema XML</xMotivo></retEnvEvento></nfeResultMsg></soap12:Body></soap12:Envelope>"#;
        let err = parse_ret_evento::<CancelarInfEvento>(resp).unwrap_err();
        match err {
            DfeError::Xml(m) => {
                assert!(m.contains("215"), "erro deve conter cStat de lote: {m}");
                assert!(m.contains("Falha no schema"), "erro deve conter xMotivo: {m}");
            }
            other => panic!("esperado DfeError::Xml, veio {other:?}"),
        }
    }

    // Resposta sem retEnvEvento (ex.: SOAP Fault) → erro claro, sem panicar.
    #[test]
    fn parse_ret_evento_sem_ret_env_evento() {
        let resp = r#"<soap12:Envelope><soap12:Body><soap12:Fault><Reason>erro</Reason></soap12:Fault></soap12:Body></soap12:Envelope>"#;
        assert!(parse_ret_evento::<CancelarInfEvento>(resp).is_err());
    }
}
