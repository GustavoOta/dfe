fn uf_to_ibge_code(uf: &str) -> Result<&'static str, String> {
    match uf {
        "RO" => Ok("11"),
        "AC" => Ok("12"),
        "AM" => Ok("13"),
        "RR" => Ok("14"),
        "PA" => Ok("15"),
        "AP" => Ok("16"),
        "TO" => Ok("17"),
        "MA" => Ok("21"),
        "PI" => Ok("22"),
        "CE" => Ok("23"),
        "RN" => Ok("24"),
        "PB" => Ok("25"),
        "PE" => Ok("26"),
        "AL" => Ok("27"),
        "SE" => Ok("28"),
        "BA" => Ok("29"),
        "MG" => Ok("31"),
        "ES" => Ok("32"),
        "RJ" => Ok("33"),
        "SP" => Ok("35"),
        "PR" => Ok("41"),
        "SC" => Ok("42"),
        "RS" => Ok("43"),
        "MS" => Ok("50"),
        "MT" => Ok("51"),
        "GO" => Ok("52"),
        "DF" => Ok("53"),
        _ => Err(format!("UF invalida para cUF IBGE: {}", uf)),
    }
}

pub fn status_request_xml(environment: u8, uf: &str) -> Result<String, String> {
    if environment != 1 && environment != 2 {
        return Err("environment deve ser 1 (producao) ou 2 (homologacao)".to_string());
    }

    let c_uf = uf_to_ibge_code(uf)?;

    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?><soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeStatusServico4"><consStatServ xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>{}</tpAmb><cUF>{}</cUF><xServ>STATUS</xServ></consStatServ></nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        environment, c_uf
    ))
}
