use super::actions::*;
use super::validations::*;
use base64::Engine;
use std::path::Path;

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
    pub qr_side: bool,
    /// Caminho de arquivo (PNG/JPG) ou string base64 do logotipo do emitente.
    /// Data URIs (`data:image/png;base64,...`) também são aceitos.
    pub logo: Option<&'a str>,
}

impl<'a> DanfeBuilder<'a> {
    pub fn new() -> Self {
        Self {
            xml: None,
            paper_size: None,
            as_base64: None,
            as_file: None,
            qr_side: false,
            logo: None,
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

    /// Define o logotipo do emitente.
    /// Aceita: caminho de arquivo (.png / .jpg / .jpeg), base64 puro ou data URI.
    pub fn logo(mut self, logo: &'a str) -> Self {
        self.logo = Some(logo);
        self
    }

    /// Usa layout lateral para o QR Code (NFC-e 80mm):
    /// QR Code à esquerda (~33 mm) com chave de acesso e protocolo à direita.
    /// Padrão: QR Code centralizado.
    pub fn qr_side(mut self) -> Self {
        self.qr_side = true;
        self
    }

    /// Constrói e gera o PDF, retornando o resultado.
    /// Result<String, String> - Ok(String) com o resultado da geração ou Err(String) com a mensagem de erro.
    /// Ok(String) pode ser o caminho do arquivo gerado ou a string base64, dependendo da configuração escolhida.
    pub async fn build(self) -> Result<String, String> {
        // Validações:
        let nfe_proc = Validations::init(&self)?;

        // Carrega logo (opcional)
        let logo_bytes: Option<Vec<u8>> = match self.logo {
            Some(src) => Some(Self::load_logo_bytes(src)?),
            None => None,
        };

        // Ações para geração do PDF pelo tamanho do papel
        let paper_size = self.paper_size.unwrap_or("a4");
        let mod_ = match nfe_proc.nfe.inf_nfe.ide.mod_.clone() {
            Some(m) => m,
            None => format!("Campo 'mod' ausente no XML"),
        };
        let pdf_bytes: Vec<u8> = match paper_size {
            "a4" => {
                match mod_.as_str() {
                    "55" => DanfeBuilderActions::generate_55_a4(nfe_proc, logo_bytes).await?,
                    "65" => return Err("Geração de DANFE para modelo 65 em formato A4 ainda não implementada".to_string()),
                    _ => return Err(format!("Modelo de documento inválido: {}. O DANFE é gerado apenas para o modelo 55 ou 65.", mod_)),
                }
            }
            "80mm" => {
                match mod_.as_str() {
                    "55" => DanfeBuilderActions::generate_55_80mm(nfe_proc).await?,
                    "65" => DanfeBuilderActions::generate_65_80mm(nfe_proc, self.qr_side).await?,
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

    fn load_logo_bytes(src: &str) -> Result<Vec<u8>, String> {
        // Data URI: data:image/png;base64,<dados>
        if src.starts_with("data:") {
            let b64 = src
                .splitn(2, ',')
                .nth(1)
                .ok_or("Data URI inválida: falta vírgula separadora")?;
            return base64::engine::general_purpose::STANDARD
                .decode(b64)
                .map_err(|e| format!("Erro ao decodificar logo data URI: {}", e));
        }

        // Caminho de arquivo com extensão de imagem
        let ext = Path::new(src)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if matches!(ext.as_str(), "png" | "jpg" | "jpeg") {
            return std::fs::read(src)
                .map_err(|e| format!("Erro ao ler arquivo de logo '{}': {}", src, e));
        }

        // Base64 puro
        base64::engine::general_purpose::STANDARD
            .decode(src)
            .map_err(|e| format!("Erro ao decodificar logo base64: {}", e))
    }
}
