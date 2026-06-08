use crate::error::{DfeError, Result};
use crate::xml_extractor::{XmlExtractor, XmlExtractorSignature};

use super::EscPosBuilder;

const SPACING_NORMAL: u8 = 30;
const SPACING_DIVIDER: u8 = 4;

/// Builder fluente para impressão de **NFC-e** (modelo 65) em impressoras térmicas via ESC/POS,
/// espelhando com máxima fidelidade o layout do DANFE NFC-e 80 mm em PDF.
///
/// Aceita XML como caminho de arquivo (`.xml`) ou string direta.
/// Suporta papel 80 mm e 58 mm e dois posicionamentos de QR Code.
/// Em ambientes Windows, o método `print` envia os bytes
/// diretamente à impressora via job RAW sem diálogo.
///
/// # Exemplo
///
/// ```no_run
/// use dfe::EscPosNFCeBuilder;
///
/// # fn example() -> Result<(), dfe::DfeError> {
/// // Apenas gerar os bytes
/// let bytes = EscPosNFCeBuilder::new()
///     .xml("./nota_nfce.xml")
///     .paper_width(80)
///     .build()?;
///
/// // Imprimir diretamente (Windows)
/// EscPosNFCeBuilder::new()
///     .xml("./nota_nfce.xml")
///     .printer_name("EPSON TM-T20 Receipt")
///     .print()?;
/// # Ok(())
/// # }
/// ```
pub struct EscPosNFCeBuilder {
    xml: Option<String>,
    qr_side: bool,
    paper_width: u8,
    printer_name: String,
    paper_dots: Option<u32>,
    printer_dpi: Option<u32>,
}

impl EscPosNFCeBuilder {
    /// Cria um builder com QR Code centralizado e papel de 80 mm.
    pub fn new() -> Self {
        Self {
            xml: None,
            qr_side: false,
            paper_width: 80,
            printer_name: String::new(),
            paper_dots: None,
            printer_dpi: None,
        }
    }

    /// String XML do `nfeProc` autorizado, ou caminho de arquivo terminado em `".xml"`.
    pub fn xml(mut self, src: impl Into<String>) -> Self {
        self.xml = Some(src.into());
        self
    }

    /// QR Code alinhado à esquerda (compacto, tamanho 3). Padrão: centralizado (tamanho 5).
    pub fn qr_side(mut self) -> Self {
        self.qr_side = true;
        self
    }

    /// Largura do papel em milímetros. Use `80` ou `58`. Padrão: `80`.
    pub fn paper_width(mut self, mm: u8) -> Self {
        self.paper_width = mm;
        self
    }

    /// Nome da impressora Windows para uso em [`print`](Self::print).
    /// Deve coincidir exatamente com o nome exibido no Painel de Controle.
    pub fn printer_name(mut self, name: impl Into<String>) -> Self {
        self.printer_name = name.into();
        self
    }

    /// Resolução nativa da impressora em DPI.
    ///
    /// Use `203` para impressoras padrão (padrão) · `300` para alta resolução.
    /// Calcula automaticamente a largura imprimível e o espaçamento correto entre
    /// faixas do QR Code no layout lateral [`qr_side`](Self::qr_side).
    pub fn printer_dpi(mut self, dpi: u32) -> Self {
        self.printer_dpi = Some(dpi);
        self
    }

    /// Largura imprimível real da impressora em dots nativos.
    ///
    /// Alternativa a [`printer_dpi`](Self::printer_dpi). Para 300 DPI / 80 mm: `850`.
    pub fn printable_dots(mut self, dots: u32) -> Self {
        self.paper_dots = Some(dots);
        self
    }

    /// Lê o XML, extrai os dados da NFC-e e gera os bytes ESC/POS.
    ///
    /// # Erros
    ///
    /// Retorna [`DfeError`] se o XML for inválido, o arquivo não existir
    /// ou o documento não for NFC-e (modelo 65).
    pub fn build(self) -> Result<Vec<u8>> {
        let params = self.extract_params()?;
        build_receipt(params)
    }

