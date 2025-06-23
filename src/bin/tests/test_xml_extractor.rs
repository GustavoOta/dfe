#[cfg(test)]
use dfe::nfe::xml_extractor::*;
#[tokio::test]
async fn test_xml_extractor_string() {
    use dfe::nfe::xml_extractor::structs::*;
    let xml_extractor = XmlExtractor::new();

    // Test extracting from a string
    let xml_string = r#"<NFe><infNFe><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
    let result: Result<NFeProc, XMLExtractorError> = xml_extractor.nfe_proc_from_string(xml_string);
    if result.is_err() {
        println!("Error: {:?}", result.unwrap_err());
    } else {
        println!(
            "Result:\n{}",
            serde_json::to_string_pretty(&result.clone().unwrap()).unwrap()
        );
    }
}

#[tokio::test]
async fn test_xml_extractor_file() {
    use dfe::nfe::xml_extractor::structs::*;
    let xml_extractor = XmlExtractor::new();

    // Test extracting from a file
    let file_path = "D:\\Projetos\\dfe-api\\xml_validation_error.xml";
    let result: Result<NFeProc, XMLExtractorError> = xml_extractor.nfe_proc_from_file(file_path);
    if result.is_err() {
        println!("Error: {:?}", result.unwrap_err());
    } else {
        println!(
            "Result:\n{}",
            serde_json::to_string_pretty(&result.clone().unwrap()).unwrap()
        );
    }
}

#[tokio::test]
async fn test_xml_extractor_nfe_from_string() {
    use dfe::nfe::xml_extractor::structs::*;
    let xml_extractor = XmlExtractor::new();

    // Test extracting NFe from a string
    let xml_string = r#"<NFe><infNFe><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
    let result: Result<NFe, XMLExtractorError> = xml_extractor.nfe_from_string(xml_string);
    if result.is_err() {
        println!("Error: {:?}", result.unwrap_err());
    } else {
        println!(
            "Result:\n{}",
            serde_json::to_string_pretty(&result.clone().unwrap()).unwrap()
        );
    }
}

#[tokio::test]
async fn test_xml_extractor_nfe_from_file() {
    use dfe::nfe::xml_extractor::structs::*;
    let xml_extractor = XmlExtractor::new();

    // Test extracting NFe from a file
    let file_path = "./nfe_request.xml";
    let result: Result<NFe, XMLExtractorError> = xml_extractor.nfe_from_file(file_path);
    if result.is_err() {
        println!("Error: {:?}", result.unwrap_err());
    } else {
        println!(
            "Result:\n{}",
            serde_json::to_string_pretty(&result.clone().unwrap()).unwrap()
        );
    }
}
