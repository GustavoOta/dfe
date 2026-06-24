use barcoders::sym::code128::Code128;
use printpdf::*;
use qrcodegen::{QrCode, QrCodeEcc};

use super::pdf_builder_80mm::PdfItem;

const PAGE_WIDTH_MM: f32 = 80.0;
const MARGIN_MM: f32 = 3.0;
const USABLE_WIDTH: f32 = PAGE_WIDTH_MM - MARGIN_MM * 2.0;

const FONT_SIZE_HEADER: f32 = 7.0;
const FONT_SIZE_NORMAL: f32 = 6.5;
const FONT_SIZE_SMALL: f32 = 6.0;
const FONT_SIZE_CREDITS: f32 = 5.0;
const FONT_SIZE_VALUE: f32 = 10.0;

const LINE_HEIGHT: f32 = 3.0;
const SECTION_GAP: f32 = 2.5;
const EMIT_NAME_LINE_CHARS: usize = 36;

pub struct NfcePayment {
    pub t_pag: String,
    pub v_pag: String,
}

pub fn build_pdf_nfce_80mm(
    chave_acesso: &str,
    n_prot: &str,
    dh_recbto: &str,
    emit_x_nome: &str,
    emit_cnpj: &str,
    emit_ie: &str,
    emit_uf: &str,
    emit_x_lgr: &str,
    emit_nro: &str,
    emit_x_bairro: &str,
    emit_x_mun: &str,
    tp_amb: &str,
    serie: &str,
    n_nf: &str,
    dh_emi: &str,
    dest_cpf_cnpj: &str,
    dest_x_nome: &str,
    v_nf: &str,
    v_desc: &str,
    v_prod: &str,
    v_troco: &str,
    v_tot_trib: &str,
    payments: &[NfcePayment],
    inf_cpl: &str,
    items: &[PdfItem],
    qr_code_url: &str,
    qr_side: bool,
) -> Result<Vec<u8>, String> {
    // ── Emitente nome (wrap) ──────────────────────────────
    let emit_name_fallback = "EMITENTE NAO INFORMADO";
    let emit_name = if emit_x_nome.trim().is_empty() {
        emit_name_fallback
    } else {
        emit_x_nome
    };
    let mut emit_name_lines = wrap_text(emit_name, EMIT_NAME_LINE_CHARS);
    if emit_name_lines.is_empty() {
        emit_name_lines.push(emit_name_fallback.to_string());
    }

    // ── QR Code ──────────────────────────────────────────
    let qr_url = if qr_code_url.is_empty() {
        // Fallback: use access key URL pattern if no QR code URL provided
        format!("CH:{}", chave_acesso)
    } else {
        qr_code_url.to_string()
    };
    let qr = QrCode::encode_text(&qr_url, QrCodeEcc::Medium)
        .map_err(|e| format!("Erro ao gerar QR code: {:?}", e))?;
    let qr_n = qr.size() as f32;
    let qr_size_mm = if qr_side {
        33.0
    } else {
        USABLE_WIDTH.min(55.0)
    };
    let qr_module_mm = qr_size_mm / qr_n;
    let qr_actual_size = qr_module_mm * qr_n;
    let qr_x_origin = if qr_side {
        MARGIN_MM
    } else {
        MARGIN_MM + (USABLE_WIDTH - qr_actual_size) / 2.0
    };

    // ── Page height calculation ───────────────────────────
    let extra_emit_lines = emit_name_lines.len().saturating_sub(1) as f32;
    let homolog_h = if tp_amb == "2" {
        LINE_HEIGHT * 2.0 + SECTION_GAP
    } else {
        0.0
    };
    let emit_h = 15.0 + extra_emit_lines * LINE_HEIGHT; // CNPJ/IE/addr + extra name lines
    let items_h = items.len() as f32 * (LINE_HEIGHT * 2.0);
    let payments_h = payments.len() as f32 * LINE_HEIGHT + LINE_HEIGHT; // each + troco
    let consumer_h = if qr_side { 0.0 } else { LINE_HEIGHT };
    let prot_h = if n_prot.is_empty() {
        0.0
    } else {
        LINE_HEIGHT * 2.0 + SECTION_GAP
    };
    let v_tot_trib_f: f64 = v_tot_trib.replace(',', ".").parse().unwrap_or(0.0);
    let trib_h = if v_tot_trib_f > 0.0 {
        LINE_HEIGHT + SECTION_GAP
    } else {
        0.0
    };
    let obs_lines = if inf_cpl.is_empty() {
        0
    } else {
        wrap_text(inf_cpl, 55).len() + 1
    };
    let obs_h = obs_lines as f32 * LINE_HEIGHT + if inf_cpl.is_empty() { 0.0 } else { SECTION_GAP };

    // Bloco fixo: "Consulte" + URL + "CHAVE DE ACESSO" + chave + barcode (sempre acima do QR)
    let barcode_height_mm: f32 = 8.0;
    let compact_line: f32 = 2.2;
    let key_block_h = LINE_HEIGHT * 2.0                                  // "Consulte..." + URL
        + compact_line                                                   // chave formatada
        + barcode_height_mm
        + 1.5; // gap após barcode

    // +1 linha: NF-e No/Serie (movido do topo para cá)
    let nf_info_h = LINE_HEIGHT * 1.0;
    let qr_block_h = key_block_h
        + nf_info_h
        + if qr_side {
            qr_actual_size + SECTION_GAP
        } else {
            qr_actual_size + SECTION_GAP + prot_h
        };

    let base_h = homolog_h
        + LINE_HEIGHT * 1.0       // subtitle only
        + SECTION_GAP
        + emit_h
        + SECTION_GAP
        + LINE_HEIGHT * 1.0       // items header (NF info movido para seção QR)
        + SECTION_GAP
        + items_h
        + 0.5
        + LINE_HEIGHT * 3.0       // totals
        + SECTION_GAP
        + LINE_HEIGHT * 2.0       // payment header
        + payments_h
        + SECTION_GAP
        + qr_block_h
        + consumer_h
        + obs_h
        + trib_h
        + LINE_HEIGHT * 2.0; // credits

    let page_height_mm = (base_h + MARGIN_MM * 2.0).max(120.0);

    // ── Create document ───────────────────────────────────
    let (doc, page1, layer1) = PdfDocument::new(
        "DANFE NFC-e",
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

    // ── Homologação watermark ─────────────────────────────
    if tp_amb == "2" {
        write_center(&layer, &font_bold, 7.0, y, "AMBIENTE DE HOMOLOGACAO");
        y -= LINE_HEIGHT;
        write_center(&layer, &font_bold, 6.0, y, "SEM VALOR FISCAL");
        y -= LINE_HEIGHT;
        draw_line(&layer, y, 0.6);
        y -= SECTION_GAP;
    }

    // ── Título ────────────────────────────────────────────
    write_center(
        &layer,
        &font_bold,
        FONT_SIZE_SMALL,
        y,
        "Documento Auxiliar da NFC-e",
    );
    y -= 1.5;
    draw_line(&layer, y, 0.5);
    y -= SECTION_GAP;

    // ── Emitente ──────────────────────────────────────────
    for line in &emit_name_lines {
        write_center(&layer, &font_bold, FONT_SIZE_NORMAL, y, line);
        y -= LINE_HEIGHT;
    }
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
    y -= LINE_HEIGHT;

    // Endereço do emitente
    {
        let mut addr = String::new();
        if !emit_x_lgr.is_empty() {
            addr.push_str(emit_x_lgr);
            if !emit_nro.is_empty() {
                addr.push_str(&format!(", {}", emit_nro));
            }
        }
        if !emit_x_bairro.is_empty() {
            if !addr.is_empty() {
                addr.push_str(" - ");
            }
            addr.push_str(emit_x_bairro);
        }
        if !emit_x_mun.is_empty() {
            if !addr.is_empty() {
                addr.push_str(" - ");
            }
            addr.push_str(&format!("{}/{}", emit_x_mun, emit_uf));
        }
        if !addr.is_empty() {
            for line in wrap_text(&addr, 48) {
                write_center(&layer, &font, FONT_SIZE_SMALL, y, &line);
                y -= LINE_HEIGHT;
            }
        }
    }

    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── Cabeçalho dos itens ───────────────────────────────
    write_left(&layer, &font_bold, FONT_SIZE_SMALL, y, "#  Descricao");
    write_right(
        &layer,
        &font_bold,
        FONT_SIZE_SMALL,
        y,
        "Qtd   UN    VlUnit    Total",
    );
    y -= LINE_HEIGHT;
    draw_line(&layer, y + 0.5, 0.15);
    y -= 1.5;

    // ── Itens ─────────────────────────────────────────────
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
                "    {}  {}  R$ {}",
                format_decimal_br(&item.q_com),
                item.u_com,
                format_brl(&item.v_un_com),
            ),
        );
        write_right(
            &layer,
            &font,
            FONT_SIZE_SMALL,
            y,
            &format!("R$ {}", format_brl(&item.v_prod)),
        );
        y -= LINE_HEIGHT;
    }

    draw_line(&layer, y + 0.5, 0.3);
    y -= 1.5;

    // ── Totais ────────────────────────────────────────────
    write_left(
        &layer,
        &font,
        FONT_SIZE_NORMAL,
        y,
        &format!("Qtd. Itens: {}", items.len()),
    );
    y -= LINE_HEIGHT;

    let v_desc_f: f64 = v_desc.replace(',', ".").parse().unwrap_or(0.0);
    if v_desc_f > 0.0 {
        write_left(&layer, &font, FONT_SIZE_NORMAL, y, "Subtotal:");
        write_right(
            &layer,
            &font,
            FONT_SIZE_NORMAL,
            y,
            &format!("R$ {}", format_brl(v_prod)),
        );
        y -= LINE_HEIGHT;
        write_left(&layer, &font, FONT_SIZE_NORMAL, y, "Desconto:");
        write_right(
            &layer,
            &font,
            FONT_SIZE_NORMAL,
            y,
            &format!("- R$ {}", format_brl(v_desc)),
        );
        y -= LINE_HEIGHT;
    }

    write_left(&layer, &font_bold, FONT_SIZE_HEADER, y, "TOTAL");
    write_right(
        &layer,
        &font_bold,
        FONT_SIZE_VALUE,
        y,
        &format!("R$ {}", format_brl(v_nf)),
    );
    y -= 1.5;
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── Formas de pagamento ───────────────────────────────
    write_left(
        &layer,
        &font_bold,
        FONT_SIZE_HEADER,
        y,
        "FORMA DE PAGAMENTO",
    );
    write_right(&layer, &font_bold, FONT_SIZE_HEADER, y, "VALOR");
    y -= LINE_HEIGHT;

    for pmt in payments {
        let desc_pag = pag_type_name(&pmt.t_pag);
        write_left(&layer, &font, FONT_SIZE_NORMAL, y, desc_pag);
        write_right(
            &layer,
            &font,
            FONT_SIZE_NORMAL,
            y,
            &format!("R$ {}", format_brl(&pmt.v_pag)),
        );
        y -= LINE_HEIGHT;
    }

    let v_troco_f: f64 = v_troco.replace(',', ".").parse().unwrap_or(0.0);
    write_left(&layer, &font, FONT_SIZE_NORMAL, y, "Troco:");
    write_right(
        &layer,
        &font,
        FONT_SIZE_NORMAL,
        y,
        &format!("R$ {}", format_brl(&format!("{:.2}", v_troco_f))),
    );
    y -= 1.5;
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;

    // ── Chave de acesso + código de barras (acima do QR) ─────────────────────
    write_center(
        &layer,
        &font_bold,
        FONT_SIZE_SMALL,
        y,
        "Consulte pela Chave de Acesso em",
    );
    y -= LINE_HEIGHT;
    write_center(
        &layer,
        &font,
        FONT_SIZE_SMALL,
        y,
        "www.nfce.fazenda.sp.gov.br/consulta",
    );
    y -= LINE_HEIGHT;
    let chave_fmt = format_chave_acesso(chave_acesso);
    write_center(&layer, &font, FONT_SIZE_SMALL - 0.5, y, &chave_fmt);
    y -= compact_line;
    y = draw_barcode_128(&layer, chave_acesso, y, barcode_height_mm)?;
    y -= 1.5;

    // ── QR Code ───────────────────────────────────────────
    if qr_side {
        // Side layout: QR on left, protocol + consumer on right column
        let x_col = MARGIN_MM + qr_actual_size + 2.0;
        let fsz: f32 = 5.5;
        let w_col = USABLE_WIDTH - qr_actual_size - 2.0;
        let col_chars = ((w_col / (fsz * 0.3528 * 556.0 / 1000.0)) as usize)
            .max(10)
            .min(40);

        let y_qr_bottom = draw_qr_code(&layer, &qr, qr_x_origin, y, qr_module_mm)?;

        let mut y_col = y - 1.5;
        if !n_prot.is_empty() {
            y_col -= 0.5;
            layer.use_text("Protocolo:", fsz, Mm(x_col), Mm(y_col), &font_bold);
            y_col -= LINE_HEIGHT;
            layer.use_text(n_prot, fsz, Mm(x_col), Mm(y_col), &font);
            y_col -= LINE_HEIGHT;
            let date_emit = format!("Emissao: {}", format_datetime(dh_emi));
            layer.use_text(&date_emit, fsz, Mm(x_col), Mm(y_col), &font);
            y_col -= LINE_HEIGHT;
        }
        // NF-e nro / série / data emissão (abaixo do protocolo na coluna direita)
        {
            let nf_line = format!("NF-e {:>09} Serie {}", n_nf, serie);
            for line in wrap_text(&nf_line, col_chars) {
                layer.use_text(&line, fsz, Mm(x_col), Mm(y_col), &font);
                y_col -= LINE_HEIGHT;
            }
        }
        {
            let consumer_line = if dest_cpf_cnpj.is_empty() {
                "CONSUMIDOR NAO IDENTIFICADO".to_string()
            } else {
                let doc_digits: String = dest_cpf_cnpj
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                let doc_label = if doc_digits.len() == 14 {
                    "CNPJ"
                } else {
                    "CPF"
                };
                if dest_x_nome.trim().is_empty() {
                    format!(
                        "CONSUMIDOR - {}: {}",
                        doc_label,
                        format_cnpj_cpf(dest_cpf_cnpj)
                    )
                } else {
                    format!(
                        "{} - {}: {}",
                        dest_x_nome,
                        doc_label,
                        format_cnpj_cpf(dest_cpf_cnpj)
                    )
                }
            };
            y_col -= 0.5;
            for line in wrap_text(&consumer_line, col_chars) {
                layer.use_text(&line, fsz, Mm(x_col), Mm(y_col), &font);
                y_col -= LINE_HEIGHT;
            }
        }
        y = y_qr_bottom.min(y_col);

        y -= SECTION_GAP;
    } else {
        // Center layout: QR centralizado, depois protocolo
        y = draw_qr_code(&layer, &qr, qr_x_origin, y, qr_module_mm)?;
        y -= 2.0;
        draw_line(&layer, y, 0.3);
        y -= SECTION_GAP;

        if !n_prot.is_empty() {
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
            y -= LINE_HEIGHT;
        }
        // NF-e nro / série / data emissão (centralizado, abaixo do protocolo)
        write_center(
            &layer,
            &font,
            FONT_SIZE_SMALL,
            y,
            &format!(
                "NF-e No {:>09}  Serie {}  {}",
                n_nf,
                serie,
                format_datetime(dh_emi)
            ),
        );
        y -= 1.5;
        draw_line(&layer, y, 0.3);
        y -= SECTION_GAP;
    }

    // ── Consumidor (centralizado — abaixo do protocolo) ──
    if !qr_side {
        let consumer_line = if dest_cpf_cnpj.is_empty() {
            "CONSUMIDOR NAO IDENTIFICADO".to_string()
        } else {
            let doc_digits: String = dest_cpf_cnpj
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            let doc_label = if doc_digits.len() == 14 {
                "CNPJ"
            } else {
                "CPF"
            };
            if dest_x_nome.trim().is_empty() {
                format!(
                    "CONSUMIDOR - {}: {}",
                    doc_label,
                    format_cnpj_cpf(dest_cpf_cnpj)
                )
            } else {
                format!(
                    "{} - {}: {}",
                    dest_x_nome,
                    doc_label,
                    format_cnpj_cpf(dest_cpf_cnpj)
                )
            }
        };
        for line in wrap_text(&consumer_line, 48) {
            write_center(&layer, &font, FONT_SIZE_SMALL, y, &line);
            y -= LINE_HEIGHT;
        }
    }

    // ── Informações adicionais ────────────────────────────
    if !inf_cpl.is_empty() {
        draw_line(&layer, y, 0.3);
        y -= SECTION_GAP;
        for line in wrap_text(inf_cpl, 55) {
            write_left(&layer, &font, FONT_SIZE_SMALL, y, &line);
            y -= LINE_HEIGHT;
        }
    }

    // ── Tributos aproximados (rodapé) ─────────────────────
    if v_tot_trib_f > 0.0 {
        draw_line(&layer, y, 0.3);
        y -= SECTION_GAP;
        write_center(
            &layer,
            &font,
            FONT_SIZE_SMALL,
            y,
            &format!(
                "Valor Aproximado dos Tributos R$ {} (Fonte: IBPT)",
                format_brl(v_tot_trib)
            ),
        );
        y -= LINE_HEIGHT;
    }

    // ── Créditos ──────────────────────────────────────────
    draw_line(&layer, y, 0.3);
    y -= SECTION_GAP;
    let credits_text = "Gerado por dfe - https://crates.io/crates/dfe";
    write_center(&layer, &font, FONT_SIZE_CREDITS, y, credits_text);

    // Bounds para a anotação de link (mm → pt: 1mm = 72/25.4 pt)
    let mm_to_pt = 72.0_f32 / 25.4;
    let tw = estimate_text_width(credits_text, FONT_SIZE_CREDITS);
    let tx = (MARGIN_MM + (USABLE_WIDTH - tw) / 2.0).max(MARGIN_MM);
    let font_h = FONT_SIZE_CREDITS * 0.3528;
    let annot_rect = [
        tx * mm_to_pt,
        (y - 1.0) * mm_to_pt,
        (tx + tw) * mm_to_pt,
        (y + font_h) * mm_to_pt,
    ];

    let raw = doc
        .save_to_bytes()
        .map_err(|e| format!("Erro ao gerar PDF: {}", e))?;

    // PdfPageReference não expõe add_link_annotation — injeta via lopdf
    let mut lpdf = lopdf::Document::load_mem(&raw)
        .map_err(|e| format!("Erro ao parsear PDF para anotação: {}", e))?;

    let mut a_dict = lopdf::Dictionary::new();
    a_dict.set("S", lopdf::Object::Name(b"URI".to_vec()));
    a_dict.set(
        "URI",
        lopdf::Object::String(
            b"https://crates.io/crates/dfe".to_vec(),
            lopdf::StringFormat::Literal,
        ),
    );
    let mut annot_dict = lopdf::Dictionary::new();
    annot_dict.set("Type", lopdf::Object::Name(b"Annot".to_vec()));
    annot_dict.set("Subtype", lopdf::Object::Name(b"Link".to_vec()));
    annot_dict.set(
        "Rect",
        lopdf::Object::Array(annot_rect.iter().map(|&v| lopdf::Object::Real(v)).collect()),
    );
    annot_dict.set("A", lopdf::Object::Dictionary(a_dict));
    annot_dict.set(
        "Border",
        lopdf::Object::Array(vec![
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(0),
            lopdf::Object::Integer(0),
        ]),
    );
    let annot = lopdf::Object::Dictionary(annot_dict);
    let annot_id = lpdf.add_object(annot);

    let pages = lpdf.get_pages();
    if let Some(&page_id) = pages.get(&1) {
        if let Ok(lopdf::Object::Dictionary(ref mut dict)) = lpdf.get_object_mut(page_id) {
            match dict.get_mut(b"Annots") {
                Ok(lopdf::Object::Array(ref mut arr)) => {
                    arr.push(lopdf::Object::Reference(annot_id));
                }
                _ => {
                    dict.set(
                        "Annots",
                        lopdf::Object::Array(vec![lopdf::Object::Reference(annot_id)]),
                    );
                }
            }
        }
    }

    let mut out = Vec::new();
    lpdf.save_to(&mut out)
        .map_err(|e| format!("Erro ao salvar PDF com link: {}", e))?;

    Ok(out)
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
    let scale = font_size / 1000.0 * 0.3528;
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

