//! Parse da resposta da SEFAZ à emissão (`retEnviNFe`/`protNFe`). Extraído do `mod.rs` na fase
//! A7 (§2.1). Mantém o recorte por regex do `<protNFe>` (a troca por serde é candidata futura,
//! análoga à A5 dos eventos).

use crate::error::{DfeError, Result};
use regex::Regex;

use super::types::{Alerta, InfProt, Response, TagInfProt};

/// Máximo de alertas por nota (NT 2026.002 §2.2): a SEFAZ devolve só os 5 primeiros.
const MAX_ALERTAS: usize = 5;

pub(super) fn xml_result(response: &str, validated_xml: String) -> Result<Response> {
    let re = Regex::new(r#"<protNFe versao="4.00">(.*?)</protNFe>"#)
        .map_err(|e| DfeError::Xml(format!("Erro regex: {}", e)))?;
    if let Some(prot_nfe) = re.captures(response).and_then(|c| c.get(0)).map(|m| m.as_str()) {
        let mut tag_inf_prot: TagInfProt = quick_xml::de::from_str(prot_nfe)
            .map_err(|e| DfeError::Xml(format!("Erro desserializar protocolo: {} — {}", e, prot_nfe)))?;
        tag_inf_prot.inf_prot.alertas = extrair_alertas(prot_nfe);
        return Ok(Response {
            protocolo: tag_inf_prot,
            xml: validated_xml,
            send_xml: String::new(),
            receive_xml: String::new(),
        });
    }

    // Sem <protNFe>: o SEFAZ rejeitou o LOTE inteiro (ex.: cStat 225 — "Falha no
    // Schema XML do lote de NFe"). Nesse caso o cStat/xMotivo vêm no nível do
    // retEnviNFe, sem protocolo por nota. Monta um protocolo sintético com o
    // status do lote (c_stat != 100) para que a rejeição siga o mesmo caminho de
    // uma rejeição por nota, em vez de mascarar o motivo real como "protNFe not found".
    if let Some(c_stat) = extract_xml_tag(response, "cStat").and_then(|s| s.parse::<i32>().ok()) {
        let inf_prot = InfProt {
            tp_amb: extract_xml_tag(response, "tpAmb").and_then(|s| s.parse().ok()).unwrap_or(0),
            ver_aplic: extract_xml_tag(response, "verAplic").unwrap_or_default(),
            ch_nfe: extract_xml_tag(response, "chNFe").unwrap_or_default(),
            dh_recbto: extract_xml_tag(response, "dhRecbto").unwrap_or_default(),
            n_prot: None,
            dig_val: None,
            c_stat,
            x_motivo: extract_xml_tag(response, "xMotivo").unwrap_or_default(),
            alertas: Vec::new(),
        };
        return Ok(Response {
            protocolo: TagInfProt { inf_prot },
            xml: validated_xml,
            send_xml: String::new(),
            receive_xml: String::new(),
        });
    }

    // Nem protocolo de nota, nem status de lote — resposta realmente inesperada.
    Err(DfeError::Xml(format!("protNFe not found: {}", response)))
}

/// Pares `cMsg`/`xMsg` (grupo PR13) do `<protNFe>`, na ordem em que vieram, até [`MAX_ALERTAS`].
/// Na autorização com alerta (`cStat 120`) cada alerta é um par; um `cMsg` sem `xMsg` é ignorado.
pub(super) fn extrair_alertas(prot_nfe: &str) -> Vec<Alerta> {
    let Ok(re) = Regex::new(r"(?s)<cMsg>(.*?)</cMsg>\s*<xMsg>(.*?)</xMsg>") else {
        return Vec::new();
    };
    re.captures_iter(prot_nfe)
        .take(MAX_ALERTAS)
        .map(|c| Alerta {
            c_msg: c[1].trim().to_string(),
            x_msg: c[2].trim().to_string(),
        })
        .collect()
}

/// Extrai o conteúdo da primeira ocorrência de `<tag>…</tag>` no XML de resposta.
/// Usado para ler o status no nível do lote (`retEnviNFe`) quando não há `<protNFe>`.
pub(super) fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?s)<{0}[^>]*>(.*?)</{0}>", regex::escape(tag))).ok()?;
    re.captures(xml)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_result_autorizacao_extrai_protocolo() {
        let resp = r#"<retEnviNFe versao="4.00"><protNFe versao="4.00"><infProt><tpAmb>2</tpAmb><verAplic>SP_NFCE</verAplic><chNFe>3500000000000000000065010000000011234567890</chNFe><dhRecbto>2026-06-22T11:09:10-03:00</dhRecbto><nProt>135260000000001</nProt><digVal>abc=</digVal><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe></retEnviNFe>"#;
        let r = xml_result(resp, "<xml/>".into()).expect("deve extrair protocolo");
        assert_eq!(r.protocolo.inf_prot.c_stat, 100);
        assert_eq!(r.protocolo.inf_prot.n_prot.as_deref(), Some("135260000000001"));
    }

    #[test]
    fn xml_result_rejeicao_de_lote_vira_protocolo_sintetico() {
        // retEnviNFe sem <protNFe>: o lote inteiro falhou no schema (cStat 225).
        // Antes, isso virava o erro opaco "protNFe not found"; agora deve expor
        // cStat/xMotivo do nível do lote para o consumidor.
        let resp = r#"<retEnviNFe versao="4.00"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL_009_V400</verAplic><cStat>225</cStat><xMotivo>Rejeição: Falha no Schema XML do lote de NFe</xMotivo><cUF>35</cUF><dhRecbto>2026-06-22T11:09:10-03:00</dhRecbto></retEnviNFe>"#;
        let r = xml_result(resp, "<xml/>".into()).expect("rejeição de lote deve virar Ok com protocolo sintético");
        assert_eq!(r.protocolo.inf_prot.c_stat, 225);
        assert_eq!(r.protocolo.inf_prot.ver_aplic, "SP_NFCE_PL_009_V400");
        assert_eq!(r.protocolo.inf_prot.x_motivo, "Rejeição: Falha no Schema XML do lote de NFe");
        assert!(r.protocolo.inf_prot.n_prot.is_none());
    }

    #[test]
    fn xml_result_autorizacao_com_alerta_extrai_alertas() {
        // NT 2026.002 §2.3: cStat 120 + pares cMsg/xMsg (PR13) dentro do infProt.
        let resp = r#"<retEnviNFe versao="4.00"><protNFe versao="4.00"><infProt><tpAmb>2</tpAmb><verAplic>SP_NFCE</verAplic><chNFe>3500000000000000000065010000000011234567890</chNFe><dhRecbto>2026-10-13T11:09:10-03:00</dhRecbto><nProt>135260000000002</nProt><digVal>abc=</digVal><cStat>120</cStat><xMotivo>Autorizado o uso da NF-e, com alerta</xMotivo><cMsg>172</cMsg><xMsg>Alerta: Situacao do CNPJ destinatario inabilitado no momento da autorizacao</xMsg><cMsg>999</cMsg><xMsg>Outro alerta</xMsg></infProt></protNFe></retEnviNFe>"#;
        let r = xml_result(resp, "<xml/>".into()).expect("deve extrair protocolo");
        let inf = &r.protocolo.inf_prot;
        assert_eq!(inf.c_stat, 120);
        assert!(crate::cstat_autorizado(inf.c_stat));
        assert_eq!(inf.n_prot.as_deref(), Some("135260000000002"));
        assert_eq!(inf.alertas.len(), 2);
        assert_eq!(inf.alertas[0].c_msg, "172");
        assert!(inf.alertas[0].x_msg.contains("CNPJ destinatario inabilitado"));
        assert_eq!(inf.alertas[1].c_msg, "999");
    }

    #[test]
    fn alertas_limitados_a_cinco_e_ausentes_na_autorizacao_comum() {
        let pares: String = (1..=7).map(|i| format!("<cMsg>{i}</cMsg><xMsg>m{i}</xMsg>")).collect();
        assert_eq!(extrair_alertas(&pares).len(), 5);
        assert!(extrair_alertas("<infProt><cStat>100</cStat></infProt>").is_empty());
    }

    #[test]
    fn alertas_vazios_nao_aparecem_no_json() {
        let resp = r#"<retEnviNFe versao="4.00"><protNFe versao="4.00"><infProt><tpAmb>2</tpAmb><verAplic>SP</verAplic><chNFe>1</chNFe><dhRecbto>x</dhRecbto><cStat>100</cStat><xMotivo>ok</xMotivo></infProt></protNFe></retEnviNFe>"#;
        let r = xml_result(resp, "<xml/>".into()).unwrap();
        let json = serde_json::to_string(&r.protocolo).unwrap();
        assert!(!json.contains("alertas"), "contrato JSON da autorização comum não muda: {json}");
    }

    #[test]
    fn xml_result_sem_status_retorna_erro() {
        let resp = "<html><body>502 Bad Gateway</body></html>";
        let err = xml_result(resp, "<xml/>".into()).unwrap_err();
        assert!(matches!(err, DfeError::Xml(msg) if msg.contains("protNFe not found")));
    }
}
