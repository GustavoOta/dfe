mod commands;
pub mod nfce;
pub use nfce::EscPosNFCeBuilder;

use image::{io::Reader as ImageReader, DynamicImage, GenericImageView};
use std::io::Cursor;

/// Builder fluente para geração de comandos **ESC/POS** (Epson Standard Code for Printers).
///
/// Produz um `Vec<u8>` pronto para enviar à impressora via porta serial, USB ou rede.
///
/// # Exemplo
///
/// ```
/// use dfe::EscPosBuilder;
///
/// let bytes = EscPosBuilder::new()
///     .paper_width(80)
///     .align_center()
///     .bold(true)
///     .text("EMPRESA LTDA\n")
///     .bold(false)
///     .align_left()
///     .text("CNPJ: 11.222.333/0001-81\n")
///     .divider()
///     .cut()
///     .build();
///
/// assert!(!bytes.is_empty());
/// // std::fs::write("\\\\.\\COM3", &bytes).unwrap(); // Windows
/// // std::fs::write("/dev/usb/lp0", &bytes).unwrap(); // Linux
/// ```
pub struct EscPosBuilder {
    buffer: Vec<u8>,
    paper_width: u8,
}

impl EscPosBuilder {
    /// Cria um builder inicializado com o comando ESC/POS `ESC @` (reset da impressora).
    pub fn new() -> Self {
        let mut s = Self {
            buffer: Vec::new(),
            paper_width: 80,
        };
        s.buffer.extend_from_slice(commands::INIT);
        s
    }

    /// Define a largura do papel em milímetros. Use `80` ou `58`. Padrão: `80`.
    /// Afeta a largura de [`divider`](Self::divider).
    pub fn paper_width(mut self, mm: u8) -> Self {
        self.paper_width = mm;
        self
    }

    /// Alinha o texto à esquerda (`ESC a 0`).
    pub fn align_left(mut self) -> Self {
        self.buffer.extend_from_slice(commands::ALIGN_LEFT);
        self
    }

    /// Centraliza o texto (`ESC a 1`).
    pub fn align_center(mut self) -> Self {
        self.buffer.extend_from_slice(commands::ALIGN_CENTER);
        self
    }

    /// Alinha o texto à direita (`ESC a 2`).
    pub fn align_right(mut self) -> Self {
        self.buffer.extend_from_slice(commands::ALIGN_RIGHT);
        self
    }

    /// Ativa (`true`) ou desativa (`false`) o negrito (`ESC E`).
    pub fn bold(mut self, on: bool) -> Self {
        let cmd = if on { commands::BOLD_ON } else { commands::BOLD_OFF };
        self.buffer.extend_from_slice(cmd);
        self
    }

    /// Ativa (`true`) ou desativa (`false`) o sublinhado (`ESC -`).
    pub fn underline(mut self, on: bool) -> Self {
        let cmd = if on { commands::UNDERLINE_ON } else { commands::UNDERLINE_OFF };
        self.buffer.extend_from_slice(cmd);
        self
    }

    /// Tamanho da fonte: 1 = normal, 2 = duplo, 3 = triplo (width × height combinados via GS !)
    pub fn font_size(mut self, size: u8) -> Self {
        let n = size.saturating_sub(1).min(7);
        let byte = (n << 4) | n; // mesma escala em largura e altura
        self.buffer.extend_from_slice(&[0x1D, 0x21, byte]);
        self
    }

    /// Insere texto na posição atual. Use `\n` para quebra de linha.
    pub fn text(mut self, s: impl AsRef<str>) -> Self {
        self.buffer.extend_from_slice(s.as_ref().as_bytes());
        self
    }

    /// Imprime uma linha separadora (`---…`) proporcional à largura do papel.
    /// 80 mm → 48 traços · 58 mm → 32 traços.
    pub fn divider(mut self) -> Self {
        let cols: usize = if self.paper_width >= 80 { 48 } else { 32 };
        let mut line = "-".repeat(cols);
        line.push('\n');
        self.buffer.extend_from_slice(line.as_bytes());
        self
    }

    /// Avança n linhas (ESC d n)
    pub fn feed(mut self, lines: u8) -> Self {
        self.buffer.extend_from_slice(&[0x1B, 0x64, lines]);
        self
    }

    /// Código de barras Code 128 nativo ESC/POS (GS k 73)
    pub fn barcode_128(mut self, data: &str) -> Self {
        let bytes = data.as_bytes();
        self.buffer.extend_from_slice(&[0x1D, 0x6B, 0x49]);
        self.buffer.push(bytes.len() as u8);
        self.buffer.extend_from_slice(bytes);
        self
    }

    /// QR Code nativo ESC/POS via sequência GS ( k
    pub fn qr_code(mut self, data: &str, size: u8) -> Self {
        let model: u8 = 2; // model 2 (padrão QR)
        let size = size.clamp(1, 16);
        let data_bytes = data.as_bytes();
        let data_len = data_bytes.len() as u16 + 3;
        let pl = (data_len & 0xFF) as u8;
        let ph = ((data_len >> 8) & 0xFF) as u8;

        // Selecionar modelo
        self.buffer.extend_from_slice(&[0x1D, 0x28, 0x6B, 0x04, 0x00, 0x31, 0x41, model, 0x00]);
        // Definir tamanho do módulo
        self.buffer.extend_from_slice(&[0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x43, size]);
        // Nível de correção de erros (M = 0x31)
        self.buffer.extend_from_slice(&[0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x45, 0x31]);
        // Armazenar dados
        self.buffer.extend_from_slice(&[0x1D, 0x28, 0x6B, pl, ph, 0x31, 0x50, 0x30]);
        self.buffer.extend_from_slice(data_bytes);
        // Imprimir símbolo armazenado
        self.buffer.extend_from_slice(&[0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x51, 0x30]);
        self
    }

