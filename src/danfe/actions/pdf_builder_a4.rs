use barcoders::sym::code128::Code128;
use printpdf::*;

// ── Layout constants (ABNT/NFe spec) ────────────

const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const M: f32 = 2.0; // margin all sides
const BODY_W: f32 = PAGE_W - 2.0 * M; // 206mm

// Header column widths: 41% emitente | 17.5% crachá | remainder chave
const COL_EMIT: f32 = 84.0;
const COL_DANFE: f32 = 36.0;
const COL_CHAVE: f32 = BODY_W - COL_EMIT - COL_DANFE; // 86mm

// Section heights (mm)
const H_CAB: f32 = 44.0;
const H_SUBROW: f32 = 8.0; // bottom strip of header (CNPJ/IE/dates)
const H_DEST: f32 = 22.0;
const H_IMP: f32 = 13.0;
const H_TRANSP: f32 = 22.0;
const H_ITEMS_HDR: f32 = 8.5; // faixa título (3.5 mm) + faixa cabeçalhos colunas (5.0 mm)
const H_ITEM: f32 = 4.0;
const H_ADIC_MIN: f32 = 18.0;
const H_PROT: f32 = 5.0;

// Font sizes (pt)
const FS_LBL: f32 = 5.5;
const FS_SM: f32 = 6.5;
const FS_BD: f32 = 8.0;
const FS_DANFE_BIG: f32 = 20.0;
const FS_SUBTIT: f32 = 6.0;

const LH: f32 = 3.5;

// Field text offsets from the TOP edge of the field box (mm)
// Baseline sits below the top so the text cap appears inside the box.
const FLBL: f32 = 2.0; // label baseline: 2 mm below field top (~1 mm visual padding)
const FVAL: f32 = 6.0; // value baseline: 6 mm below field top (4 mm below label)

pub struct PdfItemA4 {
    pub n_item: String,
    pub c_prod: String,
    pub x_prod: String,
    pub ncm: String,
    pub cfop: String,
    pub u_com: String,
    pub q_com: String,
    pub v_un_com: String,
    pub v_prod: String,
    pub v_desc: String,
    pub v_ipi: String,
    pub p_icms: String,
    pub v_icms: String,
    pub p_ipi: String,
}

