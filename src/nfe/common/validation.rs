use anyhow::{Error, Result};
use std::fs::File;
use std::io::Write;

pub fn is_xml_valid(xml: &str, xsd: &str) -> Result<String, Error> {
    let raw_incoming_xml = xml;
    let xml = libxml::parser::Parser::default().parse_string(xml)?;

    let mut xsdparser = libxml::schemas::SchemaParserContext::from_file(xsd);

    let xsd = libxml::schemas::SchemaValidationContext::from_parser(&mut xsdparser);

    if let Err(errors) = xsd {
        let mut messages = Vec::new();
        for err in &errors {
            messages.push(err.message.as_ref().unwrap().to_string());
        }
        save_xml(&raw_incoming_xml)?;
        return Err(Error::msg(messages.join("\n")));
    }

    let mut xsd = xsd.unwrap();

    if let Err(errors) = xsd.validate_document(&xml) {
        let mut messages = Vec::new();
        for err in &errors {
            messages.push(err.message.as_ref().unwrap().to_string());
        }
        save_xml(&raw_incoming_xml)?;
        return Err(Error::msg(messages.join("\n")));
    }

    Ok(raw_incoming_xml.to_string())
}

fn save_xml(xml: &str) -> Result<(), Error> {
    // save xml_validation_error.xml
    let mut file = File::create("./xml_validation_error.xml")?;
    file.write_all(xml.as_bytes())?;
    Ok(())
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
        let _result = result.unwrap();
    }
}
