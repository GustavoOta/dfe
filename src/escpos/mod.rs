mod commands;
pub mod nfce;
pub mod nfe;
pub use nfce::EscPosNFCeBuilder;
pub use nfe::EscPosDanfeNFeBuilder;

use image::{io::Reader as ImageReader, DynamicImage, GenericImageView, GrayImage, Luma};
use std::io::Cursor;

// ── Layout "QR à esquerda, texto à direita" ───────────────────────────────────
// A fonte bitmap 8×8 é desenhada em escala 2×, então cada caractere ocupa 16 dots.

/// Lado do glifo da fonte bitmap, em pixels, antes de escalar.
const QR_SIDE_GLYPH: u32 = 8;
/// Escala nominal do texto lateral: cada pixel do glifo vira um bloco 2×2.
const QR_SIDE_SCALE: u32 = 2;
/// Escala reduzida, usada só para o campo que não cabe inteiro na nominal.
const QR_SIDE_SCALE_MIN: u32 = 1;
/// Largura de um caractere do texto lateral na escala nominal, em dots.
const QR_SIDE_FONT_W: u32 = QR_SIDE_GLYPH * QR_SIDE_SCALE;
/// Altura de um caractere do texto lateral na escala nominal, em dots.
const QR_SIDE_FONT_H: u32 = QR_SIDE_GLYPH * QR_SIDE_SCALE;
/// Altura de uma linha do texto lateral: a fonte mais 4 dots de respiro.
const QR_SIDE_LINE_H: u32 = QR_SIDE_FONT_H + 4;
/// Respiro horizontal entre o QR Code e o texto, em dots.
const QR_SIDE_GAP: u32 = 8;
/// Zona de silêncio do QR Code, em módulos, de cada lado.
const QR_SIDE_QUIET: u32 = 4;

/// Medidas do layout lateral, todas derivadas do QR Code real.
#[derive(Clone, Copy, Debug)]
struct QrSideGeom {
    /// Fator de ampliação de cada módulo do QR, em dots.
    scale: u32,
    /// Lado do quadrado do QR já ampliado, em dots.
    qr_px: u32,
    /// Coluna em que o texto lateral começa, em dots.
    text_x: u32,
    /// Linha em que o texto lateral começa, em dots.
    ///
    /// O QR só passa a desenhar depois da zona de silêncio, então o topo do papel é
    /// branco por `quiet × scale` dots. O texto arrancava em `y = 0` e encostava no
    /// que vem impresso acima (o código de barras da chave); alinhado ao topo visível
    /// do QR, ganha a mesma respiração que o código tem.
    text_top: u32,
    /// Caracteres que cabem **ao lado** do QR, na escala nominal.
    text_cols: usize,
    /// Linhas de texto que cabem **inteiras** ao lado do QR, já a partir de `text_top`.
    ///
    /// Conta a linha cujo glifo termina antes da base do código: linha que só caberia
    /// pela metade invadiria o canto inferior do QR quando passasse a usar a largura
    /// cheia do papel.
    side_slots: u32,
}

/// Empilha um campo do layout lateral do QR em linhas já posicionadas e dimensionadas.
///
/// `abaixo_do_qr` força a faixa de largura cheia; sem ele, o campo fica ao lado do QR
/// enquanto houver linha livre lá. Em ambos os casos, a primeira linha de largura cheia
/// é empurrada para depois da base do código — senão entraria por cima do canto dele.
fn empilha_linha_lateral(
    placed: &mut Vec<(String, bool, u32, u32)>,
    geom: &QrSideGeom,
    paper_dots: u32,
    text: &str,
    bold: bool,
    abaixo_do_qr: bool,
) {
    let x = if !abaixo_do_qr && (placed.len() as u32) < geom.side_slots {
        geom.text_x
    } else {
        while geom.text_top + placed.len() as u32 * QR_SIDE_LINE_H < geom.qr_px {
            placed.push((String::new(), false, 0, QR_SIDE_SCALE));
        }
        0
    };

    let largura = paper_dots.saturating_sub(x);
    let cols = (largura / QR_SIDE_FONT_W).max(1) as usize;

    // Só encolhe quando a alternativa é partir uma palavra ao meio. Quebra entre palavras
    // não mutila nada, então texto corrido segue no corpo nominal.
    if text.split_whitespace().any(|w| w.chars().count() > cols) {
        let cols_min = (largura / (QR_SIDE_GLYPH * QR_SIDE_SCALE_MIN)).max(1) as usize;
        if text.chars().count() <= cols_min {
            placed.push((text.to_string(), bold, x, QR_SIDE_SCALE_MIN));
            return;
        }
    }

    for line in wrap_hard(text, cols) {
        placed.push((line, bold, x, QR_SIDE_SCALE));
    }
}