    /// Gera os bytes ESC/POS e envia diretamente à impressora `printer_name` como job RAW.
    ///
    /// Disponível apenas em **Windows** (`cfg(windows)`).
    /// Retorna erro se o nome da impressora não foi informado ou se o job falhar.
    #[cfg(target_os = "windows")]
    #[cfg_attr(docsrs, doc(cfg(target_os = "windows")))]
    pub fn print(self) -> Result<()> {
        let printer = self.printer_name.clone();
        if printer.is_empty() {
            return Err(DfeError::Configuracao(
                "printer_name não informado".to_string(),
            ));
        }
        let bytes = self.build()?;
        raw_print_windows(&printer, &bytes)
    }

    // ── Extração de dados do XML ──────────────────────────────────────────────

    fn extract_params(self) -> Result<BuildParams> {
        let src = self
            .xml
            .ok_or_else(|| DfeError::Configuracao("XML não informado".to_string()))?;

        let extractor = XmlExtractor::new();
        let nfe_proc = if src.trim_end().ends_with(".xml") {
            extractor.nfe_proc_from_file(&src)?
        } else {
            extractor.nfe_proc_from_string(&src)?
        };

        let inf = &nfe_proc.nfe.inf_nfe;

        let mod_ = inf.ide.mod_.as_deref().unwrap_or("55");
        if mod_ != "65" {
            return Err(DfeError::Configuracao(format!(
                "EscPosNFCeBuilder espera modelo 65 (NFC-e), recebeu modelo {mod_}"
            )));
        }

        let chave_acesso = inf
            .id
            .as_deref()
            .unwrap_or("")
            .strip_prefix("NFe")
            .unwrap_or(inf.id.as_deref().unwrap_or(""))
            .to_string();

        let prot = &nfe_proc.prot_nfe.inf_prot;
        let n_prot = prot.as_ref().and_then(|p| p.n_prot.clone()).unwrap_or_default();
        let dh_recbto = prot.as_ref().and_then(|p| p.dh_recbto.clone()).unwrap_or_default();

        let emit = &inf.emit;
        let emit_x_nome = emit
            .x_fant
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .or_else(|| emit.x_nome.as_deref().filter(|v| !v.trim().is_empty()))
            .unwrap_or_default()
            .to_string();
        let emit_cnpj = emit.cnpj.clone().unwrap_or_default();
        let emit_ie = emit.ie.clone().unwrap_or_default();
        let emit_uf = emit.ender_emit.uf.clone().unwrap_or_default();
        let emit_x_lgr = emit.ender_emit.x_lgr.clone().unwrap_or_default();
        let emit_nro = emit.ender_emit.nro.clone().unwrap_or_default();
        let emit_x_bairro = emit.ender_emit.x_bairro.clone().unwrap_or_default();
        let emit_x_mun = emit.ender_emit.x_mun.clone().unwrap_or_default();

        let ide = &inf.ide;
        let tp_amb = ide.tp_amb.clone().unwrap_or_default();
        let serie = ide.serie.clone().unwrap_or_default();
        let n_nf = ide.n_nf.clone().unwrap_or_default();
        let dh_emi = ide.dh_emi.clone().unwrap_or_default();

        let dest = &inf.dest;
        let dest_cpf_cnpj = dest
            .as_ref()
            .and_then(|d| d.cnpj.clone().or_else(|| d.cpf.clone()))
            .unwrap_or_default();
        let dest_x_nome = dest.as_ref().and_then(|d| d.x_nome.clone()).unwrap_or_default();

        let icms_tot = inf.total.icms_tot.as_ref();
        let v_nf = icms_tot.and_then(|t| t.v_nf.clone()).unwrap_or_default();
        let v_desc = icms_tot.and_then(|t| t.v_desc.clone()).unwrap_or_default();
        let v_prod_total = icms_tot.and_then(|t| t.v_prod.clone()).unwrap_or_default();

        let v_tot_trib_items: f64 = inf
            .det
            .iter()
            .filter_map(|det| det.imposto.v_tot_trib.as_deref())
            .filter_map(|s| s.parse::<f64>().ok())
            .sum();
        let v_tot_trib = if v_tot_trib_items > 0.0 {
            format!("{:.2}", v_tot_trib_items)
        } else {
            icms_tot.and_then(|t| t.v_tot_trib.clone()).unwrap_or_default()
        };

        let v_troco = inf.pag.v_troco.clone().unwrap_or_else(|| "0.00".to_string());

        let payments: Vec<(String, String)> = inf
            .pag
            .det_pag
            .iter()
            .map(|d| {
                (
                    d.t_pag.clone().unwrap_or_default(),
                    d.v_pag.clone().unwrap_or_default(),
                )
            })
            .collect();

        let inf_cpl = inf.inf_adic.inf_cpl.clone().unwrap_or_default();

        let qr_code_url = nfe_proc
            .nfe
            .inf_nfe_supl
            .as_ref()
            .and_then(|s| s.qr_code.clone())
            .unwrap_or_else(|| format!("CH:{chave_acesso}"));

        let url_chave = nfe_proc
            .nfe
            .inf_nfe_supl
            .as_ref()
            .and_then(|s| s.url_chave.clone())
            .unwrap_or_default();

        let items: Vec<NfceItem> = inf
            .det
            .iter()
            .map(|det| {
                let p = &det.prod;
                NfceItem {
                    n_item: det.n_item.clone().unwrap_or_default(),
                    x_prod: p.x_prod.clone().unwrap_or_default(),
                    q_com: p.q_com.clone().unwrap_or_default(),
                    u_com: p.u_com.clone().unwrap_or_default(),
                    v_un_com: p.v_un_com.clone().unwrap_or_default(),
                    v_prod: p.v_prod.clone().unwrap_or_default(),
                }
            })
            .collect();

        let paper_dots = if let Some(dpi) = self.printer_dpi {
            let printable_mm: f32 = if self.paper_width >= 80 { 72.0 } else { 48.0 };
            (printable_mm * dpi as f32 / 25.4).round() as u32
        } else {
            self.paper_dots.unwrap_or(if self.paper_width >= 80 { 576 } else { 384 })
        };

        Ok(BuildParams {
            chave_acesso,
            n_prot,
            dh_recbto,
            emit_x_nome,
            emit_cnpj,
            emit_ie,
            emit_uf,
            emit_x_lgr,
            emit_nro,
            emit_x_bairro,
            emit_x_mun,
            tp_amb,
            serie,
            n_nf,
            dh_emi,
            dest_cpf_cnpj,
            dest_x_nome,
            v_nf,
            v_desc,
            v_prod_total,
            v_tot_trib,
            v_troco,
            payments,
            inf_cpl,
            items,
            qr_code_url,
            url_chave,
            qr_side: self.qr_side,
            paper_width: self.paper_width,
            paper_dots,
        })
    }
}

