use barcoders::sym::code128::Code128;
use printpdf::*;

const PAGE_WIDTH_MM: f32 = 80.0;
const MARGIN_MM: f32 = 3.0;
const USABLE_WIDTH: f32 = PAGE_WIDTH_MM - MARGIN_MM * 2.0;

const FONT_SIZE_TITLE: f32 = 8.0;
const FONT_SIZE_HEADER: f32 = 7.0;
const FONT_SIZE_NORMAL: f32 = 6.5;
const FONT_SIZE_SMALL: f32 = 6.0;
const FONT_SIZE_VALUE: f32 = 9.0;

const LINE_HEIGHT: f32 = 3.0;
const SECTION_GAP: f32 = 3.0;

pub struct PdfItem {
    pub n_item: String,
    pub x_prod: String,
    pub q_com: String,
    pub u_com: String,
    pub v_un_com: String,
    pub v_prod: String,
}

pub fn build_pdf_80mm(
    chave_acesso: &str,
    n_prot: &str,
    dh_recbto: &str,
    emit_x_nome: &str,
    emit_uf: &str,
    emit_cnpj: &str,
    emit_ie: &str,
    tp_nf: &str,
    serie: &str,
    n_nf: &str,
    dh_emi: &str,
    dest_x_nome: &str,
    dest_cnpj_cpf: &str,
    dest_uf: &str,
    dest_ie: &str,
    v_nf: &str,
    t_pag: &str,
    inf_cpl: &str,
    items: &[PdfItem],
) -> Result<Vec<u8>, String> {
    // Altura dinamica: base + itens + barcode + observacao
    let barcode_height_mm = 10.0;
    let base_height = 105.0 + barcode_height_mm;
    let items_height = items.len() as f32 * (LINE_HEIGHT * 2.0 + 1.0);
    let obs_lines = if inf_cpl.is_empty() {
        2
    } else {
        wrap_text(inf_cpl, 55).len() + 1
    };
    let obs_height = obs_lines as f32 * LINE_HEIGHT + SECTION_GAP;
    let page_height_mm = (base_height + items_height + obs_height + MARGIN_MM * 2.0).max(135.0);

    let (doc, page1, layer1) = PdfDocument::new(
        "DANFE Simplificado",
        Mm(PAGE_WIDTH_MM),
        Mm(page_height_mm),
        "Layer 1",
    );

    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Erro ao carregar fonte: {}", e))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Erro ao carregar fonte bold: {}", e))?;

    let layer = doc.get_page(page1).get_layer(layer1);
    let mut y = page_height_mm - MARGIN_MM;

    // ── TITULO ──────────────────────────────
    write_center(
        &layer,
        &font_bold,
        FONT_SIZE_TITLE,
        y,
        "DANFE SIMPLIFICADO - ETIQUETA",
    );
    y -= 1.5;
    draw_line(&layer, y, 0.6);
    y -= SECTION_GAP;

    // ── EMITENTE ────────────────────────────
    write_center(&layer, &font_bold, FONT_SIZE_HEADER, y, "EMITENTE");
    y -= LINE_HEIGHT;
    write_center(&layer, &font, FONT_SIZE_NORMAL, y, emit_x_nome);
    y -= LINE_HEIGHT;
    write_center(
        &layer,
        &font,
        FONT_SIZE_SMALL,
        y,
        &format!(
            "CNPJ: {}  IE: {}  UF: {}",
            format_cnpj_cpf(emit_cnpj),
            emit_ie,
            emit_uf
        ),
    );
    y -= 1.5;
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── DADOS DA NF-e ───────────────────────
    write_center(&layer, &font_bold, FONT_SIZE_HEADER, y, "DADOS DA NF-e");
    y -= LINE_HEIGHT;
    let tipo_op = match tp_nf {
        "0" => "ENTRADA",
        "1" => "SAIDA",
        _ => tp_nf,
    };
    write_center(
        &layer,
        &font,
        FONT_SIZE_NORMAL,
        y,
        &format!("Tipo: {}  Serie: {}  No: {}", tipo_op, serie, n_nf),
    );
    y -= LINE_HEIGHT;
    write_center(
        &layer,
        &font,
        FONT_SIZE_NORMAL,
        y,
        &format!("Emissao: {}", format_datetime(dh_emi)),
    );
    y -= 1.5;
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── DESTINATARIO ────────────────────────
    write_center(&layer, &font_bold, FONT_SIZE_HEADER, y, "DESTINATARIO");
    y -= LINE_HEIGHT;
    write_center(&layer, &font, FONT_SIZE_NORMAL, y, dest_x_nome);
    y -= LINE_HEIGHT;
    let mut dest_info = format!(
        "CNPJ/CPF: {}  UF: {}",
        format_cnpj_cpf(dest_cnpj_cpf),
        dest_uf
    );
    if !dest_ie.is_empty() {
        dest_info.push_str(&format!("  IE: {}", dest_ie));
    }
    write_center(&layer, &font, FONT_SIZE_SMALL, y, &dest_info);
    y -= 1.5;
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── PRODUTOS / SERVICOS ─────────────────
    write_center(&layer, &font_bold, FONT_SIZE_HEADER, y, "PRODUTOS");
    y -= LINE_HEIGHT + 0.5;
    write_left(
        &layer,
        &font_bold,
        FONT_SIZE_SMALL,
        y,
        "#  Descricao | Qtd x Un x Vl Unit",
    );
    write_right(&layer, &font_bold, FONT_SIZE_SMALL, y, "Valor");
    y -= LINE_HEIGHT;
    draw_line(&layer, y + 1.0, 0.15);
    y -= 1.5;

    for item in items {
        let desc = truncate_str(&item.x_prod, 42);
        write_left(
            &layer,
            &font,
            FONT_SIZE_SMALL,
            y,
            &format!("{}  {}", item.n_item, desc),
        );
        y -= LINE_HEIGHT;
        write_left(
            &layer,
            &font,
            FONT_SIZE_SMALL,
            y,
            &format!(
                "    {} {} x R$ {}",
                format_decimal_br(&item.q_com),
                item.u_com,
                format_brl(&item.v_un_com)
            ),
        );
        write_right(
            &layer,
            &font,
            FONT_SIZE_SMALL,
            y,
            &format!("R$ {}", format_brl(&item.v_prod)),
        );
        y -= LINE_HEIGHT + 0.5;
    }

    draw_line(&layer, y + 0.5, 0.3);
    y -= SECTION_GAP;

    // ── FORMA DE PAGAMENTO ──────────────────
    let desc_pag = match t_pag {
        "01" => "Dinheiro",
        "02" => "Cheque",
        "03" => "Cartao de Credito",
        "04" => "Cartao de Debito",
        "05" => "Credito Loja",
        "10" => "Vale Alimentacao",
        "11" => "Vale Refeicao",
        "12" => "Vale Presente",
        "13" => "Vale Combustivel",
        "14" => "Duplicata Mercantil",
        "15" => "Boleto Bancario",
        "16" => "Deposito Bancario",
        "17" => "PIX",
        "18" => "Transferencia bancaria",
        "19" => "Programa fidelidade",
        "90" => "Sem Pagamento",
        "99" => "Outros",
        _ => t_pag,
    };
    write_left(
        &layer,
        &font_bold,
        FONT_SIZE_HEADER,
        y,
        "Forma de Pagamento",
    );
    write_right(&layer, &font, FONT_SIZE_NORMAL, y, desc_pag);
    y -= 1.5;
    y -= SECTION_GAP;

    // ── VALOR TOTAL ─────────────────────────
    write_left(
        &layer,
        &font_bold,
        FONT_SIZE_HEADER,
        y,
        "VALOR TOTAL DA NF-e",
    );
    write_right(
        &layer,
        &font_bold,
        FONT_SIZE_VALUE,
        y,
        &format!("R$ {}", format_brl(v_nf)),
    );
    y -= LINE_HEIGHT;
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── CHAVE DE ACESSO ─────────────────────
    write_center(&layer, &font_bold, FONT_SIZE_HEADER, y, "CHAVE DE ACESSO");
    y -= 1.5;

    // Codigo de barras Code128
    y = draw_barcode_128(&layer, chave_acesso, y, barcode_height_mm)?;
    y -= 3.0;

    let chave_fmt = format_chave_acesso(chave_acesso);
    write_center(&layer, &font, FONT_SIZE_NORMAL, y, &chave_fmt);
    y -= 1.5;
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── PROTOCOLO DE AUTORIZACAO ────────────
    write_center(
        &layer,
        &font_bold,
        FONT_SIZE_HEADER,
        y,
        "PROTOCOLO DE AUTORIZACAO",
    );
    y -= LINE_HEIGHT;
    write_center(
        &layer,
        &font,
        FONT_SIZE_NORMAL,
        y,
        &format!("{} - {}", n_prot, format_datetime(dh_recbto)),
    );
    y -= 1.5;
    draw_line(&layer, y, 0.6);

    // ── OBSERVACAO DO CONTRIBUINTE ──────────
    y -= SECTION_GAP;
    write_center(
        &layer,
        &font_bold,
        FONT_SIZE_HEADER,
        y,
        "OBSERVACAO DO CONTRIBUINTE",
    );
    y -= LINE_HEIGHT;
    if inf_cpl.is_empty() {
        write_left(
            &layer,
            &font,
            FONT_SIZE_SMALL,
            y,
            "Nenhuma informacao adicional",
        );
        y -= LINE_HEIGHT;
    } else {
        for line in wrap_text(inf_cpl, 55) {
            write_left(&layer, &font, FONT_SIZE_SMALL, y, &line);
            y -= LINE_HEIGHT;
        }
    }
    draw_line(&layer, y + 1.0, 0.6);

    // ── CREDITOS ────────────────────────────
    y -= SECTION_GAP;
    write_center(
        &layer,
        &font,
        FONT_SIZE_SMALL,
        y,
        "Gerado por dfe - https://crates.io/crates/dfe",
    );

    let bytes = doc
        .save_to_bytes()
        .map_err(|e| format!("Erro ao gerar PDF: {}", e))?;

    Ok(bytes)
}