/// Quebra `text` em linhas de no máximo `max_chars` caracteres.
///
/// Quebra por espaço quando dá e **parte a palavra** que sozinha não cabe. Sem o corte
/// duro, uma razão social longa sem espaços atravessaria a coluna e seria descartada em
/// silêncio pelo rasterizador, que simplesmente para de desenhar no fim do papel.
///
/// Texto vazio (ou só espaços) rende uma linha vazia, para que o chamador possa usar uma
/// entrada em branco como espaçador.
pub(crate) fn wrap_hard(text: &str, max_chars: usize) -> Vec<String> {
    let max = max_chars.max(1);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        // Cabe no resto da linha corrente.
        if current_len != 0 && current_len + 1 + word_len <= max {
            current.push(' ');
            current.push_str(word);
            current_len += 1 + word_len;
            continue;
        }

        if current_len != 0 {
            lines.push(std::mem::take(&mut current));
            current_len = 0;
        }

        // Cabe sozinha em uma linha: começa a próxima com ela.
        if word_len <= max {
            current.push_str(word);
            current_len = word_len;
            continue;
        }

        // Maior que a coluna: parte em pedaços do tamanho da coluna.
        for ch in word.chars() {
            current.push(ch);
            current_len += 1;
            if current_len == max {
                lines.push(std::mem::take(&mut current));
                current_len = 0;
            }
        }
    }

    if current_len != 0 {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

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
        // `GS ! n`: nibble **inferior** = altura, nibble **superior** = largura.
        // Inverter os dois não dá erro nem sai borrado: dobra a largura, a linha
        // passa a ocupar o dobro de colunas e o fim dela cai no papel de baixo.
        let byte = size.saturating_sub(1).min(7);
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
        let encoded = match encode_code128(data) {
            Some(e) => e,
            None => return self,
        };

        let img = barcode_image(&encoded, 2, 80);
        let raster = rasterize(&DynamicImage::ImageLuma8(img), self.paper_width);
        self.buffer.extend_from_slice(&raster);
        self.buffer.push(b'\n');
        self
    }

    /// Código de barras **EAN-13** renderizado como imagem raster, no mesmo
    /// pipeline de [`barcode_128`](Self::barcode_128).
    ///
    /// É o padrão dos produtos de varejo no Brasil, e um leitor de PDV lê EAN-13
    /// bem mais rápido do que o mesmo número em Code 128.
    ///
    /// Aceita 12 dígitos (calcula o dígito verificador) ou 13 (confere o que veio).
    /// Quando `data` não é um EAN-13 válido — código interno, campo em branco,
    /// dígito verificador errado — **cai automaticamente em Code 128**, que aceita
    /// qualquer conteúdo: melhor uma etiqueta legível em outro padrão do que uma
    /// etiqueta sem código de barras nenhum.
    pub fn barcode_ean13(mut self, data: &str) -> Self {
        let encoded = match encode_ean13(data) {
            Some(e) => e,
            None => return self.barcode_128(data),
        };

        let img = barcode_image(&encoded, 2, 80);
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

    /// Geometria do layout "QR à esquerda, texto à direita" para um QR de
    /// `qr_modules` módulos de lado.
    ///
    /// `scale` é arredondado para baixo, então `qr_px` — e com ele a largura que
    /// sobra para o texto — muda conforme a versão do QR. Por isso a coluna lateral
    /// nunca é estimada por percentual fixo do papel: é derivada do QR real.
    fn qr_side_geometry(&self, qr_modules: u32) -> QrSideGeom {
        let total_mod = qr_modules + QR_SIDE_QUIET * 2;
        let max_qr_w = self.paper_dots * 55 / 100;
        let scale = (max_qr_w / total_mod).max(2);
        let qr_px = total_mod * scale; // quadrado: qr_px × qr_px
        let text_x = qr_px + QR_SIDE_GAP;
        let text_top = QR_SIDE_QUIET * scale;
        QrSideGeom {
            scale,
            qr_px,
            text_x,
            text_top,
            text_cols: (self.paper_dots.saturating_sub(text_x) / QR_SIDE_FONT_W).max(1) as usize,
            side_slots: match qr_px.checked_sub(text_top + QR_SIDE_FONT_H) {
                Some(util) => util / QR_SIDE_LINE_H + 1,
                None => 0,
            },
        }
    }

    /// Quantos caracteres cabem **ao lado** do QR Code em
    /// [`qr_with_text_right`](Self::qr_with_text_right), para este payload e este papel.
    ///
    /// A largura não é constante: depende da versão do QR — que cresce com a UF, com o
    /// tamanho do `idCSC` e com a contingência — e do `scale` arredondado para baixo.
    /// Medida com URLs de QR Code de NFC-e reais, ela varia entre 16 e 19 em 80 mm
    /// (576 dots) e entre 10 e 12 em 58 mm (384 dots).
    ///
    /// Serve para o chamador **escolher entre formas curta e longa** de um campo (CNPJ
    /// com ou sem máscara, por exemplo). A quebra de linha em si já é feita dentro do
    /// `qr_with_text_right` — não é preciso pré-quebrar nada.
    ///
    /// Retorna `0` quando o payload não cabe em um QR Code.
    pub fn qr_text_cols(&self, qr_data: &str) -> usize {
        match qrcodegen::QrCode::encode_text(qr_data, qrcodegen::QrCodeEcc::Medium) {
            Ok(qr) => self.qr_side_geometry(qr.size() as u32).text_cols,
            Err(_) => 0,
        }
    }

    /// QR Code à esquerda e texto à direita como **imagem raster `GS v 0`**.
    ///
    /// Abordagem 100 % compatível com qualquer impressora ESC/POS.
    /// O QR ocupa ~55 % de [`printable_dots`](Self::printable_dots) e é sempre quadrado.
    /// O texto usa font8x8 em escala **2×** (cada pixel = bloco 2×2, ~2 mm por caractere).
    /// Cada entrada é `(texto, negrito)` — linhas negrito são desenhadas duas vezes
    /// com 1 px de deslocamento horizontal para simular espessura extra.
    ///
    /// **A quebra de linha acontece aqui**, contra a largura real derivada do QR
    /// ([`qr_text_cols`](Self::qr_text_cols)): o chamador passa cada campo inteiro, sem
    /// pré-quebrar. Palavra maior que a coluna é partida em vez de estourar, então nada
    /// é cortado em silêncio. Entrada que já não cabe ao lado do QR é desenhada **abaixo
    /// dele, com a largura inteira do papel**, em vez de continuar espremida na coluna
    /// estreita — a decisão é por entrada, na linha em que ela começa, para um mesmo
    /// campo não trocar de recuo no meio.
    ///
    /// O bloco de texto começa na mesma altura em que o QR começa a desenhar, isto é,
    /// depois da zona de silêncio: assim ele tem a mesma margem superior que o código,
    /// em vez de encostar no que foi impresso na linha anterior.
    ///
    /// **Campo que só caberia partindo uma palavra sai em corpo menor**, inteiro, em vez
    /// de quebrado. É o caso do CNPJ com máscara (18 caracteres contra as 16 colunas do
    /// pior caso em 80 mm) e do número do protocolo em 58 mm: partir `24.341.498/0001-14`
    /// ao meio, ou largar a máscara, custa mais legibilidade do que meio corpo de fonte.
    /// Texto que quebra bem entre palavras — a razão social, por exemplo — continua no
    /// corpo nominal, porque ali a quebra não mutila nada. A altura da linha não muda,
    /// então o corpo menor apenas ganha respiro dentro da mesma pauta.
    pub fn qr_with_text_right(self, qr_data: &str, lines: &[(String, bool)]) -> Self {
        self.qr_with_text_right_and_below(qr_data, lines, &[])
    }

    /// Como [`qr_with_text_right`](Self::qr_with_text_right), mas com uma segunda faixa
    /// de campos **abaixo** do QR Code, na largura inteira do papel.
    ///
    /// Serve ao campo largo demais para a coluna estreita ao lado do código — a razão
    /// social e o documento do destinatário, por exemplo: ao lado do QR eles teriam de
    /// quebrar em quatro linhas ou encolher de corpo; embaixo dele cabem inteiros, no
    /// corpo nominal.
    ///
    /// `below` vazio não gasta uma linha sequer: a faixa inferior só existe quando tem
    /// conteúdo, e o cupom continua terminando na base do QR.
    pub fn qr_with_text_right_and_below(
        mut self,
        qr_data: &str,
        right: &[(String, bool)],
        below: &[(String, bool)],
    ) -> Self {
        use font8x8::UnicodeFonts;
        use qrcodegen::{QrCode, QrCodeEcc};

        let qr = match QrCode::encode_text(qr_data, QrCodeEcc::Medium) {
            Ok(q) => q,
            Err(_) => return self,
        };

        let qr_modules = qr.size() as u32;
        let quiet = QR_SIDE_QUIET;

        let paper_dots = self.paper_dots;
        let geom = self.qr_side_geometry(qr_modules);
        let scale = geom.scale;
        let qr_px = geom.qr_px;

        const LINE_H: u32 = QR_SIDE_LINE_H; // 20 px por linha, em qualquer escala

        // ── Quebra e posicionamento (largura real, nunca estimada) ────────────
        // (texto, negrito, x, escala)
        let mut placed: Vec<(String, bool, u32, u32)> = Vec::new();
        for (text, bold) in right {
            empilha_linha_lateral(&mut placed, &geom, paper_dots, text, *bold, false);
        }
        for (text, bold) in below {
            empilha_linha_lateral(&mut placed, &geom, paper_dots, text, *bold, true);
        }

        let img_h = qr_px.max(geom.text_top + placed.len() as u32 * LINE_H).max(1);
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

        // ── Texto à direita (font8x8, escala por linha) ──────────────────────
        let draw_glyph =
            |img: &mut GrayImage, cx: u32, y0: u32, glyph: [u8; 8], offset_x: u32, s: u32| {
                for (row, &byte) in glyph.iter().enumerate() {
                    for sy in 0..s {
                        let y = y0 + row as u32 * s + sy;
                        if y >= img_h { break; }
                        for bit in 0..8u32 {
                            if byte & (1u8 << bit) != 0 {
                                for sx in 0..s {
                                    let x = cx + offset_x + bit * s + sx;
                                    if x < paper_dots {
                                        img.put_pixel(x, y, Luma([0u8]));
                                    }
                                }
                            }
                        }
                    }
                }
            };

        for (li, (line, bold, x, scale)) in placed.iter().enumerate() {
            let glifo_w = QR_SIDE_GLYPH * scale;
            // Corpo menor fica centrado na mesma pauta, não colado no topo da linha.
            let y0 = geom.text_top
                + li as u32 * LINE_H
                + (QR_SIDE_FONT_H - QR_SIDE_GLYPH * scale) / 2;
            let mut cx = *x;
            for ch in line.chars() {
                if cx + glifo_w > paper_dots { break; }
                let glyph = font8x8::BASIC_FONTS
                    .get(ch)
                    .or_else(|| font8x8::LATIN_FONTS.get(ch))
                    .unwrap_or([0u8; 8]);
                draw_glyph(&mut img, cx, y0, glyph, 0, *scale);
                if *bold {
                    draw_glyph(&mut img, cx, y0, glyph, 1, *scale);
                }
                cx += glifo_w;
            }
        }

        // rasterize() via DynamicImage garante orientação correta (caminho comprovado)
        let raster = rasterize(&DynamicImage::ImageLuma8(img), self.paper_width);
        self.buffer.extend_from_slice(&raster);

        self
    }

    /// Código de barras à **esquerda** e texto à **direita**, na mesma faixa,
    /// como uma única imagem raster (`GS v 0`).
    ///
    /// Em ESC/POS, tanto `GS k` quanto `GS v 0` encerram a linha: nada é impresso
    /// ao lado deles por comando de texto. Lado a lado só existe compondo **uma**
    /// imagem — é o mesmo caminho de [`qr_with_text_right`](Self::qr_with_text_right).
    ///
    /// Pensado para etiqueta de gôndola (barras + preço), mas serve a qualquer par.
    ///
    /// - `data` vira EAN-13 e, se não for um EAN válido, Code 128 — como em
    ///   [`barcode_ean13`](Self::barcode_ean13).
    /// - O número legível sai em corpo pequeno sob as barras.
    /// - O texto da direita é ampliado **até o maior corpo que couber** no espaço
    ///   que sobrou, e centralizado na vertical: o mesmo código serve 80 e 58 mm.
    /// - Cada entrada é `(texto, negrito)`.
    /// - Se o código de barras não couber deixando espaço de texto utilizável,
    ///   cai no empilhado (barras em cima, texto embaixo) em vez de espremer.
    pub fn barcode_with_text_right(mut self, data: &str, lines: &[(String, bool)]) -> Self {
        let encoded = match encode_ean13(data).or_else(|| encode_code128(data)) {
            Some(e) => e,
            None => return self,
        };

        let paper_dots = self.paper_dots;
        let modules = encoded.len() as u32;

        // Barras ocupam ~50 % do papel; o módulo nunca fica abaixo de 2 px, que é
        // o mínimo para o leitor não perder a barra fina.
        const GAP: u32 = 12;
        const BAR_H: u32 = 90;
        const LEGENDA_S: u32 = 2;

        let module_width = (paper_dots * 50 / 100 / modules).max(2);
        let bc_w = modules * module_width;
        let texto_w = paper_dots.saturating_sub(bc_w + GAP);

        // Sem largura para pelo menos ~6 caracteres legíveis, lado a lado piora a
        // etiqueta em vez de melhorar: volta ao empilhado.
        if texto_w < 6 * 8 * 2 {
            self = self.barcode_ean13(data);
            for (linha, _) in lines {
                self = self.text(format!("{linha}\n"));
            }
            return self;
        }

        let bloco_barras = BAR_H + 4 + 8 * LEGENDA_S;

        // Maior corpo que cabe: largura da maior linha e altura do bloco inteiro.
        let mais_larga = lines.iter().map(|(l, _)| l.chars().count() as u32).max().unwrap_or(0);
        let escala = (2..=8u32)
            .rev()
            .find(|s| mais_larga * 8 * s <= texto_w && lines.len() as u32 * (8 * s + 4) <= bloco_barras)
            .unwrap_or(2);
        let bloco_texto = lines.len() as u32 * (8 * escala + 4);

        let img_h = bloco_barras.max(bloco_texto).max(1);
        let mut img = GrayImage::from_pixel(paper_dots, img_h, Luma([255u8]));

        // ── Barras + número legível, centralizados na vertical ────────────────
        let bar_y = (img_h - bloco_barras) / 2;
        for (idx, &bar) in encoded.iter().enumerate() {
            if bar != 1 {
                continue;
            }
            for dx in 0..module_width {
                let x = idx as u32 * module_width + dx;
                for y in bar_y..bar_y + BAR_H {
                    img.put_pixel(x, y, Luma([0u8]));
                }
            }
        }
        let legenda = data.trim();
        let legenda_x = bc_w.saturating_sub(text_width(legenda, LEGENDA_S)) / 2;
        draw_text(&mut img, legenda_x, bar_y + BAR_H + 4, legenda, LEGENDA_S, false);

        // ── Texto à direita, centralizado na vertical ─────────────────────────
        let texto_x = bc_w + GAP;
        let mut y = (img_h - bloco_texto.min(img_h)) / 2;
        for (linha, bold) in lines {
            draw_text(&mut img, texto_x, y, linha, escala, *bold);
            y += 8 * escala + 4;
        }

        let raster = rasterize(&DynamicImage::ImageLuma8(img), self.paper_width);
        self.buffer.extend_from_slice(&raster);
        self.buffer.push(b'\n');
        self
    }
}