impl Default for EscPosNFCeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tipos internos ────────────────────────────────────────────────────────────

struct NfceItem {
    n_item: String,
    x_prod: String,
    q_com: String,
    u_com: String,
    v_un_com: String,
    v_prod: String,
}

struct BuildParams {
    chave_acesso: String,
    n_prot: String,
    dh_recbto: String,
    emit_x_nome: String,
    emit_cnpj: String,
    emit_ie: String,
    emit_uf: String,
    emit_x_lgr: String,
    emit_nro: String,
    emit_x_bairro: String,
    emit_x_mun: String,
    tp_amb: String,
    serie: String,
    n_nf: String,
    dh_emi: String,
    dest_cpf_cnpj: String,
    dest_x_nome: String,
    v_nf: String,
    v_desc: String,
    v_prod_total: String,
    v_tot_trib: String,
    v_troco: String,
    payments: Vec<(String, String)>,
    inf_cpl: String,
    items: Vec<NfceItem>,
    qr_code_url: String,
    url_chave: String,
    qr_side: bool,
    paper_width: u8,
    paper_dots: u32,
}

// ── Builder do cupom ──────────────────────────────────────────────────────────

fn build_receipt(p: BuildParams) -> Result<Vec<u8>> {
    let cols: usize = if p.paper_width >= 80 { 48 } else { 32 };

    let mut b = EscPosBuilder::new()
        .paper_width(p.paper_width)
        .printable_dots(p.paper_dots)
        .line_spacing(SPACING_NORMAL);

    // ── Homologação ───────────────────────────────────────────────────────────
    if p.tp_amb == "2" {
        b = b
            .align_center()
            .bold(true)
            .text("AMBIENTE DE HOMOLOGAÇÃO\n")
            .text("SEM VALOR FISCAL\n")
            .bold(false)
            .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL);
    }

    // ── Título ────────────────────────────────────────────────────────────────
    b = b
        .align_center()
        .text("Documento Auxiliar da NFC-e\n")
        .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL);

    // ── Emitente ──────────────────────────────────────────────────────────────
    let emit_name = if p.emit_x_nome.trim().is_empty() {
        "EMITENTE NAO INFORMADO".to_string()
    } else {
        p.emit_x_nome.clone()
    };
    b = b.bold(true);
    for line in wrap_text(&emit_name, cols) {
        b = b.text(format!("{line}\n"));
    }
    b = b.bold(false);

    let ie_label = if p.emit_ie.is_empty() { "Isento".to_string() } else { p.emit_ie.clone() };
    b = b.text(format!(
        "CNPJ: {}  IE: {}  {}\n",
        format_cnpj_cpf(&p.emit_cnpj),
        ie_label,
        p.emit_uf
    ));

    if !p.emit_x_lgr.is_empty() {
        let mut addr = p.emit_x_lgr.clone();
        if !p.emit_nro.is_empty() {
            addr.push_str(&format!(", {}", p.emit_nro));
        }
        if !p.emit_x_bairro.is_empty() {
            addr.push_str(&format!(" - {}", p.emit_x_bairro));
        }
        for line in wrap_text(&addr, cols) {
            b = b.text(format!("{line}\n"));
        }
        if !p.emit_x_mun.is_empty() {
            b = b.text(format!("{}/{}\n", p.emit_x_mun, p.emit_uf));
        }
    }

    // ── Cabeçalho dos itens ───────────────────────────────────────────────────
    b = b
        .align_left()
        .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
        .bold(true)
        .text(pad_lr("# DESCRIÇÃO", "Qtd  UN  VlUnit     Total", cols))
        .bold(false)
        .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL);

    // ── Itens ─────────────────────────────────────────────────────────────────
    for item in &p.items {
        let desc = truncate_str(&format!("{}  {}", item.n_item, item.x_prod), cols);
        b = b.text(format!("{desc}\n"));

        let left = format!(
            "   {}  {}  R$ {}",
            format_decimal_br(&item.q_com),
            item.u_com,
            format_brl(&item.v_un_com),
        );
        b = b.text(pad_lr(&left, &format!("R$ {}", format_brl(&item.v_prod)), cols));
    }

    // ── Totais ────────────────────────────────────────────────────────────────
    b = b
        .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
        .text(format!("Qtd. Itens: {}\n", p.items.len()));

    let v_desc_f: f64 = p.v_desc.replace(',', ".").parse().unwrap_or(0.0);
    if v_desc_f > 0.0 {
        b = b
            .text(pad_lr("Subtotal:", &format!("R$ {}", format_brl(&p.v_prod_total)), cols))
            .text(pad_lr("Desconto:", &format!("- R$ {}", format_brl(&p.v_desc)), cols));
    }

    b = b
        .bold(true)
        .text(pad_lr("TOTAL", &format!("R$ {}", format_brl(&p.v_nf)), cols))
        .bold(false);

    // ── Pagamentos ────────────────────────────────────────────────────────────
    b = b
        .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
        .bold(true)
        .text(pad_lr("FORMA DE PAGAMENTO", "VALOR", cols))
        .bold(false);

    for (t_pag, v_pag) in &p.payments {
        b = b.text(pad_lr(pag_type_name(t_pag), &format!("R$ {}", format_brl(v_pag)), cols));
    }

    // Troco — exibido sempre (espelho do PDF)
    b = b.text(pad_lr("Troco:", &format!("R$ {}", format_brl(&p.v_troco)), cols));

    // ── Chave de acesso + código de barras ────────────────────────────────────
    b = b
        .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
        .align_center()
        .text("Consulte pela Chave de Acesso em\n");
    if !p.url_chave.is_empty() {
        b = b.text(format!("{}\n", p.url_chave));
    }
    // Font B (~64 cols em 80 mm) para a chave caber em 1 linha
    let cols_b: usize = if p.paper_width >= 80 { 64 } else { 42 };
    b = b.text("\n").font_b(true);
    for chunk in wrap_text(&format_chave_acesso(&p.chave_acesso), cols_b) {
        b = b.text(format!("{chunk}\n"));
    }
    b = b.font_b(false);
    b = b.barcode_128(&p.chave_acesso);

    // ── QR Code ───────────────────────────────────────────────────────────────
    if p.qr_side {
        // Consumidor (calculado aqui para incluir na imagem combinada)
        let consumer_str_side = if p.dest_cpf_cnpj.is_empty() {
            "CONSUMIDOR NÃO IDENTIFICADO".to_string()
        } else {
            let doc_digits: String =
                p.dest_cpf_cnpj.chars().filter(|c| c.is_ascii_digit()).collect();
            let doc_label = if doc_digits.len() == 14 { "CNPJ" } else { "CPF" };
            if p.dest_x_nome.trim().is_empty() {
                format!("CONSUMIDOR - {}: {}", doc_label, format_cnpj_cpf(&p.dest_cpf_cnpj))
            } else {
                format!("{} - {}: {}", p.dest_x_nome, doc_label, format_cnpj_cpf(&p.dest_cpf_cnpj))
            }
        };

        // Abordagem raster: QR ≈ 55 % de paper_dots, fonte bitmap 8 px/char escala 2× = 16 px/char
        // text_col = paper_dots × 45 % − gap(8px); chars = text_col / 16
        let text_col_px = (p.paper_dots * 45 / 100).saturating_sub(8);
        let text_cols: usize = (text_col_px / 16).max(10) as usize;

        let mut right: Vec<(String, bool)> = Vec::new();
        if !p.n_prot.is_empty() {
            right.push(("Protocolo:".to_string(), false));
            right.push((p.n_prot.clone(), false));
        }
        right.push(("NFC-e Serie/Num:".to_string(), false));
        right.push((format!("{} / {:>09}", p.serie, p.n_nf), false));
        right.push(("Data emissao:".to_string(), false));
        let (d, t) = split_datetime(&p.dh_emi);
        right.push((d, false));
        right.push((t, false));
        if !p.dest_x_nome.trim().is_empty() {
            right.push(("Cliente".to_string(), true));
            for line in wrap_text(&p.dest_x_nome, text_cols) {
                right.push((line, false));
            }
        } else {
            for line in wrap_text(&consumer_str_side, text_cols) {
                right.push((line, false));
            }
        }

        b = b.qr_with_text_right(&p.qr_code_url, &right);
    } else {
        b = b.align_center().qr_code(&p.qr_code_url, 5);
        b = b.line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL);
        if !p.n_prot.is_empty() {
            b = b
                .bold(true)
                .text("PROTOCOLO DE AUTORIZAÇÃO\n")
                .bold(false)
                .text(format!("{} - {}\n", p.n_prot, format_datetime(&p.dh_recbto)));
        }
        b = b
            .bold(true).text("NFC-e Série/Núm: ").bold(false)
            .text(format!("{} / {:>09}\n", p.serie, p.n_nf))
            .bold(true).text("Data emissão: ").bold(false)
            .text(format!("{}\n", format_datetime(&p.dh_emi)))
            .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL);
    }

    // ── Consumidor (apenas no layout centralizado; no qr_side já está na imagem) ──
    if !p.qr_side {
        let consumer_str = if p.dest_cpf_cnpj.is_empty() {
            "CONSUMIDOR NÃO IDENTIFICADO".to_string()
        } else {
            let doc_digits: String =
                p.dest_cpf_cnpj.chars().filter(|c| c.is_ascii_digit()).collect();
            let doc_label = if doc_digits.len() == 14 { "CNPJ" } else { "CPF" };
            if p.dest_x_nome.trim().is_empty() {
                format!("CONSUMIDOR - {}: {}", doc_label, format_cnpj_cpf(&p.dest_cpf_cnpj))
            } else {
                format!("{} - {}: {}", p.dest_x_nome, doc_label, format_cnpj_cpf(&p.dest_cpf_cnpj))
            }
        };
        b = b.align_center();
        for line in wrap_text(&consumer_str, cols) {
            b = b.text(format!("{line}\n"));
        }
    } // if !p.qr_side

    // ── Informações adicionais ────────────────────────────────────────────────
    if !p.inf_cpl.is_empty() {
        b = b
            .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
            .align_left();
        for line in wrap_text(&p.inf_cpl, cols) {
            b = b.text(format!("{line}\n"));
        }
    }

    // ── Tributos aproximados ──────────────────────────────────────────────────
    let v_tot_trib_f: f64 = p.v_tot_trib.replace(',', ".").parse().unwrap_or(0.0);
    if v_tot_trib_f > 0.0 {
        b = b
            .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
            .align_center()
            .text(format!(
                "Valor Aprox. dos Tributos R$ {} (IBPT)\n",
                format_brl(&p.v_tot_trib)
            ));
    }

    // ── Créditos ─────────────────────────────────────────────────────────────
    b = b
        .line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
        .align_center()
        .text("Gerado por dfe - crates.io/crates/dfe\n");

    b = b.line_spacing_default().feed(6).cut();
    Ok(b.build())
}

