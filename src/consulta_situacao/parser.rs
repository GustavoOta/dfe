use crate::error::{DfeError, Result};
use crate::tipos::consulta_situacao::RetConsSitNFe;

// Mesma técnica de `interno::evento::parse_ret_evento` (A5): recorte textual só para isolar o
// subtree de nome estável (o wrapper SOAP varia), seguido de desserialização tipada via
// `quick_xml::de`. Duplicada aqui (função pequena, 4 linhas) em vez de promover para `interno`
// agora — se aparecer um 3º consumidor, promover.
fn extract_subtree<'a>(xml: &'a str, local_name: &str) -> Option<&'a str> {
    let start = xml.find(&format!("<{local_name}"))?;
    let close = format!("</{local_name}>");
    let end = xml[start..].find(&close)? + start + close.len();
    Some(&xml[start..end])
}

pub fn parse_ret_cons_sit_nfe(response: &str) -> Result<RetConsSitNFe> {
    let subtree = extract_subtree(response, "retConsSitNFe").ok_or_else(|| {
        DfeError::Xml(format!(
            "retConsSitNFe não encontrado na resposta. Início={}",
            response.chars().take(220).collect::<String>()
        ))
    })?;

    quick_xml::de::from_str(subtree)
        .map_err(|e| DfeError::Xml(format!("Erro ao desserializar retConsSitNFe: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAVE: &str = "35000000000000000000550010000000001000000001";

    fn envelope(inner: &str) -> String {
        format!(
            r#"<soap12:Envelope xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeResultMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeConsultaProtocolo4">{inner}</nfeResultMsg></soap12:Body></soap12:Envelope>"#
        )
    }

    #[test]
    fn autorizada() {
        let resp = envelope(&format!(
            r#"<retConsSitNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo><cUF>35</cUF><dhRecbto>2026-07-10T10:00:00-03:00</dhRecbto><chNFe>{CHAVE}</chNFe><protNFe versao="4.00"><infProt Id="ID1"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><chNFe>{CHAVE}</chNFe><dhRecbto>2026-07-10T10:00:00-03:00</dhRecbto><nProt>135260000000001</nProt><digVal>ABC123==</digVal><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe></retConsSitNFe>"#
        ));
        let parsed = parse_ret_cons_sit_nfe(&resp).unwrap();
        assert_eq!(parsed.c_stat, "100");
        let prot = parsed.prot_nfe.expect("protNFe deveria estar presente");
        assert_eq!(prot.inf_prot.c_stat, "100");
        assert_eq!(prot.inf_prot.n_prot.as_deref(), Some("135260000000001"));
        assert!(parsed.proc_evento_nfe.is_empty());
    }

    #[test]
    fn nao_consta_na_base() {
        // cStat 217 no nível da consulta = "NF-e não consta na base de dados da SEFAZ" — sem protNFe.
        let resp = envelope(&format!(
            r#"<retConsSitNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><cStat>217</cStat><xMotivo>NF-e nao consta na base de dados da SEFAZ</xMotivo><cUF>35</cUF><dhRecbto>2026-07-10T10:00:00-03:00</dhRecbto><chNFe>{CHAVE}</chNFe></retConsSitNFe>"#
        ));
        let parsed = parse_ret_cons_sit_nfe(&resp).unwrap();
        assert_eq!(parsed.c_stat, "217");
        assert!(parsed.prot_nfe.is_none());
        assert!(parsed.proc_evento_nfe.is_empty());
    }

    #[test]
    fn autorizada_e_cancelada() {
        let resp = envelope(&format!(
            r#"<retConsSitNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><cStat>101</cStat><xMotivo>Cancelamento de NF-e homologado</xMotivo><cUF>35</cUF><dhRecbto>2026-07-10T10:00:00-03:00</dhRecbto><chNFe>{CHAVE}</chNFe><protNFe versao="4.00"><infProt Id="ID1"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><chNFe>{CHAVE}</chNFe><dhRecbto>2026-07-10T10:00:00-03:00</dhRecbto><nProt>135260000000001</nProt><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe><procEventoNFe versao="1.00"><evento versao="1.00"><infEvento Id="ID110111{CHAVE}01"><cOrgao>35</cOrgao><tpAmb>2</tpAmb><CNPJ>11222333000181</CNPJ><chNFe>{CHAVE}</chNFe><dhEvento>2026-07-10T11:00:00-03:00</dhEvento><tpEvento>110111</tpEvento><nSeqEvento>1</nSeqEvento><verEvento>1.00</verEvento><detEvento versao="1.00"/></infEvento></evento><retEvento versao="1.00"><infEvento Id="ID110111"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><cOrgao>35</cOrgao><cStat>135</cStat><xMotivo>Evento registrado e vinculado a NF-e</xMotivo><chNFe>{CHAVE}</chNFe><tpEvento>110111</tpEvento><nSeqEvento>1</nSeqEvento><dhRegEvento>2026-07-10T11:00:01-03:00</dhRegEvento><nProt>135260000000002</nProt></infEvento></retEvento></procEventoNFe></retConsSitNFe>"#
        ));
        let parsed = parse_ret_cons_sit_nfe(&resp).unwrap();
        assert_eq!(parsed.c_stat, "101");
        assert!(parsed.prot_nfe.is_some());
        assert_eq!(parsed.proc_evento_nfe.len(), 1);
        let evento = &parsed.proc_evento_nfe[0].ret_evento.inf_evento;
        assert_eq!(evento.tp_evento, "110111");
        assert_eq!(evento.c_stat, "135");
        assert_eq!(evento.n_prot.as_deref(), Some("135260000000002"));
    }

    #[test]
    fn resposta_sem_ret_cons_sit_nfe() {
        let resp = r#"<soap12:Envelope><soap12:Body><soap12:Fault><Reason>erro</Reason></soap12:Fault></soap12:Body></soap12:Envelope>"#;
        assert!(parse_ret_cons_sit_nfe(resp).is_err());
    }
}