/// Draws a QR code at the given position, returning the y coordinate of the bottom edge.
fn draw_qr_code(
    layer: &PdfLayerReference,
    qr: &QrCode,
    x_origin: f32,
    y_top: f32,
    module_mm: f32,
) -> Result<f32, String> {
    let n = qr.size();
    let module_pt = module_mm * 72.0 / 25.4;

    layer.set_outline_color(Color::Greyscale(Greyscale::new(0.0, None)));
    // Use butt line caps so that thick horizontal lines form exact rectangles
    layer.set_line_cap_style(LineCapStyle::Butt);

    for row in 0..n {
        let y_center = y_top - row as f32 * module_mm - module_mm / 2.0;

        let mut col = 0i32;
        while col < n {
            if !qr.get_module(col, row) {
                col += 1;
                continue;
            }
            let run_start = col;
            while col < n && qr.get_module(col, row) {
                col += 1;
            }
            // Draw the run of dark modules as a single thick horizontal line
            let x_start = x_origin + run_start as f32 * module_mm;
            let x_end = x_origin + col as f32 * module_mm;
            layer.set_outline_thickness(module_pt);
            let points = vec![
                (Point::new(Mm(x_start), Mm(y_center)), false),
                (Point::new(Mm(x_end), Mm(y_center)), false),
            ];
            let line = Line {
                points,
                is_closed: false,
            };
            layer.add_line(line);
        }
    }

    let y_bottom = y_top - n as f32 * module_mm;
    Ok(y_bottom)
}