// ── Impressão Windows ─────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn raw_print_windows(printer_name: &str, data: &[u8]) -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Graphics::Printing::{
        ClosePrinter, DOC_INFO_1W, EndDocPrinter, EndPagePrinter, OpenPrinterW,
        PRINTER_HANDLE, StartDocPrinterW, StartPagePrinter, WritePrinter,
    };

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let printer_wide = to_wide(printer_name);
    let doc_name_wide = to_wide("NFC-e");
    let datatype_wide = to_wide("RAW");

    unsafe {
        let mut handle = PRINTER_HANDLE { Value: std::ptr::null_mut() };
        if OpenPrinterW(printer_wide.as_ptr(), &mut handle, std::ptr::null()) == 0 {
            return Err(DfeError::Configuracao(format!(
                "Falha ao abrir impressora '{printer_name}'"
            )));
        }
        let doc_info = DOC_INFO_1W {
            pDocName: doc_name_wide.as_ptr() as *mut u16,
            pOutputFile: std::ptr::null_mut(),
            pDatatype: datatype_wide.as_ptr() as *mut u16,
        };
        if StartDocPrinterW(handle, 1, &doc_info) == 0 {
            ClosePrinter(handle);
            return Err(DfeError::Configuracao("Falha ao iniciar job de impressão".to_string()));
        }
        if StartPagePrinter(handle) == 0 {
            EndDocPrinter(handle);
            ClosePrinter(handle);
            return Err(DfeError::Configuracao("Falha ao iniciar página".to_string()));
        }
        let mut written: u32 = 0;
        let ok = WritePrinter(handle, data.as_ptr() as *const _, data.len() as u32, &mut written);
        EndPagePrinter(handle);
        EndDocPrinter(handle);
        ClosePrinter(handle);
        if ok == 0 {
            return Err(DfeError::Configuracao("Falha ao escrever na impressora".to_string()));
        }
    }
    Ok(())
}

