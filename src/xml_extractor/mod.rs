pub mod structs;

use crate::error::{DfeError, Result};
use quick_xml::de::from_str;
use std::io::Read;
use structs::*;

pub trait XmlExtractorSignature {
    fn new() -> Self;
    fn nfe_proc_from_string(&self, xml: &str) -> Result<NFeProc>;
    fn nfe_proc_from_file(&self, file_path: &str) -> Result<NFeProc>;
    fn nfe_from_string(&self, xml: &str) -> Result<NFe>;
    fn nfe_from_file(&self, file_path: &str) -> Result<NFe>;
}

pub struct XmlExtractor;

impl XmlExtractorSignature for XmlExtractor {
    fn new() -> Self { XmlExtractor }

    fn nfe_proc_from_string(&self, xml: &str) -> Result<NFeProc> {
        if xml.is_empty() {
            return Err(DfeError::Xml("O XML enviado está vazio.".to_string()));
        }
        from_str(xml).map_err(|e| DfeError::Xml(format!(
            "Formato incompatível com NFeProc: {:?}", e
        )))
    }

    fn nfe_proc_from_file(&self, file_path: &str) -> Result<NFeProc> {
        let file = std::fs::File::open(file_path)
            .map_err(|e| DfeError::Io(format!("Failed to open file: {} [{}]", file_path, e)))?;
        let mut reader = std::io::BufReader::new(file);
        let mut xml_content = String::new();
        reader.read_to_string(&mut xml_content)
            .map_err(|e| DfeError::Io(format!("Failed to read file: {} [{}]", file_path, e)))?;
        self.nfe_proc_from_string(&xml_content)
    }

    fn nfe_from_string(&self, xml: &str) -> Result<NFe> {
        if xml.is_empty() {
            return Err(DfeError::Xml("XML string is empty".to_string()));
        }
        let start = xml.find("<NFe").ok_or_else(|| DfeError::Xml("Tag <NFe> não encontrada".to_string()))?;
        let end = xml[start..].find("</NFe>")
            .ok_or_else(|| DfeError::Xml("Tag </NFe> não encontrada".to_string()))?
            + start + "</NFe>".len();
        let nfe_xml = &xml[start..end];
        from_str(nfe_xml).map_err(|e| DfeError::Xml(format!("Failed to parse XML: {}", e)))
    }

    fn nfe_from_file(&self, file_path: &str) -> Result<NFe> {
        let file = std::fs::File::open(file_path)
            .map_err(|e| DfeError::Io(format!("Failed to open file: {} [{}]", file_path, e)))?;
        let mut reader = std::io::BufReader::new(file);
        let mut xml_content = String::new();
        reader.read_to_string(&mut xml_content)
            .map_err(|e| DfeError::Io(format!("Failed to read file: {} [{}]", file_path, e)))?;
        self.nfe_from_string(&xml_content)
    }
}