// ── Helpers ─────────────────────────────────────────────────

fn write_left(layer: &PdfLayerReference, font: &IndirectFontRef, size: f32, y: f32, text: &str) {
    layer.use_text(text, size, Mm(MARGIN_MM), Mm(y), font);
}

fn write_center(layer: &PdfLayerReference, font: &IndirectFontRef, size: f32, y: f32, text: &str) {
    let text_width = estimate_text_width(text, size);
    let x = (MARGIN_MM + (USABLE_WIDTH - text_width) / 2.0).max(MARGIN_MM);
    layer.use_text(text, size, Mm(x), Mm(y), font);
}

fn write_right(layer: &PdfLayerReference, font: &IndirectFontRef, size: f32, y: f32, text: &str) {
    let text_width = estimate_text_width(text, size);
    let x = (MARGIN_MM + USABLE_WIDTH - text_width).max(MARGIN_MM);
    layer.use_text(text, size, Mm(x), Mm(y), font);
}

fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    let scale = font_size / 1000.0 * 0.3528; // pt to mm (1pt = 0.3528mm)
    text.chars()
        .map(|c| match c {
            ' ' => 278.0,
            '!' | 'i' | 'l' | ':' | ';' | ',' | '.' | '\'' => 278.0,
            'f' | 'j' | 't' => 333.0,
            'r' => 333.0,
            'I' | '[' | ']' | '(' | ')' | '/' | '-' => 278.0,
            'a' | 'c' | 'e' | 'o' | 's' => 556.0,
            'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'p' | 'q' | 'u' | 'v' | 'x' | 'y' | 'z' => 556.0,
            'm' | 'w' => 778.0,
            'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'K' | 'N' | 'P' | 'R' | 'S' | 'T'
            | 'U' | 'V' | 'X' | 'Y' | 'Z' => 667.0,
            'M' | 'O' | 'Q' | 'W' => 778.0,
            'J' | 'L' => 556.0,
            '0'..='9' => 556.0,
            _ => 556.0,
        })
        .sum::<f32>()
        * scale
}