// ── Código de barras: codificação e desenho ──────────────────────────────────

/// Módulos de um EAN-13 (aceita 12 dígitos e calcula o verificador, ou 13 e o
/// confere). `None` quando `data` não é EAN-13 válido.
fn encode_ean13(data: &str) -> Option<Vec<u8>> {
    barcoders::sym::ean13::EAN13::new(data).ok().map(|b| b.encode())
}

/// Módulos de um Code 128. Dados só-dígitos de comprimento par vão em Code 128C
/// (dobro da densidade); o resto em Code 128B.
fn encode_code128(data: &str) -> Option<Vec<u8>> {
    // Marcadores de conjunto exigidos pelo `barcoders`:
    // \u{0106} = Ć = Start-C (pares de dígitos) · \u{0181} = Ɓ = Start-B (ASCII imprimível).
    // Não confundir o Start-B com \u{0105} (ą): o crate rejeita e o código de
    // barras sai vazio — era o que acontecia com qualquer dado não numérico.
    let numerico_par = data.chars().all(|c| c.is_ascii_digit()) && data.len() % 2 == 0;
    let marcado = if numerico_par {
        format!("\u{0106}{data}")
    } else {
        format!("\u{0181}{data}")
    };
    barcoders::sym::code128::Code128::new(&marcado).ok().map(|b| b.encode())
}

