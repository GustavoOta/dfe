/// Monta o XML de request do `consSitNFe` (não assinado — só `tpAmb`/`xServ`/`chNFe`,
/// conforme `leiauteConsSitNFe_v4.00.xsd`). Mirror de `status::xml::status_request_xml`.
pub fn cons_sit_nfe_request_xml(environment: u8, chave: &str) -> Result<String, String> {
    if environment != 1 && environment != 2 {
        return Err("environment deve ser 1 (producao) ou 2 (homologacao)".to_string());
    }

    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?><soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeConsultaProtocolo4"><consSitNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>{}</tpAmb><xServ>CONSULTAR</xServ><chNFe>{}</chNFe></consSitNFe></nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        environment, chave
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAVE: &str = "35000000000000000000550010000000001000000001";

    #[test]
    fn golden_cons_sit_nfe_request_xml() {
        let out = cons_sit_nfe_request_xml(2, CHAVE).unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn rejeita_environment_invalido() {
        assert!(cons_sit_nfe_request_xml(0, CHAVE).is_err());
        assert!(cons_sit_nfe_request_xml(3, CHAVE).is_err());
    }
}
