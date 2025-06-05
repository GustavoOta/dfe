pub mod structs;

use quick_xml::de::from_str;
use quick_xml::de::DeError;
use std::io::Read;
use structs::*;

pub trait XmlExtractorSignature {
    fn new() -> Self;
    fn nfe_proc_from_string(&self, xml: &str) -> Result<NFeProc, XMLExtractorError>;
    fn nfe_proc_from_file(&self, file_path: &str) -> Result<NFeProc, XMLExtractorError>;
    fn nfe_from_string(&self, xml: &str) -> Result<NFe, XMLExtractorError>;
    fn nfe_from_file(&self, file_path: &str) -> Result<NFe, XMLExtractorError>;
}

pub struct XmlExtractor;

impl XmlExtractorSignature for XmlExtractor {
    fn new() -> Self {
        XmlExtractor
    }

    fn nfe_proc_from_string(&self, xml: &str) -> Result<NFeProc, XMLExtractorError> {
        if xml.is_empty() {
            return Err(XMLExtractorError {
                error: 0,
                msg: "XML string is empty".to_string(),
                data: None,
            });
        }

        let nfe: Result<NFeProc, DeError> = from_str(xml);
        if nfe.is_err() {
            return Err(XMLExtractorError {
                error: 1,
                msg: format!("Failed to parse XML: {}", nfe.unwrap_err()),
                data: Some(xml.to_string()),
            });
        }
        let nfe = nfe.unwrap();
        return Ok(nfe);
    }

    fn nfe_proc_from_file(&self, file_path: &str) -> Result<NFeProc, XMLExtractorError> {
        let file = std::fs::File::open(file_path).map_err(|e| XMLExtractorError {
            error: 1,
            msg: format!("Failed to open file: {} [{}]", file_path, e),
            data: None,
        })?;
        let mut reader = std::io::BufReader::new(file);
        let mut xml_content = String::new();
        reader
            .read_to_string(&mut xml_content)
            .map_err(|e| XMLExtractorError {
                error: 2,
                msg: format!("Failed to read file: {} [{}]", file_path, e),
                data: None,
            })?;
        self.nfe_proc_from_string(&xml_content)
    }

    fn nfe_from_string(&self, xml: &str) -> Result<NFe, XMLExtractorError> {
        if xml.is_empty() {
            return Err(XMLExtractorError {
                error: 0,
                msg: "XML string is empty".to_string(),
                data: None,
            });
        }

        // Tenta extrair apenas o conteúdo da tag <NFe>...</NFe>
        let start = xml.find("<NFe").ok_or(XMLExtractorError {
            error: 1,
            msg: "Tag <NFe> não encontrada".to_string(),
            data: Some(xml.to_string()),
        })?;
        let end = xml[start..].find("</NFe>").ok_or(XMLExtractorError {
            error: 1,
            msg: "Tag </NFe> não encontrada".to_string(),
            data: Some(xml.to_string()),
        })? + start
            + "</NFe>".len();

        let nfe_xml = &xml[start..end];

        let nfe: Result<NFe, DeError> = from_str(nfe_xml);
        if nfe.is_err() {
            return Err(XMLExtractorError {
                error: 1,
                msg: format!("Failed to parse XML: {}", nfe.unwrap_err()),
                data: Some(nfe_xml.to_string()),
            });
        }
        let nfe = nfe.unwrap();
        return Ok(nfe);
    }

    fn nfe_from_file(&self, file_path: &str) -> Result<NFe, XMLExtractorError> {
        let file = std::fs::File::open(file_path).map_err(|e| XMLExtractorError {
            error: 1,
            msg: format!("Failed to open file: {} [{}]", file_path, e),
            data: None,
        })?;
        let mut reader = std::io::BufReader::new(file);
        let mut xml_content = String::new();
        reader
            .read_to_string(&mut xml_content)
            .map_err(|e| XMLExtractorError {
                error: 2,
                msg: format!("Failed to read file: {} [{}]", file_path, e),
                data: None,
            })?;
        self.nfe_from_string(&xml_content)
    }
}
