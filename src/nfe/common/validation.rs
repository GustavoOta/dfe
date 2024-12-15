pub fn is_xml_valid(xml: &str, xsd: &str) -> Result<bool, Vec<String>> {
    // clean \ backslash
    let xml = xml.replace("\\", "");
    // clean /n and /r white space
    let xml = xml.replace("\n", "");
    let xml = xml.replace("\r", "");
    // clean /t tab space
    let xml = xml.replace("\t", "");
    // clean / white space
    let xml = libxml::parser::Parser::default().parse_string(xml);

    if xml.is_err() {
        return Err(vec![xml.err().unwrap().to_string()]);
    }

    let xml = xml.unwrap();

    let mut xsdparser = libxml::schemas::SchemaParserContext::from_file(xsd);

    let xsd = libxml::schemas::SchemaValidationContext::from_parser(&mut xsdparser);

    if let Err(errors) = xsd {
        let mut messages = Vec::new();
        for err in &errors {
            messages.push(err.message.as_ref().unwrap().to_string());
        }
        return Err(messages);
    }

    let mut xsd = xsd.unwrap();

    if let Err(errors) = xsd.validate_document(&xml) {
        let mut messages = Vec::new();
        for err in &errors {
            messages.push(err.message.as_ref().unwrap().to_string());
        }
        return Err(messages);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xml_valid() {
        let xml = "D:/Projetos/dfe/shema/exemplo/consStatServ_v4.00.xml";
        let xsd = "D:/Projetos/dfe/shema/PL_009p_NT2024_003_v1.02/consStatServ_v4.00.xsd";
        let result = is_xml_valid(xml, xsd);
        assert_eq!(result, Ok(true));
    }
}
/*
//!
//! Example Usage of XSD Schema Validation
//!
use libxml::schemas::SchemaParserContext;
use libxml::schemas::SchemaValidationContext;

use libxml::parser::Parser;

fn main() {
  let xml = Parser::default()
    .parse_file("tests/resources/schema.xml")
    .expect("Expected to be able to parse XML Document from file");

  let mut xsdparser = SchemaParserContext::from_file("tests/resources/schema.xsd");
  let xsd = SchemaValidationContext::from_parser(&mut xsdparser);

  if let Err(errors) = xsd {
    for err in &errors {
      println!("{}", err.message.as_ref().unwrap());
    }

    panic!("Failed to parse schema");
  }

  let mut xsd = xsd.unwrap();

  if let Err(errors) = xsd.validate_document(&xml) {
    for err in &errors {
      println!("{}", err.message.as_ref().unwrap());
    }

    panic!("Invalid XML accoding to XSD schema");
  }
}
   */
