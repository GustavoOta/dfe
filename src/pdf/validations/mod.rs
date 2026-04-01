use super::builder::DanfeBuilder;
use crate::nfe::xml_extractor::{structs::*, XmlExtractor, XmlExtractorSignature};
use anyhow::Result;

pub struct Validations;

impl Validations {
    pub fn init(data: &DanfeBuilder) -> Result<NFeProc, String> {
        // check if is file or string
        let mut xml_content = String::new();
        if let Some(xml) = data.xml {
            if xml.ends_with(".xml") {
                xml_content = Self::xml_file_to_string(xml)?;
            } else {
                xml_content = xml.to_string();
            }
        }
        let nfe_proc = Self::convert_xml_to_struct(&xml_content)?;
        // Validações adicionais podem ser implementadas aqui, como:
        // - Verificar se o XML é válido e bem formado
        // - Verificar se o XML contém os campos necessários para gerar o DANFE
        // - Validar o tamanho do papel especificado
        // - Verificar se a saída solicitada (base64 ou arquivo) é válida
        match nfe_proc.nfe.inf_nfe.ide.mod_.clone() {
            Some(modelo) => {
                if modelo != "55" && modelo != "65" {
                    return Err(format!("Modelo de documento inválido: {}. O DANFE é gerado apenas para o modelo 55 ou 65.", modelo));
                }
                modelo
            }
            None => return Err("O campo modelo está ausente no XML".to_string()),
        };

        let paper_size = data.paper_size.unwrap_or("a4");
        if paper_size != "a4" && paper_size != "80mm" && paper_size != "54mm" {
            return Err(format!(
                "Tamanho de papel inválido: {}. Os tamanhos válidos são 'a4', '80mm' ou '54mm'.",
                paper_size
            ));
        }

        if data.as_base64.is_some() && data.as_file.is_some() {
            return Err("Configurações de saída conflitantes: não é possível gerar o DANFE como base64 e arquivo ao mesmo tempo. Por favor, escolha apenas uma opção.".to_string());
        }

        if data.as_base64.is_none() && data.as_file.is_none() {
            return Err("Nenhuma configuração de saída especificada: por favor, configure a saída do DANFE usando as_base64() ou as_file().".to_string());
        }

        // Se todas as validações passarem, retorne Ok com o conteúdo XML ou outra informação relevante

        // Se todas as validações passarem, retorne Ok com uma mensagem ou valor relevante
        Ok(nfe_proc)
    }

    fn xml_file_to_string(file_path: &str) -> Result<String, String> {
        std::fs::read_to_string(file_path).map_err(|e| format!("Erro ao ler o arquivo XML: {}", e))
    }

    fn convert_xml_to_struct(xml: &str) -> Result<NFeProc, String> {
        let extractor = XmlExtractor::new();
        let nfe_proc = extractor
            .nfe_proc_from_string(xml)
            .map_err(|e| format!("Erro ao converter XML para struct NFeProc: {}", e.msg))?;
        Ok(nfe_proc)
    }
}
