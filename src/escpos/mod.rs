mod commands;
pub mod nfce;
pub use nfce::EscPosNFCeBuilder;

use image::{io::Reader as ImageReader, DynamicImage, GenericImageView, GrayImage, Luma};
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
    /// Largura imprimível real em dots nativos da impressora.
    /// 576 = padrão 80 mm / 203 DPI. Use [`printable_dots`](Self::printable_dots) para ajustar.
    paper_dots: u32,
    cols: Option<usize>,
}

impl EscPosBuilder {
    /// Cria um builder inicializado com o comando ESC/POS `ESC @` (reset da impressora).
    pub fn new() -> Self {
        let mut s = Self {
            buffer: Vec::new(),
            paper_width: 80,
            paper_dots: 576,
            cols: None,
        };
        s.buffer.extend_from_slice(commands::INIT);
        s.buffer.extend_from_slice(&[0x1B, 0x74, 0x02]); // ESC t 2 = CP850
        s
    }

    /// Define a largura do papel em milímetros. Use `80` ou `58`. Padrão: `80`.
    /// Afeta a largura de [`divider`](Self::divider) e o padrão de [`paper_dots`](Self::printable_dots).
    pub fn paper_width(mut self, mm: u8) -> Self {
        self.paper_width = mm;
        self.paper_dots = if mm >= 80 { 576 } else { 384 };
        self
    }

    /// Define a resolução nativa da impressora em DPI.
    ///
    /// Calcula automaticamente `printable_dots` = largura imprimível × DPI / 25,4.
    /// Use `203` para impressoras padrão ESC/POS · `300` para alta resolução.
    /// Deve ser chamado **após** [`paper_width`](Self::paper_width).
    pub fn printer_dpi(mut self, dpi: u32) -> Self {
        let printable_mm: f32 = if self.paper_width >= 80 { 72.0 } else { 48.0 };
        self.paper_dots = (printable_mm * dpi as f32 / 25.4).round() as u32;
        self
    }

    /// Define a largura imprimível real da impressora em dots nativos.
    ///
    /// Alternativa manual a [`printer_dpi`](Self::printer_dpi).
    /// Padrão: `576` (80 mm · 203 DPI). Para 300 DPI / 80 mm: `850`.
    pub fn printable_dots(mut self, dots: u32) -> Self {
        self.paper_dots = dots;
        self
    }

