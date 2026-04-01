use super::actions::*;
use super::validations::*;
use anyhow::Result;
use base64::Engine;

/// Builder para geração de DANFE (PDF) a partir de XML.
///
/// # Exemplo
///
/// ```rust
/// use dfe::pdf::DanfeBuilder;
///
/// let output_as_file = DanfeBuilder::new()
///     .xml_from_file("./nota.xml")
///     .paper_size("80mm")
///     .as_file("./danfe.pdf")
///     .build()
///     .await;
///
/// let output_as_base64 = DanfeBuilder::new()
///     .xml_from_file("./nota.xml")
///     .paper_size("a4")
///     .as_base64()
///     .build()
///     .await;
/// ```
#[derive(Debug, Clone)]
pub struct DanfeBuilder<'a> {
    pub xml: Option<&'a str>,
    pub paper_size: Option<&'a str>,
    pub as_base64: Option<&'a str>,
    pub as_file: Option<&'a str>,
}

impl<'a> DanfeBuilder<'a> {
    pub fn new() -> Self {
        Self {
            xml: None,
            paper_size: None,
            as_base64: None,
            as_file: None,
        }
    }
    /// Configura o XML a partir de uma string ou caminho de arquivo.
    /// O método detecta automaticamente se a entrada é um caminho de arquivo ou uma string XML.
    /// Se a string terminar com ".xml", será tratada como um caminho de arquivo; caso contrário, será tratada como XML bruto.
    pub fn xml(mut self, xml: &'a str) -> Self {
        self.xml = Some(xml);
        self
    }

    /// Configura o tamanho do papel para o DANFE.
    /// Exemplos de tamanhos: "a4", "80mm", "54mm".
    /// Se não enviado, o padrão será "a4".
    pub fn paper_size(mut self, size: &'a str) -> Self {
        self.paper_size = Some(size);
        self
    }

    /// Configura a saída do DANFE como base64.
    pub fn as_base64(mut self) -> Self {
        self.as_base64 = Some("base64");
        self
    }

    /// Configura a saída do DANFE como arquivo PDF.
    /// O caminho do arquivo deve ser fornecido como argumento.
    pub fn as_file(mut self, path: &'a str) -> Self {
        self.as_file = Some(path);
        self
    }

    /// Constrói e gera o PDF, retornando o resultado.
    /// Result<String, String> - Ok(String) com o resultado da geração ou Err(String) com a mensagem de erro.
    /// Ok(String) pode ser o caminho do arquivo gerado ou a string base64, dependendo da configuração escolhida.
    pub async fn build(self) -> Result<String, String> {
        // Validações:
        let nfe_proc = Validations::init(&self)?;

        // Ações para geração do PDF pelo tamanho do papel
        let paper_size = self.paper_size.unwrap_or("a4");
        let mod_ = match nfe_proc.nfe.inf_nfe.ide.mod_.clone() {
            Some(m) => m,
            None => format!("Campo 'mod' ausente no XML"),
        };
        let pdf_bytes: Vec<u8> = match paper_size {
            "a4" => {
                match mod_.as_str() {
                    "55" => return Err("Geração de DANFE para modelo 55 em formato A4 ainda não implementada".to_string()),
                    "65" => return Err("Geração de DANFE para modelo 65 em formato A4 ainda não implementada".to_string()),
                    _ => return Err(format!("Modelo de documento inválido: {}. O DANFE é gerado apenas para o modelo 55 ou 65.", mod_)),
                }
            }
            "80mm" => {
                match mod_.as_str() {
                    "55" => DanfeBuilderActions::generate_55_80mm(nfe_proc).await?,
                    "65" => return Err("Geração de DANFE para modelo 65 em formato 80mm ainda não implementada".to_string()),
                    _ => return Err(format!("Modelo de documento inválido: {}. O DANFE é gerado apenas para o modelo 55 ou 65.", mod_)),
                }
            }
            "54mm" => {
                match mod_.as_str() {
                    "55" => return Err("Geração de DANFE para modelo 55 em formato 54mm ainda não implementada".to_string()),
                    "65" => return Err("Geração de DANFE para modelo 65 em formato 54mm ainda não implementada".to_string()),
                    _ => return Err(format!("Modelo de documento inválido: {}. O DANFE é gerado apenas para o modelo 55 ou 65.", mod_)),
                }
            }
            _ => {
                return Err(format!(
                    "Tamanho de papel inválido: {}. Os tamanhos válidos são 'a4', '80mm' ou '54mm'.",
                    self.paper_size.unwrap_or("a4")
                ))
            }
        };

        // Saída: salvar como arquivo ou retornar como base64
        if let Some(file_path) = self.as_file {
            std::fs::write(file_path, &pdf_bytes)
                .map_err(|e| format!("Erro ao salvar o arquivo PDF: {}", e))?;
            Ok(file_path.to_string())
        } else {
            let base64_pdf = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);
            Ok(base64_pdf)
        }
    }
}