    /// Imagem rasterizada (PNG ou JPEG) via GS v 0.
    /// Converte para bitmap 1-bit com limiarização em 128.
    /// Retorna self inalterado se a imagem não puder ser decodificada.
    pub fn image(mut self, img_bytes: &[u8]) -> Self {
        let img = match ImageReader::new(Cursor::new(img_bytes))
            .with_guessed_format()
            .ok()
            .and_then(|r| r.decode().ok())
        {
            Some(i) => i,
            None => return self,
        };

        let raster = rasterize(&img, self.paper_width);
        self.buffer.extend_from_slice(&raster);
        self
    }

    /// Corte total do papel (`GS V 0`).
    pub fn cut(mut self) -> Self {
        self.buffer.extend_from_slice(commands::CUT_FULL);
        self
    }

    /// Corte parcial do papel (`GS V 1`).
    pub fn partial_cut(mut self) -> Self {
        self.buffer.extend_from_slice(commands::CUT_PARTIAL);
        self
    }

    /// Constrói e retorna os bytes ESC/POS prontos para envio à impressora.
    ///
    /// # Exemplo
    ///
    /// ```
    /// use dfe::EscPosBuilder;
    ///
    /// let bytes = EscPosBuilder::new().text("Olá!\n").cut().build();
    /// assert!(!bytes.is_empty());
    /// ```
    pub fn build(self) -> Vec<u8> {
        self.buffer
    }
}

impl Default for EscPosBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Converte uma imagem para o formato GS v 0 (bitmap 1-bit, MSB primeiro).
/// Redimensiona para caber na largura do papel se necessário.
fn rasterize(img: &DynamicImage, paper_width_mm: u8) -> Vec<u8> {
    let max_dots: u32 = if paper_width_mm >= 80 { 576 } else { 384 };
    let (orig_w, orig_h) = img.dimensions();

    let (w, h) = if orig_w > max_dots {
        let scale = max_dots as f32 / orig_w as f32;
        (max_dots, (orig_h as f32 * scale) as u32)
    } else {
        (orig_w, orig_h)
    };

    let img = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
    let gray = img.to_luma8();

    // Bytes por linha (arredondado para cima para múltiplo de 8, dividido por 8)
    let bytes_per_row = ((w + 7) / 8) as u16;
    let xl = (bytes_per_row & 0xFF) as u8;
    let xh = ((bytes_per_row >> 8) & 0xFF) as u8;
    let yl = (h & 0xFF) as u8;
    let yh = ((h >> 8) & 0xFF) as u8;

    // GS v 0 header: 0x1D 0x76 0x30 m xL xH yL yH
    let mut out = vec![0x1D, 0x76, 0x30, 0x00, xl, xh, yl, yh];

    for row in gray.rows() {
        let pixels: Vec<u8> = row.map(|p| p.0[0]).collect();
        for chunk in pixels.chunks(8) {
            let mut byte = 0u8;
            for (i, &luma) in chunk.iter().enumerate() {
                if luma < 128 {
                    byte |= 0x80 >> i; // pixel escuro = bit 1
                }
            }
            out.push(byte);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_returns_nonempty_bytes() {
        let bytes = EscPosBuilder::new()
            .paper_width(80)
            .align_center()
            .bold(true)
            .text("EMPRESA LTDA\n")
            .bold(false)
            .align_left()
            .text("CNPJ: 11.222.333/0001-81\n")
            .divider()
            .text(format!("{:<20} {:>10}\n", "PRODUTO EXEMPLO", "R$  50,00"))
            .divider()
            .align_right()
            .bold(true)
            .text("TOTAL  R$  50,00\n")
            .bold(false)
            .cut()
            .build();

        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], &[0x1B, 0x40]); // INIT
        assert_eq!(&bytes[bytes.len() - 3..], &[0x1D, 0x56, 0x00]); // CUT_FULL
    }

    #[test]
    fn divider_58mm_is_shorter() {
        let b80 = EscPosBuilder::new().paper_width(80).divider().build();
        let b58 = EscPosBuilder::new().paper_width(58).divider().build();
        assert!(b80.len() > b58.len());
    }

    #[test]
    fn qr_code_produces_bytes() {
        let bytes = EscPosBuilder::new().qr_code("https://example.com", 4).build();
        assert!(bytes.len() > 2);
    }

    #[test]
    fn barcode_128_encodes_data() {
        let data = "12345678";
        let bytes = EscPosBuilder::new().barcode_128(data).build();
        // GS k 0x49 + len + data
        let pos = bytes.windows(3).position(|w| w == [0x1D, 0x6B, 0x49]).unwrap();
        assert_eq!(bytes[pos + 3], data.len() as u8);
    }
}
