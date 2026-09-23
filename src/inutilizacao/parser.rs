use crate::error::{DfeError, Result};
use crate::tipos::inutilizacao::InfInut;

// Mesma técnica de `consulta_situacao::parser` e `interno::evento::parse_ret_evento`: recorte
// textual do subtree de nome estável (o wrapper SOAP varia entre UFs) + desserialização tipada.
fn extract_subtree<'a>(xml: &'a str, local_name: &str) -> Option<&'a str> {
    let start = xml.find(&format!("<{local_name}"))?;
    let close = format!("</{local_name}>");
    let end = xml[start..].find(&close)? + start + close.len();
    Some(&xml[start..end])
}

pub fn parse_ret_inut_nfe(response: &str) -> Result<InfInut> {
    let subtree = extract_subtree(response, "infInut").ok_or_else(|| {
        DfeError::Xml(format!(
            "infInut não encontrado na resposta de inutilização. Início={}",
            response.chars().take(220).collect::<String>()
        ))
    })?;

    quick_xml::de::from_str(subtree)
        .map_err(|e| DfeError::Xml(format!("Erro ao desserializar infInut: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(inner: &str) -> String {
        format!(
            r#"<soap12:Envelope xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeResultMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeInutilizacao4">{inner}</nfeResultMsg></soap12:Body></soap12:Envelope>"#
        )
    }

    #[test]
    fn homologada() {
        let resp = envelope(
            r#"<retInutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><infInut Id="ID35261122233300018165001000000325000000325"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><cStat>102</cStat><xMotivo>Inutilizacao de numero homologado</xMotivo><cUF>35</cUF><ano>26</ano><CNPJ>11222333000181</CNPJ><mod>65</mod><serie>1</serie><nNFIni>325</nNFIni><nNFFin>325</nNFFin><dhRecbto>2026-09-20T10:00:00-03:00</dhRecbto><nProt>135260000000009</nProt></infInut></retInutNFe>"#,
        );
        let parsed = parse_ret_inut_nfe(&resp).unwrap();
        assert_eq!(parsed.c_stat, "102");
        assert_eq!(parsed.n_prot.as_deref(), Some("135260000000009"));
        assert_eq!(parsed.n_nf_ini.as_deref(), Some("325"));
    }

    // Recusa: a faixa já foi usada por uma nota autorizada. Vem sem nProt/ano/CNPJ — os campos
    // opcionais do tipo existem exatamente por causa deste retorno.
    #[test]
    fn recusada_sem_campos_opcionais() {
        let resp = envelope(
            r#"<retInutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><infInut><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL009</verAplic><cStat>563</cStat><xMotivo>Ja existe pedido de Inutilizacao com a mesma faixa de inutilizacao</xMotivo><cUF>35</cUF></infInut></retInutNFe>"#,
        );
        let parsed = parse_ret_inut_nfe(&resp).unwrap();
        assert_eq!(parsed.c_stat, "563");
        assert!(parsed.n_prot.is_none());
        assert!(parsed.cnpj.is_none());
    }

    #[test]
    fn resposta_sem_inf_inut() {
        let resp = r#"<soap12:Envelope><soap12:Body><soap12:Fault><Reason>erro</Reason></soap12:Fault></soap12:Body></soap12:Envelope>"#;
        assert!(parse_ret_inut_nfe(resp).is_err());
    }
}