#[allow(clippy::too_many_arguments)]
pub fn build_pdf_a4(
    logo_bytes: Option<&[u8]>,
    chave_acesso: &str,
    n_prot: &str,
    dh_recbto: &str,
    tp_nf: &str,
    n_nf: &str,
    serie: &str,
    dh_emi: &str,
    dh_sai_ent: &str,
    nat_op: &str,
    tp_amb: &str,
    // emitente
    emit_x_nome: &str,
    emit_cnpj: &str,
    emit_ie: &str,
    emit_iest: &str,
    emit_x_lgr: &str,
    emit_nro: &str,
    emit_x_bairro: &str,
    emit_x_mun: &str,
    emit_uf: &str,
    emit_cep: &str,
    emit_fone: &str,
    // destinatário
    dest_x_nome: &str,
    dest_cnpj_cpf: &str,
    dest_ie: &str,
    dest_x_lgr: &str,
    dest_nro: &str,
    dest_x_bairro: &str,
    dest_x_mun: &str,
    dest_uf: &str,
    dest_cep: &str,
    dest_fone: &str,
    // totais
    v_bc: &str,
    v_icms: &str,
    v_bc_st: &str,
    v_st: &str,
    v_prod: &str,
    v_frete: &str,
    v_seg: &str,
    v_desc: &str,
    v_outro: &str,
    v_ipi: &str,
    v_nf: &str,
    // transporte
    transp_x_nome: &str,
    transp_cnpj_cpf: &str,
    mod_frete_label: &str,
    transp_uf: &str,
    placa: &str,
    marca: &str,
    vol_qtd: &str,
    vol_esp: &str,
    vol_peso_b: &str,
    vol_peso_l: &str,
    // info adicional
    inf_cpl: &str,
    inf_fisco: &str,
    // itens
    items: &[PdfItemA4],
) -> Result<Vec<u8>, String> {
    // Fixed overhead: all sections except items rows
    let fixed_h = H_CAB
        + H_DEST
        + H_IMP
        + H_TRANSP
        + H_ITEMS_HDR
        + (items.len().max(1) as f32 * H_ITEM)
        + H_ADIC_MIN
        + H_PROT;
    let page_h = (2.0 * M + fixed_h).max(PAGE_H);

    let (doc, page1, layer1) = PdfDocument::new("DANFE NF-e A4", Mm(PAGE_W), Mm(page_h), "Layer 1");
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| format!("Erro fonte: {}", e))?;
    let font_b = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| format!("Erro fonte bold: {}", e))?;
    let layer = doc.get_page(page1).get_layer(layer1);

    // y increases upward; start from top of printable area
    let mut y = page_h - M;

    // ══════════════════════════════════════════════════════════════════════
    // BLOCO 1 — CABEÇALHO (47 mm)
    // ══════════════════════════════════════════════════════════════════════
    let cab_top = y;
    let cab_bot = cab_top - H_CAB;
    let subrow_top = cab_bot + H_SUBROW; // separator between main header and sub-row

    draw_rect(&layer, M, cab_bot, BODY_W, H_CAB, 0.4);

    // Column positions
    let x_danfe = M + COL_EMIT;
    let x_chave = x_danfe + COL_DANFE;

    vline(&layer, x_danfe, cab_bot, H_CAB, 0.3);
    vline(&layer, x_chave, cab_bot, H_CAB, 0.3);
    hline(&layer, M, subrow_top, BODY_W, 0.3);

    // ── Coluna emitente (0..84 mm) ──
    {
        let x = M + 0.8;
        let max_w = COL_EMIT - 1.6;
        let logo_max_h = 18.0_f32; // altura máxima reservada para o logo (mm)

        // y inicial: se há logo, renderiza primeiro e empurra texto abaixo
        let mut yi = cab_top - FLBL;
        if let Some(bytes) = logo_bytes {
            match embed_logo(&layer, bytes, M + 0.8, max_w, cab_top - 0.5, logo_max_h) {
                Ok(logo_bot) => yi = logo_bot - 1.5,
                Err(_) => {} // logo inválido → ignora, mantém posição padrão
            }
        }

        t(&layer, &font, x, yi, FS_LBL, "IDENTIFICAÇÃO DO EMITENTE");
        yi -= LH;
        let max_w = COL_EMIT - 1.6;
        text_wrap_clipped(
            &layer,
            &font_b,
            x,
            &mut yi,
            FS_BD,
            emit_x_nome,
            max_w,
            subrow_top + 1.0,
        );
        let ender = format!("{}, {} - {}", emit_x_lgr, emit_nro, emit_x_bairro);
        text_wrap_clipped(
            &layer,
            &font,
            x,
            &mut yi,
            FS_SM,
            &ender,
            max_w,
            subrow_top + 1.0,
        );
        let ciduf = format!("{} - {} CEP: {}", emit_x_mun, emit_uf, format_cep(emit_cep));
        text_wrap_clipped(
            &layer,
            &font,
            x,
            &mut yi,
            FS_SM,
            &ciduf,
            max_w,
            subrow_top + 1.0,
        );
        if !emit_fone.is_empty() && yi > subrow_top + 1.0 {
            t(&layer, &font, x, yi, FS_SM, &format!("Fone: {}", emit_fone));
        }
    }

    // ── Crachá DANFE (centro) ──
    {
        let cx = x_danfe;
        let cw = COL_DANFE;
        let x = cx + 0.8;

        // "DANFE" grande centralizado
        let tw = estimate_text_width("DANFE", FS_DANFE_BIG);
        let danfe_x = cx + (cw - tw) / 2.0;
        t(
            &layer,
            &font_b,
            danfe_x,
            cab_top - 9.0,
            FS_DANFE_BIG,
            "DANFE",
        );

        // Subtítulo (deslocado para cima para dar espaço ao separador)
        let sub1 = "Documento Auxiliar da";
        let sub2 = "Nota Fiscal Eletrônica";
        let s1w = estimate_text_width(sub1, FS_SUBTIT);
        let s2w = estimate_text_width(sub2, FS_SUBTIT);
        t(
            &layer,
            &font,
            cx + (cw - s1w) / 2.0,
            cab_top - 13.5,
            FS_SUBTIT,
            sub1,
        );
        t(
            &layer,
            &font,
            cx + (cw - s2w) / 2.0,
            cab_top - 17.0,
            FS_SUBTIT,
            sub2,
        );

        // Separador horizontal no meio da coluna
        let mid_y = (cab_top + subrow_top) / 2.0;
        hline(&layer, cx, mid_y, cw, 0.2);

        // Tipo operação — ABAIXO do separador (y decresce para baixo no PDF)
        let tp_lbl = match tp_nf {
            "0" => "0 - ENTRADA",
            "1" => "1 - SAÍDA",
            _ => tp_nf,
        };
        t(&layer, &font, x, mid_y - FLBL, FS_LBL, "TIPO DE OPERAÇÃO");
        t(&layer, &font_b, x, mid_y - FVAL, FS_SM, tp_lbl);

        // Número / Série — abaixo do tipo
        t(
            &layer,
            &font,
            x,
            mid_y - FVAL - LH - 1.0,
            FS_LBL,
            "Nº NF-e / SÉRIE",
        );
        t(
            &layer,
            &font_b,
            x,
            mid_y - FVAL - LH - 1.0 - LH,
            FS_SM,
            &format!("{} / {}", n_nf, serie),
        );
    }

    // ── Chave de acesso + protocolo + nat. op. ──
    {
        let x = x_chave + 0.8;
        let max_w = COL_CHAVE - 1.6;
        let mut yi = cab_top - FLBL;

        t(&layer, &font, x, yi, FS_LBL, "CHAVE DE ACESSO");
        yi -= LH;
        let chave_fmt = format_chave_acesso(chave_acesso);
        text_wrap_clipped(
            &layer,
            &font_b,
            x,
            &mut yi,
            FS_SM,
            &chave_fmt,
            max_w,
            subrow_top + 1.0,
        );

        // Barcode Code128 da chave de acesso (44 dígitos sem espaços)
        // text_wrap_clipped já subtrai LH após a última linha (3.5mm);
        // adicionamos de volta LH e deixamos só 1.5mm de gap visual.
        let bar_h = 10.0;
        yi += LH - 1.5;
        if yi - bar_h > subrow_top + 1.0 {
            let _ = draw_barcode_a4(
                &layer,
                chave_acesso,
                x_chave + 2.0,
                COL_CHAVE - 4.0,
                yi,
                bar_h,
            );
            yi -= bar_h;
        }

        yi -= 0.5;
        if yi > subrow_top + 4.0 {
            hline(&layer, x_chave, yi, COL_CHAVE, 0.2);
            yi -= 2.5;
        }

        if !n_prot.is_empty() && yi > subrow_top + 1.0 {
            t(
                &layer,
                &font,
                x,
                yi,
                FS_LBL,
                "PROTOCOLO DE AUTORIZAÇÃO DE USO",
            );
            yi -= LH;
            if yi > subrow_top + 1.0 {
                t(
                    &layer,
                    &font_b,
                    x,
                    yi,
                    FS_SM,
                    &format!("{} {}", n_prot, format_datetime(dh_recbto)),
                );
                yi -= LH + 1.0;
            }
        } else if tp_amb == "2" && yi > subrow_top + 1.0 {
            t(
                &layer,
                &font_b,
                x,
                yi,
                FS_SM,
                "HOMOLOGAÇÃO - SEM VALOR FISCAL",
            );
            yi -= LH + 1.0;
        }

        if yi > subrow_top + 4.0 {
            hline(&layer, x_chave, yi, COL_CHAVE, 0.2);
            yi -= 2.5;
        }

        if yi > subrow_top + 1.0 {
            t(&layer, &font, x, yi, FS_LBL, "NATUREZA DA OPERAÇÃO");
            yi -= LH;
            if yi > subrow_top + 1.0 {
                text_wrap_clipped(
                    &layer,
                    &font,
                    x,
                    &mut yi,
                    FS_SM,
                    nat_op,
                    max_w,
                    subrow_top + 1.0,
                );
            }
        }
        let _ = yi;
    }

    // ── Sub-row: CNPJ | IE | IEST | Data Emissão | Data Saída/Entrada ──
    {
        // Column widths summing to BODY_W = 206
        // CNPJ:50 | IE:40 | IEST:40 | DtEmi:38 | DtEnt:38 = 206
        let subrow_cols: &[(&str, &str, f32)] = &[
            ("CNPJ", &format_cnpj_cpf(emit_cnpj), 50.0),
            ("I.E.", emit_ie, 40.0),
            ("I.E. SUBS. TRIB.", emit_iest, 40.0),
            ("DATA DE EMISSÃO", &format_date(dh_emi), 38.0),
            ("DATA SAÍ./ENT.", &format_date(dh_sai_ent), 38.0),
        ];

        let lbl_y = subrow_top - FLBL;
        let val_y = subrow_top - FVAL;
        let mut xi = M;
        for (i, (lbl, val, w)) in subrow_cols.iter().enumerate() {
            t(&layer, &font, xi + 0.8, lbl_y, FS_LBL, lbl);
            text_truncated(&layer, &font, xi + 0.8, val_y, FS_SM, val, w - 1.6);
            xi += w;
            if i < subrow_cols.len() - 1 {
                vline(&layer, xi, cab_bot, H_SUBROW, 0.2);
            }
        }
    }

    y = cab_bot;

    // ══════════════════════════════════════════════════════════════════════
    // BLOCO 2 — DESTINATÁRIO / REMETENTE (25 mm)
    // ══════════════════════════════════════════════════════════════════════
    let dest_top = y;
    let dest_bot = dest_top - H_DEST;
    draw_rect(&layer, M, dest_bot, BODY_W, H_DEST, 0.4);

    t(
        &layer,
        &font,
        M + 0.8,
        dest_top - FLBL,
        FS_LBL,
        "DESTINATÁRIO / REMETENTE",
    );
    hline(&layer, M, dest_top - 3.5, BODY_W, 0.2);

    // Row 1: Nome | CNPJ/CPF | IE | Data Emissão
    // Widths: 96 | 52 | 28 | 30 = 206
    {
        let r1_top = dest_top - 3.5;
        let r1_bot = dest_top - 13.5; // 10mm for row 1
        let x_cnpj = M + 96.0;
        let x_ie = M + 148.0;
        let x_dte = M + 178.0;

        t(
            &layer,
            &font,
            M + 0.8,
            r1_top - FLBL,
            FS_LBL,
            "NOME / RAZÃO SOCIAL",
        );
        text_truncated(
            &layer,
            &font_b,
            M + 0.8,
            r1_top - FVAL,
            FS_SM,
            dest_x_nome,
            x_cnpj - M - 1.6,
        );

        vline(&layer, x_cnpj, r1_bot, r1_top - r1_bot, 0.2);
        t(
            &layer,
            &font,
            x_cnpj + 0.8,
            r1_top - FLBL,
            FS_LBL,
            "CNPJ / CPF",
        );
        text_truncated(
            &layer,
            &font,
            x_cnpj + 0.8,
            r1_top - FVAL,
            FS_SM,
            &format_cnpj_cpf(dest_cnpj_cpf),
            x_ie - x_cnpj - 1.6,
        );

        vline(&layer, x_ie, r1_bot, r1_top - r1_bot, 0.2);
        t(
            &layer,
            &font,
            x_ie + 0.8,
            r1_top - FLBL,
            FS_LBL,
            "INSCRIÇÃO ESTADUAL",
        );
        text_truncated(
            &layer,
            &font,
            x_ie + 0.8,
            r1_top - FVAL,
            FS_SM,
            dest_ie,
            x_dte - x_ie - 1.6,
        );

        vline(&layer, x_dte, r1_bot, r1_top - r1_bot, 0.2);
        t(
            &layer,
            &font,
            x_dte + 0.8,
            r1_top - FLBL,
            FS_LBL,
            "DATA DA EMISSÃO",
        );
        t(
            &layer,
            &font,
            x_dte + 0.8,
            r1_top - FVAL,
            FS_SM,
            &format_date(dh_emi),
        );

        hline(&layer, M, r1_bot, BODY_W, 0.2);
    }

    // Row 2: Endereço | Bairro | CEP | Município | UF | Fone/Fax
    // Widths: 58 | 32 | 22 | 36 | 7 | 51 = 206
    {
        let r2_top = dest_top - 13.5;
        let x_bairro = M + 58.0;
        let x_cep = M + 90.0;
        let x_mun = M + 112.0;
        let x_uf = M + 148.0;
        let x_fone = M + 155.0;
        let h_r2 = r2_top - dest_bot;

        t(&layer, &font, M + 0.8, r2_top - FLBL, FS_LBL, "ENDEREÇO");
        text_truncated(
            &layer,
            &font,
            M + 0.8,
            r2_top - FVAL,
            FS_SM,
            &format!("{}, {}", dest_x_lgr, dest_nro),
            x_bairro - M - 1.6,
        );

        vline(&layer, x_bairro, dest_bot, h_r2, 0.2);
        t(
            &layer,
            &font,
            x_bairro + 0.8,
            r2_top - FLBL,
            FS_LBL,
            "BAIRRO",
        );
        text_truncated(
            &layer,
            &font,
            x_bairro + 0.8,
            r2_top - FVAL,
            FS_SM,
            dest_x_bairro,
            x_cep - x_bairro - 1.6,
        );

        vline(&layer, x_cep, dest_bot, h_r2, 0.2);
        t(&layer, &font, x_cep + 0.8, r2_top - FLBL, FS_LBL, "CEP");
        t(
            &layer,
            &font,
            x_cep + 0.8,
            r2_top - FVAL,
            FS_SM,
            &format_cep(dest_cep),
        );

        vline(&layer, x_mun, dest_bot, h_r2, 0.2);
        t(
            &layer,
            &font,
            x_mun + 0.8,
            r2_top - FLBL,
            FS_LBL,
            "MUNICÍPIO",
        );
        text_truncated(
            &layer,
            &font,
            x_mun + 0.8,
            r2_top - FVAL,
            FS_SM,
            dest_x_mun,
            x_uf - x_mun - 1.6,
        );

        vline(&layer, x_uf, dest_bot, h_r2, 0.2);
        t(&layer, &font, x_uf + 0.8, r2_top - FLBL, FS_LBL, "UF");
        t(&layer, &font, x_uf + 0.8, r2_top - FVAL, FS_SM, dest_uf);

        vline(&layer, x_fone, dest_bot, h_r2, 0.2);
        t(
            &layer,
            &font,
            x_fone + 0.8,
            r2_top - FLBL,
            FS_LBL,
            "FONE/FAX",
        );
        text_truncated(
            &layer,
            &font,
            x_fone + 0.8,
            r2_top - FVAL,
            FS_SM,
            dest_fone,
            M + BODY_W - x_fone - 1.6,
        );
    }

    y = dest_bot;

    // ══════════════════════════════════════════════════════════════════════
    // BLOCO 3 — CÁLCULO DO IMPOSTO (18 mm)
    // ══════════════════════════════════════════════════════════════════════
    let imp_top = y;
    let imp_bot = imp_top - H_IMP;
    draw_rect(&layer, M, imp_bot, BODY_W, H_IMP, 0.4);

    t(
        &layer,
        &font,
        M + 0.8,
        imp_top - FLBL,
        FS_LBL,
        "CÁLCULO DO IMPOSTO",
    );
    hline(&layer, M, imp_top - 3.5, BODY_W, 0.2);

    {
        let tributos: &[(&str, &str)] = &[
            ("BASE CÁLC. ICMS", v_bc),
            ("VALOR DO ICMS", v_icms),
            ("BASE CÁLC. ST", v_bc_st),
            ("VL ICMS ST", v_st),
            ("VL PRODUTOS", v_prod),
            ("VL FRETE", v_frete),
            ("VL SEGURO", v_seg),
            ("DESCONTO", v_desc),
            ("OUTRAS DESP.", v_outro),
            ("VALOR DO IPI", v_ipi),
            ("VL TOTAL NF", v_nf),
        ];

        let col_w = BODY_W / tributos.len() as f32;
        // separator at imp_top - 3.5; fields start below it
        let sep_y = imp_top - 3.5;
        let lbl_y = sep_y - FLBL;
        let val_y = sep_y - FVAL;

        let mut xi = M;
        for (i, (lbl, val)) in tributos.iter().enumerate() {
            text_truncated(&layer, &font, xi + 0.5, lbl_y, FS_LBL, lbl, col_w - 1.0);
            let vfmt = format_brl(val);
            if i == tributos.len() - 1 {
                t(&layer, &font_b, xi + 0.5, val_y, FS_SM, &vfmt);
            } else {
                t(&layer, &font, xi + 0.5, val_y, FS_SM, &vfmt);
            }
            xi += col_w;
            if i < tributos.len() - 1 {
                vline(&layer, xi, imp_bot, H_IMP - 3.5, 0.2);
            }
        }
    }

    y = imp_bot;

    // ══════════════════════════════════════════════════════════════════════
    // BLOCO 4 — TRANSPORTADOR / VOLUMES (25 mm)
    // ══════════════════════════════════════════════════════════════════════
    let transp_top = y;
    let transp_bot = transp_top - H_TRANSP;
    draw_rect(&layer, M, transp_bot, BODY_W, H_TRANSP, 0.4);

    t(
        &layer,
        &font,
        M + 0.8,
        transp_top - FLBL,
        FS_LBL,
        "TRANSPORTADOR / VOLUMES TRANSPORTADOS",
    );
    hline(&layer, M, transp_top - 3.5, BODY_W, 0.2);

    // Row 1: Razão Social | CNPJ/CPF | Frete | Placa | UF | Marca
    // Widths: 68 | 40 | 30 | 17 | 7 | 44 = 206
    {
        let r1_top = transp_top - 3.5;
        let r1_bot = transp_top - 13.5;
        let x_tc = M + 68.0;
        let x_tf = M + 108.0;
        let x_tp = M + 138.0;
        let x_tu = M + 155.0;
        let x_tm = M + 162.0;
        let h_r1 = r1_top - r1_bot;

        t(
            &layer,
            &font,
            M + 0.8,
            r1_top - FLBL,
            FS_LBL,
            "RAZÃO SOCIAL",
        );
        text_truncated(
            &layer,
            &font,
            M + 0.8,
            r1_top - FVAL,
            FS_SM,
            transp_x_nome,
            x_tc - M - 1.6,
        );

        vline(&layer, x_tc, r1_bot, h_r1, 0.2);
        t(
            &layer,
            &font,
            x_tc + 0.8,
            r1_top - FLBL,
            FS_LBL,
            "CNPJ / CPF",
        );
        text_truncated(
            &layer,
            &font,
            x_tc + 0.8,
            r1_top - FVAL,
            FS_SM,
            &format_cnpj_cpf(transp_cnpj_cpf),
            x_tf - x_tc - 1.6,
        );

        vline(&layer, x_tf, r1_bot, h_r1, 0.2);
        t(
            &layer,
            &font,
            x_tf + 0.8,
            r1_top - FLBL,
            FS_LBL,
            "FRETE POR CONTA",
        );
        text_truncated(
            &layer,
            &font,
            x_tf + 0.8,
            r1_top - FVAL,
            FS_SM,
            mod_frete_label,
            x_tp - x_tf - 1.6,
        );

        vline(&layer, x_tp, r1_bot, h_r1, 0.2);
        t(&layer, &font, x_tp + 0.8, r1_top - FLBL, FS_LBL, "PLACA");
        t(&layer, &font, x_tp + 0.8, r1_top - FVAL, FS_SM, placa);

        vline(&layer, x_tu, r1_bot, h_r1, 0.2);
        t(&layer, &font, x_tu + 0.8, r1_top - FLBL, FS_LBL, "UF");
        t(&layer, &font, x_tu + 0.8, r1_top - FVAL, FS_SM, transp_uf);

        vline(&layer, x_tm, r1_bot, h_r1, 0.2);
        t(&layer, &font, x_tm + 0.8, r1_top - FLBL, FS_LBL, "MARCA");
        text_truncated(
            &layer,
            &font,
            x_tm + 0.8,
            r1_top - FVAL,
            FS_SM,
            marca,
            M + BODY_W - x_tm - 1.6,
        );

        hline(&layer, M, r1_bot, BODY_W, 0.2);
    }

    // Row 2: Quantidade | Espécie | Nº/Marca Vol. | Peso Bruto | Peso Líquido
    // Widths: 38 | 38 | 52 | 38 | 40 = 206
    {
        let r2_top = transp_top - 13.5;
        let x_e = M + 38.0;
        let x_l = M + 76.0;
        let x_pb = M + 128.0;
        let x_pl = M + 166.0;
        let h_r2 = r2_top - transp_bot;

        t(&layer, &font, M + 0.8, r2_top - FLBL, FS_LBL, "QUANTIDADE");
        t(&layer, &font, M + 0.8, r2_top - FVAL, FS_SM, vol_qtd);

        vline(&layer, x_e, transp_bot, h_r2, 0.2);
        t(&layer, &font, x_e + 0.8, r2_top - FLBL, FS_LBL, "ESPÉCIE");
        t(&layer, &font, x_e + 0.8, r2_top - FVAL, FS_SM, vol_esp);

        vline(&layer, x_l, transp_bot, h_r2, 0.2);
        t(
            &layer,
            &font,
            x_l + 0.8,
            r2_top - FLBL,
            FS_LBL,
            "MARCA/Nº (VOL.)",
        );

        vline(&layer, x_pb, transp_bot, h_r2, 0.2);
        t(
            &layer,
            &font,
            x_pb + 0.8,
            r2_top - FLBL,
            FS_LBL,
            "PESO BRUTO",
        );
        t(
            &layer,
            &font,
            x_pb + 0.8,
            r2_top - FVAL,
            FS_SM,
            &format_decimal_br(vol_peso_b),
        );

        vline(&layer, x_pl, transp_bot, h_r2, 0.2);
        t(
            &layer,
            &font,
            x_pl + 0.8,
            r2_top - FLBL,
            FS_LBL,
            "PESO LÍQUIDO",
        );
        t(
            &layer,
            &font,
            x_pl + 0.8,
            r2_top - FVAL,
            FS_SM,
            &format_decimal_br(vol_peso_l),
        );
    }

    y = transp_bot;

    // ══════════════════════════════════════════════════════════════════════
    // BLOCO 5 — DADOS DOS PRODUTOS / SERVIÇOS
    // ══════════════════════════════════════════════════════════════════════
    let items_top = y;
    let items_rows_h = items.len().max(1) as f32 * H_ITEM;
    let items_total_h = H_ITEMS_HDR + items_rows_h;
    let items_bot = items_top - items_total_h;
    draw_rect(&layer, M, items_bot, BODY_W, items_total_h, 0.4);

    t(
        &layer,
        &font,
        M + 0.8,
        items_top - FLBL,
        FS_LBL,
        "DADOS DOS PRODUTOS / SERVIÇOS",
    );
    let title_sep_y = items_top - 3.5;
    hline(&layer, M, title_sep_y, BODY_W, 0.2);
    let col_hdr_sep_y = items_top - H_ITEMS_HDR;
    hline(&layer, M, col_hdr_sep_y, BODY_W, 0.3);

    // Columns: # | Código | Descrição | NCM | CFOP | UN | Desc | IPI% | Vl IPI | ICMS% | Vl ICMS | Qtde | Vl Unit | Vl Total
    // Widths sum = 206
    let cols: &[(&str, f32)] = &[
        ("#", 6.0),
        ("CÓDIGO", 14.0),
        ("DESCRIÇÃO DO PRODUTO", 58.0),
        ("NCM/SH", 12.0),
        ("CFOP", 8.0),
        ("UN", 7.0),
        ("DESCONTO", 13.0),
        ("IPI %", 9.0),
        ("VL IPI", 13.0),
        ("ICMS %", 9.0),
        ("VL ICMS", 13.0),
        ("QUANT.", 14.0),
        ("VL UNIT.", 14.0),
        ("VL TOTAL", 16.0),
    ];

    // Column headers: texto alinhado à esquerda para colunas de texto,
    // à direita para colunas numéricas (índice >= 6: QUANT. em diante)
    {
        // Baseline 2 mm abaixo do separador de título (na faixa dos col-headers)
        let hdr_lbl_y = title_sep_y - FLBL;
        let mut xi = M;
        for (i, (lbl, w)) in cols.iter().enumerate() {
            let lw = estimate_text_width(lbl, FS_LBL);
            let lx = if i >= 6 {
                // Alinha à direita com 0.5 mm de padding interno
                (xi + w - lw - 0.5).max(xi + 0.3)
            } else {
                xi + ((w - lw) / 2.0).max(0.3)
            };
            t(&layer, &font, lx, hdr_lbl_y, FS_LBL, lbl);
            xi += w;
            if xi < M + BODY_W - 0.1 {
                vline(&layer, xi, items_bot, items_total_h - 3.5, 0.2);
            }
        }
    }

    // Data rows
    let mut row_y = col_hdr_sep_y; // top of first data row (abaixo do separador de col-headers)
    for item in items {
        if row_y - H_ITEM < items_bot - 0.1 {
            break;
        }
        let row_bot = row_y - H_ITEM;
        hline(&layer, M, row_bot, BODY_W, 0.1);
        let text_y = row_bot + H_ITEM * 0.28; // baseline ~28% from bottom of row

        let row_vals: &[(&str, bool)] = &[
            (&item.n_item, false),
            (&item.c_prod, false),
            (&item.x_prod, false),
            (&item.ncm, false),
            (&item.cfop, false),
            (&item.u_com, false),
            (&format_brl(&item.v_desc), true),
            (&format_decimal_br(&item.p_ipi), true),
            (&format_brl(&item.v_ipi), true),
            (&format_decimal_br(&item.p_icms), true),
            (&format_brl(&item.v_icms), true),
            (&format_decimal_br(&item.q_com), true),
            (&format_brl(&item.v_un_com), true),
            (&format_brl(&item.v_prod), true),
        ];

        let mut xi = M;
        for (j, ((val, right_align), (_, w))) in row_vals.iter().zip(cols.iter()).enumerate() {
            if *right_align {
                text_right_in(&layer, &font, xi, text_y, FS_SM, val, w - 0.5);
            } else if j == 2 {
                text_truncated(&layer, &font, xi + 0.5, text_y, FS_SM, val, w - 1.0);
            } else {
                t(&layer, &font, xi + 0.5, text_y, FS_SM, val);
            }
            xi += w;
        }

        row_y = row_bot;
    }

    y = items_bot;

    // ══════════════════════════════════════════════════════════════════════
    // BLOCO 6 — DADOS ADICIONAIS
    // ══════════════════════════════════════════════════════════════════════
    let adic_top = y;
    // Use remaining space above the protocolo bar, minimum H_ADIC_MIN
    let adic_h = (adic_top - M - H_PROT).max(H_ADIC_MIN);
    let adic_bot = adic_top - adic_h;
    draw_rect(&layer, M, adic_bot, BODY_W, adic_h, 0.4);

    // Split 66% complementar | 34% fisco
    let split_x = M + (BODY_W * 0.66).round();
    vline(&layer, split_x, adic_bot, adic_h, 0.3);

    t(
        &layer,
        &font,
        M + 0.8,
        adic_top - FLBL,
        FS_LBL,
        "INFORMAÇÕES COMPLEMENTARES",
    );
    t(
        &layer,
        &font,
        split_x + 0.8,
        adic_top - FLBL,
        FS_LBL,
        "RESERVADO AO FISCO",
    );
    hline(&layer, M, adic_top - 3.5, BODY_W, 0.2);

    {
        let mut yi = adic_top - 5.5;
        let max_w = split_x - M - 1.6;
        for line in wrap_text_by_width(inf_cpl, max_w, FS_SM) {
            if yi < adic_bot + 1.0 {
                break;
            }
            t(&layer, &font, M + 0.8, yi, FS_SM, &line);
            yi -= LH;
        }
    }
    {
        let mut yi = adic_top - 5.5;
        let max_w = M + BODY_W - split_x - 1.6;
        for line in wrap_text_by_width(inf_fisco, max_w, FS_SM) {
            if yi < adic_bot + 1.0 {
                break;
            }
            t(&layer, &font, split_x + 0.8, yi, FS_SM, &line);
            yi -= LH;
        }
    }

    y = adic_bot;

    // ══════════════════════════════════════════════════════════════════════
    // BLOCO 7 — RODAPÉ (5 mm)
    // ══════════════════════════════════════════════════════════════════════
    let prot_top = y;
    let prot_bot = prot_top - H_PROT;
    draw_rect(&layer, M, prot_bot, BODY_W, H_PROT, 0.4);

    let footer_y = prot_top - H_PROT / 2.0 - 0.5;
    let footer_msg = "Gerado com dfe crate \u{2022} https://crates.io/crates/dfe";
    let footer_w = estimate_text_width(footer_msg, FS_SM);
    t(
        &layer,
        &font,
        M + (BODY_W - footer_w) / 2.0,
        footer_y,
        FS_SM,
        footer_msg,
    );

    let _ = y;

    // ── Salvar ────────────────────────────────────────────────────────────
    doc.save_to_bytes()
        .map_err(|e| format!("Erro ao gerar PDF: {}", e))
}