    /// Sobrescreve o número de colunas de texto usado por [`divider`](Self::divider).
    ///
    /// Por padrão: 48 para papel 80 mm · 32 para 58 mm.
    /// Use quando a impressora tem área imprimível diferente do padrão ESC/POS
    /// (ex.: Bematech MP-4200 TH → 42 colunas).
    pub fn columns(mut self, cols: usize) -> Self {
        self.cols = Some(cols);
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

    /// Escala somente a **altura** do caractere, mantendo a largura em 1×.
    /// Útil para texto em destaque sem reduzir o número de colunas por linha.
    /// 1 = normal · 2 = altura dupla · 3 = altura tripla (máx 8).
    pub fn font_height(mut self, size: u8) -> Self {
        let h = size.saturating_sub(1).min(7);
        let byte = h << 4; // nibble superior = altura, nibble inferior = 0 (1× largura)
        self.buffer.extend_from_slice(&[0x1D, 0x21, byte]);
        self
    }

    /// Define o espaçamento entre linhas em pontos gráficos (`ESC 3 n`).
    /// Valor padrão típico da impressora é ~30 pontos.
    /// Use valores pequenos (2–8) ao redor de separadores para reduzir o espaço vertical.
    pub fn line_spacing(mut self, dots: u8) -> Self {
        self.buffer.extend_from_slice(&[0x1B, 0x33, dots]);
        self
    }

    /// Restaura o espaçamento entre linhas ao padrão da impressora (`ESC 2`).
    pub fn line_spacing_default(mut self) -> Self {
        self.buffer.extend_from_slice(&[0x1B, 0x32]);
        self
    }

    /// Insere texto na posição atual. Use `\n` para quebra de linha.
    /// O texto é convertido automaticamente para CP850, permitindo caracteres PT-BR.
    pub fn text(mut self, s: impl AsRef<str>) -> Self {
        self.buffer.extend_from_slice(&encode_cp850(s.as_ref()));
        self
    }

    /// Imprime uma linha separadora (`---…`) proporcional à largura do papel.
    /// 80 mm → 48 traços · 58 mm → 32 traços.
    /// Pode ser sobrescrito com [`columns`](Self::columns).
    pub fn divider(mut self) -> Self {
        let cols = self.cols.unwrap_or_else(|| if self.paper_width >= 80 { 48 } else { 32 });
        let mut line = "-".repeat(cols);
        line.push('\n');
        self.buffer.extend_from_slice(line.as_bytes());
        self
    }

    /// Seleciona fonte B (`ESC M 1`), menor e mais condensada que a fonte A padrão.
    /// Restaure com `font_b(false)` (`ESC M 0`).
    pub fn font_b(mut self, on: bool) -> Self {
        self.buffer.extend_from_slice(&[0x1B, 0x4D, if on { 1 } else { 0 }]);
        self
    }

    /// Avança n linhas (ESC d n)
    pub fn feed(mut self, lines: u8) -> Self {
        self.buffer.extend_from_slice(&[0x1B, 0x64, lines]);
        self
    }

    /// Código de barras Code 128 renderizado como **imagem raster** (`GS v 0`).
    ///
    /// Usa `barcoders` para calcular os módulos, constrói um `GrayImage` via crate `image`
    /// e envia pelo mesmo pipeline de `rasterize()` usado por [`image`](Self::image).
    ///
    /// - Dados só-dígitos de comprimento par → Code 128C (2 dígitos/símbolo, máxima densidade)
    /// - Outros dados → Code 128B (ASCII imprimível)
    /// - Altura: 80 px · Largura de módulo: 2 px
    pub fn barcode_128(mut self, data: &str) -> Self {
        use barcoders::sym::code128::Code128;

        let is_numeric_even = data.chars().all(|c| c.is_ascii_digit()) && data.len() % 2 == 0;
        // \u{0106} = Ć = Start-C (pares de dígitos) · \u{0105} = ą = Start-B (ASCII)
        let code_data = if is_numeric_even {
            format!("\u{0106}{data}")
        } else {
            format!("\u{0105}{data}")
        };

        let encoded = match Code128::new(&code_data) {
            Ok(b) => b.encode(),
            Err(_) => return self,
        };

        let module_width: u32 = 2;
        let bar_height: u32 = 80;
        let total_width: u32 = encoded.len() as u32 * module_width;

        let mut img = GrayImage::new(total_width, bar_height);
        for (idx, &bar) in encoded.iter().enumerate() {
            let luma = if bar == 1 { 0u8 } else { 255u8 };
            for dx in 0..module_width {
                let x = idx as u32 * module_width + dx;
                for y in 0..bar_height {
                    img.put_pixel(x, y, Luma([luma]));
                }
            }
        }

        let raster = rasterize(&DynamicImage::ImageLuma8(img), self.paper_width);
        self.buffer.extend_from_slice(&raster);
        self.buffer.push(b'\n');
        self
    }

    /// QR Code nativo ESC/POS via sequência GS ( k
    pub fn qr_code(mut self, data: &str, size: u8) -> Self {
        let model: u8 = 50; // ESC/POS: 49=Model1, 50=Model2 (padrão), 51=MicroQR
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

    /// Abre a gaveta de dinheiro (`ESC p`).
    ///
    /// `pin` seleciona o pino de acionamento: `2` (pino 2, padrão da maioria das gavetas)
    /// ou `5` (pino 5). Qualquer outro valor usa o pino 2.
    pub fn open_drawer(mut self, pin: u8) -> Self {
        let cmd = if pin == 5 { commands::CASH_DRAWER_PIN5 } else { commands::CASH_DRAWER_PIN2 };
        self.buffer.extend_from_slice(cmd);
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

    /// QR Code à esquerda e texto à direita como **imagem raster `GS v 0`**.
    ///
    /// Abordagem 100 % compatível com qualquer impressora ESC/POS.
    /// O QR ocupa ~55 % de [`printable_dots`](Self::printable_dots) e é sempre quadrado.
    /// O texto usa font8x8 em escala **2×** (cada pixel = bloco 2×2, ~2 mm por caractere).
    /// Cada entrada é `(texto, negrito)` — linhas negrito são desenhadas duas vezes
    /// com 1 px de deslocamento horizontal para simular espessura extra.
    pub fn qr_with_text_right(mut self, qr_data: &str, lines: &[(String, bool)]) -> Self {
        use font8x8::UnicodeFonts;
        use qrcodegen::{QrCode, QrCodeEcc};

        let qr = match QrCode::encode_text(qr_data, QrCodeEcc::Medium) {
            Ok(q) => q,
            Err(_) => return self,
        };

        let qr_modules = qr.size() as u32;
        let quiet = 4u32;
        let total_mod = qr_modules + quiet * 2;

        let paper_dots = self.paper_dots;
        let max_qr_w = paper_dots * 55 / 100;
        let scale = (max_qr_w / total_mod).max(2);
        let qr_px = total_mod * scale; // quadrado: qr_px × qr_px

        const S: u32 = 2;              // font scale: cada pixel vira bloco S×S
        const FONT_W: u32 = 8 * S;    // 16 px por caractere
        const FONT_H: u32 = 8 * S;    // 16 px por caractere
        const LINE_H: u32 = FONT_H + 4; // 20 px por linha

        const GAP: u32 = 8;
        let text_x = qr_px + GAP;
        let img_h = qr_px.max(lines.len() as u32 * LINE_H).max(1);
        let mut img = GrayImage::from_pixel(paper_dots, img_h, Luma([255u8]));

        // ── QR Code (sempre quadrado) ─────────────────────────────────────────
        for my in 0..qr_modules {
            for mx in 0..qr_modules {
                if qr.get_module(mx as i32, my as i32) {
                    let px0 = (quiet + mx) * scale;
                    let py0 = (quiet + my) * scale;
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let x = px0 + dx;
                            let y = py0 + dy;
                            if x < paper_dots && y < img_h {
                                img.put_pixel(x, y, Luma([0u8]));
                            }
                        }
                    }
                }
            }
        }

        // ── Texto à direita (font8x8, escala 2×) ─────────────────────────────
        let draw_glyph = |img: &mut GrayImage, cx: u32, y0: u32, glyph: [u8; 8], offset_x: u32| {
            for (row, &byte) in glyph.iter().enumerate() {
                for sy in 0..S {
                    let y = y0 + row as u32 * S + sy;
                    if y >= img_h { break; }
                    for bit in 0..8u32 {
                        if byte & (1u8 << bit) != 0 {
                            for sx in 0..S {
                                let x = cx + offset_x + bit * S + sx;
                                if x < paper_dots {
                                    img.put_pixel(x, y, Luma([0u8]));
                                }
                            }
                        }
                    }
                }
            }
        };

        for (li, (line, bold)) in lines.iter().enumerate() {
            let y0 = li as u32 * LINE_H;
            let mut cx = text_x;
            for ch in line.chars() {
                if cx + FONT_W > paper_dots { break; }
                let glyph = font8x8::BASIC_FONTS
                    .get(ch)
                    .or_else(|| font8x8::LATIN_FONTS.get(ch))
                    .unwrap_or([0u8; 8]);
                draw_glyph(&mut img, cx, y0, glyph, 0);
                if *bold {
                    draw_glyph(&mut img, cx, y0, glyph, 1);
                }
                cx += FONT_W;
            }
        }

        // rasterize() via DynamicImage garante orientação correta (caminho comprovado)
        let raster = rasterize(&DynamicImage::ImageLuma8(img), self.paper_width);
        self.buffer.extend_from_slice(&raster);

        self
    }
}

impl Default for EscPosBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Converte uma string UTF-8 para bytes CP850 (code page da impressora).
/// Caracteres ASCII passam direto; os demais são mapeados para o equivalente CP850.
fn encode_cp850(s: &str) -> Vec<u8> {
    s.chars().map(|c| {
        if c.is_ascii() { return c as u8; }
        match c {
            'Ç' => 0x80, 'ü' => 0x81, 'é' => 0x82, 'â' => 0x83,
            'ä' => 0x84, 'à' => 0x85, 'å' => 0x86, 'ç' => 0x87,
            'ê' => 0x88, 'ë' => 0x89, 'è' => 0x8A, 'ï' => 0x8B,
            'î' => 0x8C, 'ì' => 0x8D, 'Ä' => 0x8E, 'Å' => 0x8F,
            'É' => 0x90, 'æ' => 0x91, 'Æ' => 0x92, 'ô' => 0x93,
            'ö' => 0x94, 'ò' => 0x95, 'û' => 0x96, 'ù' => 0x97,
            'ÿ' => 0x98, 'Ö' => 0x99, 'Ü' => 0x9A, 'ø' => 0x9B,
            '£' => 0x9C, 'Ø' => 0x9D, 'á' => 0xA0, 'í' => 0xA1,
            'ó' => 0xA2, 'ú' => 0xA3, 'ñ' => 0xA4, 'Ñ' => 0xA5,
            'ª' => 0xA6, 'º' => 0xA7, '¿' => 0xA8, '®' => 0xA9,
            '½' => 0xAB, '¼' => 0xAC, '«' => 0xAE, '»' => 0xAF,
            'Á' => 0xB5, 'Â' => 0xB6, 'À' => 0xB7, '©' => 0xB8,
            '¢' => 0xBD, 'ã' => 0xC6, 'Ã' => 0xC7, 'ð' => 0xD0,
            'Ð' => 0xD1, 'Ê' => 0xD2, 'Ë' => 0xD3, 'È' => 0xD4,
            'Í' => 0xD6, 'Î' => 0xD7, 'Ï' => 0xD8, 'Ì' => 0xDE,
            'Ó' => 0xE0, 'ß' => 0xE1, 'Ô' => 0xE2, 'Ò' => 0xE3,
            'õ' => 0xE4, 'Õ' => 0xE5, 'µ' => 0xE6, 'Ú' => 0xE9,
            'Û' => 0xEA, 'Ù' => 0xEB, 'ý' => 0xEC, 'Ý' => 0xED,
            '°' => 0xF8, '±' => 0xF1, '¶' => 0xF4, '§' => 0xF5,
            '÷' => 0xF6, '¸' => 0xF7, '¨' => 0xF9, '²' => 0xFD,
            '³' => 0xFC, '¹' => 0xFB, '·' => 0xFA,
            _ => b'?',
        }
    }).collect()
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
        // GS v 0 raster header
        let pos = bytes.windows(4).position(|w| w == [0x1D, 0x76, 0x30, 0x00]).unwrap();
        assert!(bytes.len() > pos + 8);
    }
}