// ── Helpers de layout ─────────────────────────────────────────────────────────

fn pad_lr(left: &str, right: &str, cols: usize) -> String {
    let ll = left.chars().count();
    let rl = right.chars().count();
    if ll + rl >= cols {
        return format!("{left}\n{right}\n");
    }
    format!("{left}{}{right}\n", " ".repeat(cols - ll - rl))
}

// ── Helpers de formatação ─────────────────────────────────────────────────────

fn format_brl(value: &str) -> String {
    let v: f64 = value.replace(',', ".").parse().unwrap_or(0.0);
    let s = format!("{:.2}", v);
    let parts: Vec<&str> = s.split('.').collect();
    let digits: Vec<char> = parts[0].chars().collect();
    let with_dots: String =
        digits.iter().rev().enumerate().fold(String::new(), |mut acc, (i, &c)| {
            if i > 0 && i % 3 == 0 { acc.insert(0, '.'); }
            acc.insert(0, c);
            acc
        });
    format!("{},{}", with_dots, parts[1])
}

fn format_cnpj_cpf(doc: &str) -> String {
    let d: String = doc.chars().filter(|c| c.is_ascii_digit()).collect();
    match d.len() {
        14 => format!("{}.{}.{}/{}-{}", &d[0..2], &d[2..5], &d[5..8], &d[8..12], &d[12..14]),
        11 => format!("{}.{}.{}-{}", &d[0..3], &d[3..6], &d[6..9], &d[9..11]),
        _ => doc.to_string(),
    }
}