// ── Logo ───────────────────────────────────────────────────────────────────

/// Renderiza o logotipo do emitente na coluna esquerda do cabeçalho.
/// Retorna o y da borda inferior do logo (em mm, coordenada PDF crescente p/ cima).
fn embed_logo(
    layer: &PdfLayerReference,
    bytes: &[u8],
    x_start: f32,
    col_w: f32,
    y_top: f32,
    max_h: f32,
) -> Result<f32, String> {
    // Usa caminho absoluto para evitar ambiguidade com o re-export do printpdf
    let img = ::image::load_from_memory(bytes).map_err(|e| format!("Logo inválido: {}", e))?;

    let (img_w, img_h) = (img.width(), img.height());
    if img_w == 0 || img_h == 0 {
        return Err("Logo com dimensões zero".to_string());
    }

    // Converte para RGB (descarta alpha, fundo branco implícito)
    let rgb = img.to_rgb8();

    // Escala proporcional para caber em col_w × max_h (nunca ampliar)
    let dpi: f32 = 300.0;
    let nat_w = img_w as f32 * 25.4 / dpi;
    let nat_h = img_h as f32 * 25.4 / dpi;
    let scale = (col_w / nat_w).min(max_h / nat_h).min(1.0);

    let final_w = nat_w * scale;
    let final_h = nat_h * scale;

    // Centraliza horizontalmente na coluna
    let x_img = x_start + (col_w - final_w) / 2.0;
    let y_bottom = y_top - final_h;

    let img_obj = ImageXObject {
        width: Px(img_w as usize),
        height: Px(img_h as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        image_data: rgb.into_raw(),
        image_filter: None,
        interpolate: true,
        clipping_bbox: None,
        smask: None,
    };

    Image { image: img_obj }.add_to_layer(
        layer.clone(),
        ImageTransform {
            translate_x: Some(Mm(x_img)),
            translate_y: Some(Mm(y_bottom)),
            scale_x: Some(scale),
            scale_y: Some(scale),
            dpi: Some(dpi),
            ..Default::default()
        },
    );

    Ok(y_bottom)
}

// ── Barcode ────────────────────────────────────────────────────────────────

fn draw_barcode_a4(
    layer: &PdfLayerReference,
    data: &str,
    x_start: f32,
    width_mm: f32,
    y_top: f32,
    bar_height_mm: f32,
) -> Result<(), String> {
    // Code128-C: otimizado para pares de dígitos (chave de acesso = 44 dígitos)
    let barcode_data = format!("\u{0106}{}", data);
    let barcode =
        Code128::new(&barcode_data).map_err(|e| format!("Erro ao gerar barcode: {}", e))?;
    let encoded: Vec<u8> = barcode.encode();

    let total_modules = encoded.len() as f32;
    let module_width_mm = width_mm / total_modules;
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
        let run_center_x =
            x_start + start as f32 * module_width_mm + run_modules * module_width_mm / 2.0;
        let points = vec![
            (Point::new(Mm(run_center_x), Mm(y_top)), false),
            (Point::new(Mm(run_center_x), Mm(y_bottom)), false),
        ];
        let line = Line {
            points,
            is_closed: false,
        };
        layer.set_outline_color(Color::Greyscale(Greyscale::new(0.0, None)));
        layer.set_outline_thickness(module_width_pt * run_modules);
        layer.add_line(line);
    }

    Ok(())
}

