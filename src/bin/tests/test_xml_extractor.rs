#[cfg(test)]
use dfe::xml_extractor::*;

#[tokio::test]
async fn test_xml_extractor_string() {
    let xml_extractor = XmlExtractor::new();
    let xml_string = r#"<NFe><infNFe><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
    match xml_extractor.nfe_proc_from_string(xml_string) {
        Err(e) => println!("Error: {:?}", e),
        Ok(r) => println!("Result:\n{}", serde_json::to_string_pretty(&r).unwrap()),
    }
}

#[tokio::test]
async fn test_xml_extractor_file() {
    let xml_extractor = XmlExtractor::new();
    let file_path = "D:\\Projetos\\dfe-api\\xml_validation_error.xml";
    match xml_extractor.nfe_proc_from_file(file_path) {
        Err(e) => println!("Error: {:?}", e),
        Ok(r) => println!("Result:\n{}", serde_json::to_string_pretty(&r).unwrap()),
    }
}

#[tokio::test]
async fn test_xml_extractor_nfe_from_string() {
    let xml_extractor = XmlExtractor::new();
    let xml_string = r#"<NFe><infNFe><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
    match xml_extractor.nfe_from_string(xml_string) {
        Err(e) => println!("Error: {:?}", e),
        Ok(r) => println!("Result:\n{}", serde_json::to_string_pretty(&r).unwrap()),
    }
}

#[tokio::test]
async fn test_xml_extractor_nfe_from_file() {
    let xml_extractor = XmlExtractor::new();
    let file_path = "./nfe_request.xml";
    match xml_extractor.nfe_from_file(file_path) {
        Err(e) => println!("Error: {:?}", e),
        Ok(r) => println!("Result:\n{}", serde_json::to_string_pretty(&r).unwrap()),
    }
}