fn format_decimal_br(value: &str) -> String {
    value.replace('.', ",")
}

fn format_datetime(dt: &str) -> String {
    if dt.len() >= 19 {
        let parts: Vec<&str> = dt[..10].split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{} {}", parts[2], parts[1], parts[0], &dt[11..19]);
        }
    }
    dt.to_string()
}

fn split_datetime(dt: &str) -> (String, String) {
    if dt.len() >= 19 {
        let parts: Vec<&str> = dt[..10].split('-').collect();
        if parts.len() == 3 {
            let date = format!("{}/{}/{}", parts[2], parts[1], parts[0]);
            let time = dt[11..19].to_string();
            return (date, time);
        }
    }
    (dt.to_string(), String::new())
}

fn format_chave_acesso(chave: &str) -> String {
    chave.chars().collect::<Vec<_>>().chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.chars().count() + 1 + word.chars().count() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() { lines.push(current); }
    lines
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars { return s.to_string(); }
    format!("{}...", chars[..max_chars.saturating_sub(3)].iter().collect::<String>())
}

fn pag_type_name(t_pag: &str) -> &'static str {
    match t_pag {
        "01" => "Dinheiro",        "02" => "Cheque",
        "03" => "Cartao de Credito","04" => "Cartao de Debito",
        "05" => "Credito Loja",    "10" => "Vale Alimentacao",
        "11" => "Vale Refeicao",   "12" => "Vale Presente",
        "13" => "Vale Combustivel","14" => "Duplicata Mercantil",
        "15" => "Boleto Bancario", "16" => "Deposito Bancario",
        "17" => "PIX",             "18" => "Transferencia bancaria",
        "19" => "Programa fidelidade","20" => "PIX Estatico",
        "21" => "Credito em Loja", "90" => "Sem Pagamento",
        "91" => "Pagamento Posterior", _ => "Outros",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_brl_correct() {
        assert_eq!(format_brl("1234.56"), "1.234,56");
        assert_eq!(format_brl("50.00"), "50,00");
        assert_eq!(format_brl("0"), "0,00");
    }

    #[test]
    fn format_cnpj_cpf_correct() {
        assert_eq!(format_cnpj_cpf("11222333000181"), "11.222.333/0001-81");
        assert_eq!(format_cnpj_cpf("12345678901"), "123.456.789-01");
    }

    #[test]
    fn format_chave_acesso_groups() {
        let chave = "35241201234567890001650010000000011234567890";
        let fmt = format_chave_acesso(chave);
        assert!(fmt.contains(' '));
        assert_eq!(fmt.chars().filter(|&c| c == ' ').count(), 10);
    }

    #[test]
    fn pad_lr_correct_width() {
        let line = pad_lr("TOTAL", "R$ 100,00", 48);
        assert_eq!(line.trim_end_matches('\n').chars().count(), 48);
    }

    #[test]
    fn pad_lr_overflow_no_panic() {
        let line = pad_lr("TEXTO MUITO LONGO QUE EXCEDE", "VALOR TAMBEM LONGO", 20);
        assert!(!line.is_empty());
    }

    #[test]
    fn builder_rejects_model_55() {
        let xml = r#"<nfeProc><NFe><infNFe Id="NFe35000000000000000000550010000000011234567890"><ide><mod>55</mod></ide><emit><CNPJ>00000000000000</CNPJ><enderEmit/></emit><det/><total/><transp/><pag/><infAdic/></infNFe></NFe><protNFe/></nfeProc>"#;
        let result = EscPosNFCeBuilder::new().xml(xml).build();
        assert!(result.is_err());
    }
}
