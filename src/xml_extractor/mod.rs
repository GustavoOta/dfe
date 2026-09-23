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

impl XmlExtractor {
    /// Lê o XML que vai ser **impresso** (DANFE em PDF ou cupom ESC/POS).
    ///
    /// Aceita o `nfeProc` autorizado, como [`XmlExtractorSignature::nfe_proc_from_string`], e
    /// também o `<NFe>` sem protocolo — **só** quando é NFC-e (mod 65) em contingência off-line
    /// (`tpEmis=9`). Essa nota é assinada no caixa e impressa na hora, antes de a SEFAZ
    /// autorizar: não existe `protNFe` para embrulhar. Ela sai com `versao` do `infNFe` e
    /// `prot_nfe.inf_prot = None` (protocolo em branco), e os layouts desenham o aviso de
    /// contingência a partir do `tpEmis`.
    ///
    /// Qualquer outro `<NFe>` sem protocolo continua recusado: DANFE de nota normal não
    /// autorizada não tem valor e não pode sair do caixa.
    pub fn nfe_proc_para_impressao(&self, xml: &str) -> Result<NFeProc> {
        let erro_proc = match self.nfe_proc_from_string(xml) {
            Ok(proc_) => return Ok(proc_),
            Err(e) => e,
        };
        if xml.contains("<nfeProc") {
            return Err(erro_proc);
        }
        let nfe = match self.nfe_from_string(xml) {
            Ok(nfe) => nfe,
            Err(_) => return Err(erro_proc),
        };

        let ide = &nfe.inf_nfe.ide;
        let contingencia_offline =
            ide.mod_.as_deref() == Some("65") && ide.tp_emis.as_deref() == Some("9");
        if !contingencia_offline {
            return Err(DfeError::Xml(
                "XML sem protocolo de autorização (nfeProc): só a NFC-e em contingência                  off-line (mod 65, tpEmis=9) pode ser impressa antes de autorizada."
                    .to_string(),
            ));
        }

        let versao = nfe.inf_nfe.versao.clone().unwrap_or_else(|| "4.00".to_string());
        Ok(NFeProc {
            versao,
            nfe,
            prot_nfe: ProtNFe { inf_prot: None },
        })
    }

    /// [`Self::nfe_proc_para_impressao`] a partir de um arquivo.
    pub fn nfe_proc_para_impressao_from_file(&self, file_path: &str) -> Result<NFeProc> {
        let xml = std::fs::read_to_string(file_path)
            .map_err(|e| DfeError::Io(format!("Failed to open file: {} [{}]", file_path, e)))?;
        self.nfe_proc_para_impressao(&xml)
    }
}

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
