use anyhow::{Error, Result};

pub fn is_xml_valid(xml: &str, xsd: &str) -> Result<bool, Error> {
    // clean \ backslash
    let xml = xml.replace("\\", "");
    // clean /n and /r white space
    let xml = xml.replace("\n", "");
    let xml = xml.replace("\r", "");
    // clean /t tab space
    let xml = xml.replace("\t", "");
    // clean / white space
    let xml = libxml::parser::Parser::default().parse_string(xml)?;

    let mut xsdparser = libxml::schemas::SchemaParserContext::from_file(xsd);

    let xsd = libxml::schemas::SchemaValidationContext::from_parser(&mut xsdparser);

    if let Err(errors) = xsd {
        let mut messages = Vec::new();
        for err in &errors {
            messages.push(err.message.as_ref().unwrap().to_string());
        }
        return Err(Error::msg(messages.join("\n")));
    }

    let mut xsd = xsd.unwrap();

    if let Err(errors) = xsd.validate_document(&xml) {
        let mut messages = Vec::new();
        for err in &errors {
            messages.push(err.message.as_ref().unwrap().to_string());
        }
        return Err(Error::msg(messages.join("\n")));
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xml_valid() {
        let xml = "<NFe></NFe>";
        let xsd = "./dfe/shema/PL_009p_NT2024_003_v1.02/nfe_v4.00.xsd";
        let result = is_xml_valid(&xml, xsd);
        if result.is_err() {
            println!("Error test_is_xml_valid:{:?}", result.err());
            assert!(false);
            return;
        }
        let result = result.unwrap();
        assert_eq!(result, true);
    }
}