/// Imagem só das barras, sem margem — usada quando o código ocupa a faixa inteira.
fn barcode_image(encoded: &[u8], module_width: u32, height: u32) -> GrayImage {
    let mut img = GrayImage::new(encoded.len() as u32 * module_width, height);
    for (idx, &bar) in encoded.iter().enumerate() {
        let luma = if bar == 1 { 0u8 } else { 255u8 };
        for dx in 0..module_width {
            for y in 0..height {
                img.put_pixel(idx as u32 * module_width + dx, y, Luma([luma]));
            }
        }
    }
    img
}

// ── Texto desenhado em imagem (font8x8) ──────────────────────────────────────

/// Largura em pixels que `texto` ocupa na escala dada.
fn text_width(texto: &str, escala: u32) -> u32 {
    texto.chars().count() as u32 * 8 * escala
}

/// Desenha `texto` na imagem a partir de (`x`, `y`), cada pixel da fonte virando
/// um bloco `escala`×`escala`. Negrito = segunda passada com 1 px de deslocamento.
/// O que passar da borda é descartado.
fn draw_text(img: &mut GrayImage, x: u32, y: u32, texto: &str, escala: u32, bold: bool) {
    use font8x8::UnicodeFonts;

    let (w, h) = (img.width(), img.height());
    let mut cx = x;
    for ch in texto.chars() {
        if cx + 8 * escala > w {
            break;
        }
        let glyph = font8x8::BASIC_FONTS
            .get(ch)
            .or_else(|| font8x8::LATIN_FONTS.get(ch))
            .unwrap_or([0u8; 8]);

        for (row, &byte) in glyph.iter().enumerate() {
            for sy in 0..escala {
                let py = y + row as u32 * escala + sy;
                if py >= h {
                    break;
                }
                for bit in 0..8u32 {
                    if byte & (1u8 << bit) == 0 {
                        continue;
                    }
                    for sx in 0..escala {
                        let px = cx + bit * escala + sx;
                        if px < w {
                            img.put_pixel(px, py, Luma([0u8]));
                        }
                        if bold && px + 1 < w {
                            img.put_pixel(px + 1, py, Luma([0u8]));
                        }
                    }
                }
            }
        }
        cx += 8 * escala;
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

    /// Último `GS ! n` emitido — o `build()` prefixa init e code page.
    fn escala(bytes: &[u8]) -> u8 {
        let i = bytes
            .windows(2)
            .rposition(|w| w == [0x1D, 0x21])
            .expect("nenhum GS ! no buffer");
        bytes[i + 2]
    }

    /// Quantos cabeçalhos de imagem raster (`GS v 0`) o buffer tem.
    fn faixas_raster(bytes: &[u8]) -> usize {
        bytes.windows(4).filter(|w| *w == [0x1D, 0x76, 0x30, 0x00]).count()
    }

    #[test]
    fn barcode_com_texto_a_direita_sai_numa_faixa_raster_unica() {
        // Lado a lado só existe como uma imagem só: duas faixas significariam
        // barras em cima e preço embaixo, que é justamente o que se quer evitar.
        let bytes = EscPosBuilder::new()
            .paper_width(80)
            .barcode_with_text_right("7898357135512", &[("R$ 27,90".into(), true)])
            .build();

        assert_eq!(faixas_raster(&bytes), 1);
        // O preço vira pixel: não pode sobrar como texto ESC/POS solto.
        assert!(!String::from_utf8_lossy(&bytes).contains("R$ 27,90"));
    }

    #[test]
    fn barcode_com_texto_a_direita_aceita_nao_ean() {
        // Código interno cai em Code 128, como no barcode_ean13.
        let bytes = EscPosBuilder::new()
            .paper_width(80)
            .barcode_with_text_right("ABC-123", &[("R$ 9,90".into(), true)])
            .build();

        assert_eq!(faixas_raster(&bytes), 1);
    }

    #[test]
    fn barcode_largo_demais_volta_a_empilhar() {
        // Dado longo: as barras tomam o papel inteiro e não sobra coluna de texto
        // utilizável. Melhor empilhar do que espremer o preço em 3 caracteres.
        let dados = "CODIGO-INTERNO-MUITO-LONGO-DE-PRODUTO-XYZ";
        let bytes = EscPosBuilder::new()
            .paper_width(80)
            .barcode_with_text_right(dados, &[("R$ 27,90".into(), true)])
            .build();

        assert_eq!(faixas_raster(&bytes), 1, "as barras continuam saindo");
        assert!(
            String::from_utf8_lossy(&bytes).contains("R$ 27,90"),
            "no empilhado o preço volta a ser texto ESC/POS"
        );
    }

    #[test]
    fn barcode_com_texto_cabe_tambem_em_58_mm() {
        let bytes = EscPosBuilder::new()
            .paper_width(58)
            .barcode_with_text_right("7898357135512", &[("R$ 27,90".into(), true)])
            .build();

        assert_eq!(faixas_raster(&bytes), 1);
        assert!(!String::from_utf8_lossy(&bytes).contains("R$ 27,90"));
    }

    #[test]
    fn draw_text_amplia_o_desenho_com_a_escala() {
        let escuros = |escala: u32| {
            let mut img = GrayImage::from_pixel(300, 60, Luma([255u8]));
            draw_text(&mut img, 0, 0, "88", escala, false);
            img.pixels().filter(|p| p.0[0] < 128).count()
        };

        assert!(escuros(1) > 0, "escala 1 nao desenhou nada");
        assert!(
            escuros(2) > escuros(1) * 3,
            "escala 2 deveria cobrir ~4x a area da escala 1"
        );
    }

    #[test]
    fn draw_text_nao_estoura_a_borda_da_imagem() {
        let mut img = GrayImage::from_pixel(20, 20, Luma([255u8]));
        draw_text(&mut img, 0, 0, "88888888", 2, true); // largura pedida >> imagem
        assert_eq!(img.width(), 20);
    }

    #[test]
    fn font_height_escala_so_a_altura() {
        // GS ! n — nibble inferior = altura, superior = largura. Só a altura
        // pode crescer: dobrar a largura corta pela metade as colunas da linha.
        assert_eq!(escala(&EscPosBuilder::new().font_height(2).build()), 0x01);
        assert_eq!(escala(&EscPosBuilder::new().font_height(3).build()), 0x02);
        assert_eq!(escala(&EscPosBuilder::new().font_height(1).build()), 0x00);
    }

    #[test]
    fn font_size_escala_altura_e_largura_juntas() {
        assert_eq!(escala(&EscPosBuilder::new().font_size(2).build()), 0x11);
    }

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

    #[test]
    fn barcode_128_encodes_dados_alfanumericos() {
        // Conjunto B (texto e dígitos em quantidade ímpar). Com o marcador de
        // Start-B errado o crate rejeitava e o barcode saía vazio, sem erro.
        for data in ["PROD-42", "ABC123", "12345"] {
            let bytes = EscPosBuilder::new().barcode_128(data).build();
            assert!(
                bytes.windows(4).any(|w| w == [0x1D, 0x76, 0x30, 0x00]),
                "code128 nao gerou raster para {data:?}"
            );
        }
    }

    /// Posição do cabeçalho raster `GS v 0`, presente em qualquer código de barras.
    fn raster_header(bytes: &[u8]) -> Option<usize> {
        bytes.windows(4).position(|w| w == [0x1D, 0x76, 0x30, 0x00])
    }

    #[test]
    fn barcode_ean13_aceita_12_e_13_digitos() {
        // 12 dígitos: o dígito verificador é calculado.
        let doze = EscPosBuilder::new().barcode_ean13("789835713551").build();
        // 13 dígitos com verificador correto.
        let treze = EscPosBuilder::new().barcode_ean13("7898357135512").build();
        assert!(raster_header(&doze).is_some());
        assert!(raster_header(&treze).is_some());
        assert_eq!(doze, treze, "o 13o digito e o verificador dos 12 primeiros");
    }

    #[test]
    fn barcode_ean13_cai_em_code128_quando_nao_e_ean() {
        // Código interno do PDV: não é EAN, mas a etiqueta não pode sair sem barras.
        let interno = EscPosBuilder::new().barcode_ean13("PROD-42").build();
        let code128 = EscPosBuilder::new().barcode_128("PROD-42").build();
        assert!(raster_header(&interno).is_some());
        assert_eq!(interno, code128);
    }

    #[test]
    fn barcode_ean13_com_verificador_errado_cai_em_code128() {
        let errado = EscPosBuilder::new().barcode_ean13("7898357135519").build();
        let code128 = EscPosBuilder::new().barcode_128("7898357135519").build();
        assert_eq!(errado, code128);
    }

    /// URLs de QR Code de NFC-e reais, do menor ao maior payload: normal, com `idCSC`
    /// longo e contingência off-line (que carrega data, valor e digest extras).
    const QR_NFCE: [&str; 3] = [
        "https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx?p=35260924341498000114650010000103919114792106|2|1|1|a3f1c2d4e5b60718293a4b5c6d7e8f9012345678",
        "https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx?p=35260924341498000114650010000103919114792106|2|1|000001|a3f1c2d4e5b60718293a4b5c6d7e8f9012345678",
        "https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx?p=35260924341498000114650010000103919114792106|2|2|1|2026-09-18T10:46:46-03:00|25.50|a1b2c3d4e5f60718293a4b5c6d7e8f9012345678|a3f1c2d4e5b60718293a4b5c6d7e8f9012345678",
    ];

    #[test]
    fn wrap_hard_parte_palavra_maior_que_a_coluna() {
        // Sem o corte duro a linha saía inteira e o rasterizador a descartava no fim do
        // papel, sem reticências e sem erro.
        let linhas = wrap_hard("COMERCIALDISTRIBUIDORA LTDA", 10);
        assert_eq!(linhas, vec!["COMERCIALD", "ISTRIBUIDO", "RA LTDA"]);
    }

    #[test]
    fn wrap_hard_nunca_excede_a_coluna() {
        // xNome vale no máximo 60 caracteres (maxLength do leiaute). Nenhuma linha pode
        // passar da coluna, com ou sem espaços, em qualquer largura plausível — é isso
        // que garante que o rasterizador nunca precise cortar.
        let casos = [
            "COMERCIO E DISTRIBUICAO DE ALIMENTOS DO VALE DO RIBEIRA LTDA", // 60, com espaços
            "COMERCIOEDISTRIBUICAODEALIMENTOSDOVALEDORIBEIRALTDAMEEPPXYZ1",  // 60, palavra única
        ];
        for nome in casos {
            assert!(nome.chars().count() <= 60, "{nome}");
            let palavras = nome.split_whitespace().count();
            for cols in 8..=24usize {
                let linhas = wrap_hard(nome, cols);
                for l in &linhas {
                    assert!(l.chars().count() <= cols, "coluna {cols} estourou com {l:?}");
                }
                // A quebra por palavra custa no máximo uma linha por palavra; o corte
                // duro acrescenta no máximo ceil(60 / cols).
                assert!(
                    linhas.len() <= palavras + 60_usize.div_ceil(cols),
                    "cols={cols} rendeu {} linhas",
                    linhas.len()
                );
            }
        }
    }

    #[test]
    fn cota_de_pior_caso_do_xnome_no_layout_lateral() {
        // Razão social de 60 caracteres sem um único espaço — o pior caso possível.
        // Nas colunas mínimas medidas, o bloco cabe no orçamento previsto.
        let pior = "A".repeat(60);
        assert_eq!(wrap_hard(&pior, 16).len(), 4); // 80 mm, coluna mínima medida
        assert_eq!(wrap_hard(&pior, 10).len(), 6); // 58 mm, coluna mínima medida
    }

    #[test]
    fn wrap_hard_texto_vazio_rende_linha_vazia() {
        // Entrada em branco é espaçador deliberado no layout lateral.
        assert_eq!(wrap_hard("", 16), vec![String::new()]);
        assert_eq!(wrap_hard("   ", 16), vec![String::new()]);
    }

    #[test]
    fn qr_text_cols_sai_da_geometria_real_e_nao_de_percentual_fixo() {
        // O chute antigo era fixo: 45 % do papel menos o gap, dividido por 16.
        let chute_80 = ((576_u32 * 45 / 100).saturating_sub(8) / 16) as usize; // 15
        let chute_58 = ((384_u32 * 45 / 100).saturating_sub(8) / 16).max(10) as usize; // 10

        for url in QR_NFCE {
            let cols80 = EscPosBuilder::new().paper_width(80).qr_text_cols(url);
            let cols58 = EscPosBuilder::new().paper_width(58).qr_text_cols(url);

            // Faixas medidas com QR de NFC-e real.
            assert!((16..=19).contains(&cols80), "80 mm deu {cols80}");
            assert!((10..=12).contains(&cols58), "58 mm deu {cols58}");
            // E sempre igual ou melhor que o chute que substituiu.
            assert!(cols80 >= chute_80);
            assert!(cols58 >= chute_58);
        }
    }

    #[test]
    fn qr_text_cols_zero_quando_o_payload_nao_vira_qr() {
        let gigante = "x".repeat(8_000);
        assert_eq!(EscPosBuilder::new().qr_text_cols(&gigante), 0);
    }

    #[test]
    fn qr_com_texto_a_direita_sai_numa_faixa_raster_unica() {
        let bytes = EscPosBuilder::new()
            .paper_width(80)
            .qr_with_text_right(QR_NFCE[0], &[("Protocolo:".into(), false)])
            .build();

        assert_eq!(faixas_raster(&bytes), 1);
        assert!(!String::from_utf8_lossy(&bytes).contains("Protocolo:"));
    }

    #[test]
    fn texto_lateral_comeca_na_mesma_altura_que_o_qr() {
        // O texto arrancava em y = 0 e encostava no código de barras da chave, impresso
        // logo acima, enquanto o QR já nascia recuado pela zona de silêncio.
        for url in QR_NFCE {
            for mm in [80u8, 58] {
                let b = EscPosBuilder::new().paper_width(mm);
                let qr = qrcodegen::QrCode::encode_text(url, qrcodegen::QrCodeEcc::Medium).unwrap();
                let g = b.qr_side_geometry(qr.size() as u32);

                assert_eq!(g.text_top, QR_SIDE_QUIET * g.scale);
                assert!(g.text_top > 0, "{mm} mm ficou sem margem superior");
                assert!(g.side_slots > 0, "{mm} mm não coube nenhuma linha ao lado");
                // A última linha prometida ao lado do QR termina antes da base dele...
                let ultima = g.text_top + (g.side_slots - 1) * QR_SIDE_LINE_H;
                assert!(ultima + QR_SIDE_FONT_H <= g.qr_px, "{mm} mm invade a base do QR");
                // ...e a seguinte já não caberia inteira.
                assert!(ultima + QR_SIDE_LINE_H + QR_SIDE_FONT_H > g.qr_px, "{mm} mm desperdiça linha");
            }
        }
    }

    /// Reproduz a decisão de escala do `qr_with_text_right` para um campo isolado:
    /// devolve `(escala, linhas)`.
    fn encaixa(texto: &str, largura: u32) -> (u32, usize) {
        let cols = (largura / QR_SIDE_FONT_W).max(1) as usize;
        if texto.split_whitespace().any(|w| w.chars().count() > cols) {
            let cols_min = (largura / (QR_SIDE_GLYPH * QR_SIDE_SCALE_MIN)).max(1) as usize;
            if texto.chars().count() <= cols_min {
                return (QR_SIDE_SCALE_MIN, 1);
            }
        }
        (QR_SIDE_SCALE, wrap_hard(texto, cols).len())
    }

    #[test]
    fn campo_que_partiria_palavra_encolhe_em_vez_de_quebrar() {
        // Largura da coluna lateral nos dois papéis, com QR de NFC-e real.
        for (mm, url) in [(80u8, QR_NFCE[0]), (58, QR_NFCE[0])] {
            let b = EscPosBuilder::new().paper_width(mm);
            let qr = qrcodegen::QrCode::encode_text(url, qrcodegen::QrCodeEcc::Medium).unwrap();
            let g = b.qr_side_geometry(qr.size() as u32);
            let largura = b.paper_dots - g.text_x;

            // CNPJ com máscara: 24 caracteres com o rótulo, não cabe no corpo nominal.
            // Sai inteiro, numa linha só, em corpo menor — com a pontuação preservada.
            assert_eq!(encaixa("CNPJ: 11.222.333/0001-81", largura), (1, 1), "{mm} mm");

            // Razão social quebra bem entre palavras: continua no corpo nominal.
            let nome = "COMERCIO E DISTRIBUICAO DE ALIMENTOS DO VALE DO RIBEIRA LTDA";
            let (escala, linhas) = encaixa(nome, largura);
            assert_eq!(escala, QR_SIDE_SCALE, "{mm} mm encolheu a razão social");
            assert!(linhas > 1);

            // Rótulos curtos nem são tocados.
            assert_eq!(encaixa("Protocolo:", largura), (QR_SIDE_SCALE, 1), "{mm} mm");
        }
    }

    #[test]
    fn palavra_longa_demais_ate_para_o_corpo_menor_ainda_quebra() {
        // 60 caracteres sem espaço não cabem nem reduzidos: aí não há saída além de
        // partir — mas parte no corpo nominal, não em corpo menor ilegível.
        let (escala, linhas) = encaixa(&"A".repeat(60), 263);
        assert_eq!(escala, QR_SIDE_SCALE);
        assert_eq!(linhas, 4);
    }

    #[test]
    fn qr_com_texto_a_direita_cresce_quando_o_texto_passa_do_qr() {
        // Estourar a altura do QR não pode truncar: a imagem cresce e as linhas
        // excedentes passam a usar a largura inteira do papel.
        let curto: Vec<(String, bool)> = vec![("Protocolo:".into(), false)];
        let longo: Vec<(String, bool)> = (0..40)
            .map(|i| (format!("linha {i}"), false))
            .collect();

        let a = EscPosBuilder::new().paper_width(80).qr_with_text_right(QR_NFCE[0], &curto).build();
        let b = EscPosBuilder::new().paper_width(80).qr_with_text_right(QR_NFCE[0], &longo).build();

        assert_eq!(faixas_raster(&a), 1);
        assert_eq!(faixas_raster(&b), 1);
        assert!(b.len() > a.len());
    }
}