// ── Primitivos de desenho ──────────────────────────────────────────────────

fn t(layer: &PdfLayerReference, font: &IndirectFontRef, x: f32, y: f32, size: f32, text: &str) {
    layer.use_text(text, size, Mm(x), Mm(y), font);
}

fn draw_rect(layer: &PdfLayerReference, x: f32, y_bot: f32, w: f32, h: f32, thickness: f32) {
    let pts = vec![
        (Point::new(Mm(x), Mm(y_bot)), false),
        (Point::new(Mm(x + w), Mm(y_bot)), false),
        (Point::new(Mm(x + w), Mm(y_bot + h)), false),
        (Point::new(Mm(x), Mm(y_bot + h)), false),
    ];
    let line = Line {
        points: pts,
        is_closed: true,
    };
    layer.set_outline_thickness(thickness);
    layer.add_line(line);
}

fn hline(layer: &PdfLayerReference, x: f32, y: f32, w: f32, thickness: f32) {
    let pts = vec![
        (Point::new(Mm(x), Mm(y)), false),
        (Point::new(Mm(x + w), Mm(y)), false),
    ];
    let line = Line {
        points: pts,
        is_closed: false,
    };
    layer.set_outline_thickness(thickness);
    layer.add_line(line);
}

fn vline(layer: &PdfLayerReference, x: f32, y_bot: f32, h: f32, thickness: f32) {
    let pts = vec![
        (Point::new(Mm(x), Mm(y_bot)), false),
        (Point::new(Mm(x), Mm(y_bot + h)), false),
    ];
    let line = Line {
        points: pts,
        is_closed: false,
    };
    layer.set_outline_thickness(thickness);
    layer.add_line(line);
}