fn draw_barcode_128(
    layer: &PdfLayerReference,
    data: &str,
    y_top: f32,
    bar_height_mm: f32,
) -> Result<f32, String> {
    let barcode_data = format!("\u{0106}{}", data); // Ć = Code128-C (pares numéricos)
    let barcode =
        Code128::new(&barcode_data).map_err(|e| format!("Erro ao gerar barcode: {}", e))?;
    let encoded: Vec<u8> = barcode.encode();

    let total_modules = encoded.len() as f32;
    let module_width_mm = USABLE_WIDTH / total_modules;
    let module_width_pt = module_width_mm * 72.0 / 25.4;

    let y_bottom = y_top - bar_height_mm;

    let mut i = 0usize;
    while i < encoded.len() {
        if encoded[i] == 0 {
            i += 1;
            continue;
        }
        let start = i;
        while i < encoded.len() && encoded[i] == 1 {
            i += 1;
        }
        let run_modules = (i - start) as f32;
        let run_center_x_mm =
            MARGIN_MM + start as f32 * module_width_mm + (run_modules * module_width_mm / 2.0);
        let points = vec![
            (Point::new(Mm(run_center_x_mm), Mm(y_top)), false),
            (Point::new(Mm(run_center_x_mm), Mm(y_bottom)), false),
        ];
        let line = Line {
            points,
            is_closed: false,
        };
        layer.set_outline_color(Color::Greyscale(Greyscale::new(0.0, None)));
        layer.set_outline_thickness(module_width_pt * run_modules);
        layer.add_line(line);
    }

    Ok(y_bottom)
}

fn pag_type_name(t_pag: &str) -> &'static str {
    match t_pag {
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
        "20" => "PIX Estatico",
        "21" => "Credito em Loja",
        "90" => "Sem Pagamento",
        "91" => "Pagamento Posterior",
        "99" => "Outros",
        _ => "Outros",
    }
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
    crate::interno::cnpj_cpf::format_cnpj_cpf(doc)
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