fn draw_line(layer: &PdfLayerReference, y: f32, thickness: f32) {
    let points = vec![
        (Point::new(Mm(MARGIN_MM), Mm(y)), false),
        (Point::new(Mm(MARGIN_MM + USABLE_WIDTH), Mm(y)), false),
    ];
    let line = Line {
        points,
        is_closed: false,
    };
    layer.set_outline_thickness(thickness);
    layer.add_line(line);
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = chars[..max_chars - 3].iter().collect();
        format!("{}...", truncated)
    }
}

fn format_chave_acesso(chave: &str) -> String {
    chave
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_decimal_br(value: &str) -> String {
    value.replace('.', ",")
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn format_brl(value: &str) -> String {
    let v: f64 = value.replace(',', ".").parse().unwrap_or(0.0);
    let formatted = format!("{:.2}", v);
    let parts: Vec<&str> = formatted.split('.').collect();
    let int_part = parts[0];
    let dec_part = parts[1];
    let digits: Vec<char> = int_part.chars().collect();
    let with_dots: String =
        digits
            .iter()
            .rev()
            .enumerate()
            .fold(String::new(), |mut acc, (i, &c)| {
                if i > 0 && i % 3 == 0 {
                    acc.insert(0, '.');
                }
                acc.insert(0, c);
                acc
            });
    format!("{},{}", with_dots, dec_part)
}

fn format_cnpj_cpf(doc: &str) -> String {
    let d: String = doc.chars().filter(|c| c.is_ascii_digit()).collect();
    match d.len() {
        14 => format!(
            "{}.{}.{}/{}-{}",
            &d[0..2],
            &d[2..5],
            &d[5..8],
            &d[8..12],
            &d[12..14]
        ),
        11 => format!("{}.{}.{}-{}", &d[0..3], &d[3..6], &d[6..9], &d[9..11]),
        _ => doc.to_string(),
    }
}

fn format_datetime(dt: &str) -> String {
    if dt.len() >= 19 {
        let date = &dt[..10];
        let time = &dt[11..19];
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{} {}", parts[2], parts[1], parts[0], time);
        }
    }
    dt.to_string()
}

fn draw_barcode_128(
    layer: &PdfLayerReference,
    data: &str,
    y_top: f32,
    bar_height_mm: f32,
) -> Result<f32, String> {
    let barcode_data = format!("\u{0106}{}", data); // Ć = Code128 character-set C (numeric pairs)
    let barcode =
        Code128::new(&barcode_data).map_err(|e| format!("Erro ao gerar barcode: {}", e))?;
    let encoded: Vec<u8> = barcode.encode();

    let total_modules = encoded.len() as f32;
    let module_width = USABLE_WIDTH / total_modules;

    let y_bottom = y_top - bar_height_mm;

    for (i, &bar) in encoded.iter().enumerate() {
        if bar == 1 {
            let x = MARGIN_MM + i as f32 * module_width + module_width / 2.0;
            let points = vec![
                (Point::new(Mm(x), Mm(y_top)), false),
                (Point::new(Mm(x), Mm(y_bottom)), false),
            ];
            let line = Line {
                points,
                is_closed: false,
            };
            layer.set_outline_color(Color::Greyscale(Greyscale::new(0.0, None)));
            layer.set_outline_thickness(module_width);
            layer.add_line(line);
        }
    }

    Ok(y_bottom)
}