fn text_truncated(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    x: f32,
    y: f32,
    size: f32,
    text: &str,
    max_w: f32,
) {
    let mut s = text.to_string();
    while !s.is_empty() && estimate_text_width(&s, size) > max_w {
        s.pop();
    }
    if s.len() < text.len() && s.len() > 3 {
        s.truncate(s.len() - 3);
        s.push_str("...");
    }
    layer.use_text(&s, size, Mm(x), Mm(y), font);
}

fn text_right_in(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    x_left: f32,
    y: f32,
    size: f32,
    text: &str,
    col_w: f32,
) {
    let tw = estimate_text_width(text, size);
    let x = (x_left + col_w - tw).max(x_left + 0.3);
    layer.use_text(text, size, Mm(x), Mm(y), font);
}

fn text_wrap_clipped(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    x: f32,
    y: &mut f32,
    size: f32,
    text: &str,
    max_w: f32,
    min_y: f32,
) {
    for line in wrap_text_by_width(text, max_w, size) {
        if *y < min_y {
            break;
        }
        layer.use_text(&line, size, Mm(x), Mm(*y), font);
        *y -= LH;
    }
}

fn wrap_text_by_width(text: &str, max_w: f32, size: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        // Palavra cabe inteira na linha atual?
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };

        if estimate_text_width(&candidate, size) <= max_w {
            current = candidate;
        } else if estimate_text_width(word, size) <= max_w {
            // Palavra cabe em linha nova
            if !current.is_empty() {
                lines.push(current);
            }
            current = word.to_string();
        } else {
            // Palavra é longa demais — quebra por caractere
            if !current.is_empty() {
                lines.push(current);
            }
            let mut buf = String::new();
            for ch in word.chars() {
                let next = format!("{}{}", buf, ch);
                if estimate_text_width(&next, size) <= max_w {
                    buf = next;
                } else {
                    if !buf.is_empty() {
                        lines.push(buf);
                    }
                    buf = ch.to_string();
                }
            }
            current = buf;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    let scale = font_size / 1000.0 * 0.3528;
    text.chars()
        .map(|c| match c {
            ' ' => 278.0,
            'i' | 'l' | ':' | ';' | ',' | '.' | '\'' | '!' | '|' => 278.0,
            'f' | 'j' | 't' | 'r' => 333.0,
            'I' | '[' | ']' | '(' | ')' | '/' | '-' => 278.0,
            'a' | 'c' | 'e' | 'o' | 's' | 'b' | 'd' | 'g' | 'h' | 'k' | 'n' | 'p' | 'q' | 'u'
            | 'v' | 'x' | 'y' | 'z' => 556.0,
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

// ── Formatadores ───────────────────────────────────────────────────────────

fn format_chave_acesso(chave: &str) -> String {
    chave
        .chars()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
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

fn format_cep(cep: &str) -> String {
    let d: String = cep.chars().filter(|c| c.is_ascii_digit()).collect();
    if d.len() == 8 {
        format!("{}-{}", &d[0..5], &d[5..8])
    } else {
        cep.to_string()
    }
}

fn format_brl(value: &str) -> String {
    if value.is_empty() {
        return "0,00".to_string();
    }
    let v: f64 = value.replace(',', ".").parse().unwrap_or(0.0);
    let formatted = format!("{:.2}", v);
    let parts: Vec<&str> = formatted.split('.').collect();
    let digits: Vec<char> = parts[0].chars().collect();
    let with_dots = digits
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
    format!("{},{}", with_dots, parts[1])
}

fn format_decimal_br(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    value.replace('.', ",")
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

fn format_date(dt: &str) -> String {
    if dt.len() >= 10 {
        let parts: Vec<&str> = dt[..10].split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
        }
    }
    dt.to_string()
}
