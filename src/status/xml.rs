pub fn status_request_xml(environment: u8, uf: &str) -> Result<String, String> {
    if environment != 1 && environment != 2 {
        return Err("environment deve ser 1 (producao) ou 2 (homologacao)".to_string());
    }

    // A6: mapa sigla→código IBGE unificado em `interno::uf` (antes duplicado aqui).
    let c_uf = crate::interno::uf::codigo_por_sigla(uf).map_err(|e| e.to_string())?;

    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?><soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeStatusServico4"><consStatServ xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>{}</tpAmb><cUF>{}</cUF><xServ>STATUS</xServ></consStatServ></nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        environment, c_uf
    ))
}
