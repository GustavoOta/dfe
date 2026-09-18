use crate::error::{DfeError, Result};
use crate::interno::cnpj_cpf::sanitize_cnpj;
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
    cols: Option<usize>,
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
            cols: None,
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

    /// Sobrescreve o número de colunas de texto.
    ///
    /// Padrão: 48 para papel 80 mm · 32 para 58 mm.
    /// Use quando a impressora tem área imprimível menor que o padrão
    /// (ex.: Bematech MP-4200 TH → 42 colunas).
    pub fn columns(mut self, cols: usize) -> Self {
        self.cols = Some(cols);
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
        let emit_x_nome = clamp_chars(
            emit.x_fant
                .as_deref()
                .filter(|v| !v.trim().is_empty())
                .or_else(|| emit.x_nome.as_deref().filter(|v| !v.trim().is_empty()))
                .unwrap_or_default()
                .to_string(),
            XNOME_MAX_CHARS,
        );
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
        let tp_emis = ide.tp_emis.clone().unwrap_or_default();

        let dest = &inf.dest;
        let dest_cpf_cnpj = dest
            .as_ref()
            .and_then(|d| d.cnpj.clone().or_else(|| d.cpf.clone()))
            .unwrap_or_default();
        let dest_x_nome = clamp_chars(
            dest.as_ref().and_then(|d| d.x_nome.clone()).unwrap_or_default(),
            XNOME_MAX_CHARS,
        );

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
            tp_emis,
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
            cols: self.cols,
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
    tp_emis: String,
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
    cols: Option<usize>,
}

// ── Builder do cupom ──────────────────────────────────────────────────────────

fn build_receipt(p: BuildParams) -> Result<Vec<u8>> {
    let cols: usize = p.cols.unwrap_or_else(|| if p.paper_width >= 80 { 48 } else { 32 });

    let mut b = EscPosBuilder::new()
        .paper_width(p.paper_width)
        .printable_dots(p.paper_dots)
        .line_spacing(SPACING_NORMAL);
    if let Some(c) = p.cols {
        b = b.columns(c);
    }

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

    // ── Contingência off-line (tpEmis=9) ──────────────────────────────────────
    if p.tp_emis == "9" {
        b = b
            .align_center()
            .bold(true)
            .text("NFC-e EMITIDA EM CONTINGÊNCIA\n")
            .text("PENDENTE DE AUTORIZAÇÃO PELA SEFAZ\n")
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
        // Razão social e documento vão para a faixa abaixo do QR, onde a largura inteira
        // do papel os acomoda no corpo nominal — ao lado do código eles quebrariam em
        // quatro linhas ou teriam de encolher. Sem destinatário não se gasta linha
        // nenhuma embaixo: o aviso fica na própria coluna lateral.
        let below = linhas_destinatario_abaixo(&p.dest_x_nome, &p.dest_cpf_cnpj);
        if below.is_empty() {
            right.push((CONSUMIDOR_NAO_IDENTIFICADO.to_string(), false));
        }

        b = b.qr_with_text_right_and_below(&p.qr_code_url, &right, &below);
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
        let consumer_str = linha_consumidor(&p.dest_x_nome, &p.dest_cpf_cnpj)
            .unwrap_or_else(|| CONSUMIDOR_NAO_IDENTIFICADO.to_string());
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

pub(super) fn pad_lr(left: &str, right: &str, cols: usize) -> String {
    let ll = left.chars().count();
    let rl = right.chars().count();
    if ll + rl >= cols {
        return format!("{left}\n{right}\n");
    }
    format!("{left}{}{right}\n", " ".repeat(cols - ll - rl))
}

// ── Helpers de formatação ─────────────────────────────────────────────────────

pub(super) fn format_brl(value: &str) -> String {
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

pub(super) fn format_cnpj_cpf(doc: &str) -> String {
    crate::interno::cnpj_cpf::format_cnpj_cpf(doc)
}

/// Rótulo do documento do consumidor: "CNPJ" para 14 posições alfanuméricas
/// (preserva letras do CNPJ alfanumérico), "CPF" caso contrário (11 dígitos).
pub(super) fn doc_label_cnpj_cpf(doc: &str) -> &'static str {
    if sanitize_cnpj(doc).len() == 14 { "CNPJ" } else { "CPF" }
}

pub(super) fn format_decimal_br(value: &str) -> String {
    value.replace('.', ",")
}

pub(super) fn format_datetime(dt: &str) -> String {
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

pub(super) fn format_chave_acesso(chave: &str) -> String {
    chave.chars().collect::<Vec<_>>().chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Quebra `text` em linhas de no máximo `max_chars` caracteres, partindo a palavra que
/// sozinha não couber (ver [`wrap_hard`](super::wrap_hard)).
///
/// Texto vazio rende lista vazia — os chamadores emitem uma linha por elemento e não
/// devem ganhar linha em branco por um campo ausente.
pub(super) fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    super::wrap_hard(text, max_chars)
}

/// Teto de `xNome` no leiaute da NF-e/NFC-e: `leiauteNFe_v4.00.xsd` restringe
/// `emit/xNome` e `dest/xNome` a `maxLength 60`.
///
/// Vale igualmente para os modelos **55 e 65** — eles compartilham o mesmo leiaute, em
/// que o modelo é apenas o campo `ide/mod`. A SEFAZ rejeita por schema antes de
/// autorizar, e o cupom só é montado a partir de um `nfeProc` (nota já autorizada):
/// 60 é teto duro, não estimativa. Com ele o bloco do consumidor tem cota de pior caso
/// conhecida — `ceil(60 / colunas)` linhas — e o layout lateral nunca precisa truncar.
pub(super) const XNOME_MAX_CHARS: usize = 60;

/// Texto exibido quando a NFC-e sai sem destinatário identificado.
pub(super) const CONSUMIDOR_NAO_IDENTIFICADO: &str = "CONSUMIDOR NÃO IDENTIFICADO";

/// Título da faixa do destinatário, abaixo do QR Code.
///
/// Sem acento de propósito: a faixa é desenhada com a font8x8, cuja cobertura de
/// maiúscula acentuada rende glifo de caixa baixa — os outros rótulos do bloco
/// (`Data emissao:`) seguem a mesma regra.
pub(super) const TITULO_DESTINATARIO: &str = "Destinatario:";

/// Faixa do destinatário que vai **abaixo** do QR Code no layout lateral.
///
/// Vazia quando a nota não traz destinatário — aí não se gasta linha nenhuma embaixo do
/// código, e quem chama imprime [`CONSUMIDOR_NAO_IDENTIFICADO`] na coluna lateral.
///
/// Cada item é `(texto, negrito)`; o título sai em negrito, o conteúdo em corpo normal.
pub(super) fn linhas_destinatario_abaixo(
    dest_x_nome: &str,
    dest_cpf_cnpj: &str,
) -> Vec<(String, bool)> {
    match linha_consumidor(dest_x_nome, dest_cpf_cnpj) {
        Some(consumidor) => vec![
            (TITULO_DESTINATARIO.to_string(), true),
            (consumidor, false),
        ],
        None => Vec::new(),
    }
}

/// Linha do consumidor no cupom — a mesma nos dois layouts, centralizado e lateral.
///
/// Devolve `None` quando a nota não traz destinatário; aí o cupom imprime
/// [`CONSUMIDOR_NAO_IDENTIFICADO`].
///
/// Emite nome **e** documento. Os dois ramos antigos perdiam informação: o lateral só
/// montava o documento quando não havia nome, e o centralizado descartava o nome quando
/// não havia documento.
pub(super) fn linha_consumidor(dest_x_nome: &str, dest_cpf_cnpj: &str) -> Option<String> {
    let nome = dest_x_nome.trim();
    let doc = dest_cpf_cnpj.trim();

    match (nome.is_empty(), doc.is_empty()) {
        (true, true) => None,
        (false, true) => Some(nome.to_string()),
        (true, false) => Some(format!(
            "CONSUMIDOR - {}: {}",
            doc_label_cnpj_cpf(doc),
            format_cnpj_cpf(doc)
        )),
        (false, false) => Some(format!(
            "{} - {}: {}",
            nome,
            doc_label_cnpj_cpf(doc),
            format_cnpj_cpf(doc)
        )),
    }
}

/// Corta `s` no teto de caracteres, sem reticências.
///
/// Rede de segurança para XML fora do schema: nota autorizada não chega aqui acima do
/// teto, então na prática isto nunca corta nada — só garante a cota de pior caso.
fn clamp_chars(s: String, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s;
    }
    s.chars().take(max_chars).collect()
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars { return s.to_string(); }
    format!("{}...", chars[..max_chars.saturating_sub(3)].iter().collect::<String>())
}

pub(super) fn pag_type_name(t_pag: &str) -> &'static str {
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
    fn doc_label_distingue_cpf_cnpj_inclusive_alfanumerico() {
        // CNPJ numérico (14) e CPF (11) — com e sem máscara.
        assert_eq!(doc_label_cnpj_cpf("11222333000181"), "CNPJ");
        assert_eq!(doc_label_cnpj_cpf("11.222.333/0001-81"), "CNPJ");
        assert_eq!(doc_label_cnpj_cpf("12345678901"), "CPF");
        // CNPJ alfanumérico (14 posições com letras) deve continuar sendo CNPJ.
        assert_eq!(doc_label_cnpj_cpf("12ABC34501DE35"), "CNPJ");
        assert_eq!(doc_label_cnpj_cpf("12.ABC.345/01DE-35"), "CNPJ");
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

    fn base_params(tp_emis: &str) -> BuildParams {
        BuildParams {
            chave_acesso: "35000000000000000000650010000000001000000001".to_string(),
            n_prot: String::new(),
            dh_recbto: String::new(),
            emit_x_nome: "EMPRESA TESTE".to_string(),
            emit_cnpj: "11222333000181".to_string(),
            emit_ie: "123456789".to_string(),
            emit_uf: "SP".to_string(),
            emit_x_lgr: "RUA TESTE".to_string(),
            emit_nro: "100".to_string(),
            emit_x_bairro: "CENTRO".to_string(),
            emit_x_mun: "SAO PAULO".to_string(),
            tp_amb: "1".to_string(),
            tp_emis: tp_emis.to_string(),
            serie: "1".to_string(),
            n_nf: "1".to_string(),
            dh_emi: "2026-07-08T10:00:00-03:00".to_string(),
            dest_cpf_cnpj: String::new(),
            dest_x_nome: String::new(),
            v_nf: "10.00".to_string(),
            v_desc: "0.00".to_string(),
            v_prod_total: "10.00".to_string(),
            v_tot_trib: "0.00".to_string(),
            v_troco: "0.00".to_string(),
            payments: vec![],
            inf_cpl: String::new(),
            items: vec![],
            qr_code_url: "https://exemplo.teste/qrcode?p=x".to_string(),
            url_chave: String::new(),
            qr_side: false,
            paper_width: 80,
            paper_dots: 576,
            cols: None,
        }
    }

    #[test]
    fn consumidor_traz_nome_e_documento_juntos() {
        // Regressão: o layout lateral só montava o documento quando o nome estava vazio,
        // então cliente identificado por razão social saía do cupom sem CPF/CNPJ.
        assert_eq!(
            linha_consumidor("Magazine Campos Eireli", "11222333000181").unwrap(),
            "Magazine Campos Eireli - CNPJ: 11.222.333/0001-81"
        );
        assert_eq!(
            linha_consumidor("FULANO DE TAL", "12345678901").unwrap(),
            "FULANO DE TAL - CPF: 123.456.789-01"
        );
    }

    #[test]
    fn consumidor_cobre_os_quatro_casos() {
        // Sem nada: não há linha de consumidor; quem chama imprime o aviso.
        assert_eq!(linha_consumidor("", ""), None);
        assert_eq!(linha_consumidor("  ", " "), None);

        // Só documento: prefixo CONSUMIDOR, como no layout centralizado de sempre.
        assert_eq!(
            linha_consumidor("   ", "12345678901").unwrap(),
            "CONSUMIDOR - CPF: 123.456.789-01"
        );

        // Regressão: só nome, sem documento, era descartado pelo layout centralizado.
        assert_eq!(linha_consumidor("LOJA X", "").unwrap(), "LOJA X");
    }

    #[test]
    fn consumidor_entrega_o_campo_inteiro_sem_pre_quebra() {
        // A quebra é do builder, que conhece a largura real. O nome de 60 caracteres
        // com o documento tem de chegar inteiro, numa string só.
        let nome = "COMERCIO E DISTRIBUICAO DE ALIMENTOS DO VALE DO RIBEIRA LTDA";
        assert_eq!(nome.chars().count(), XNOME_MAX_CHARS);

        let linha = linha_consumidor(nome, "11222333000181").unwrap();
        assert!(linha.starts_with(nome));
        assert!(linha.ends_with("CNPJ: 11.222.333/0001-81"));
    }

    #[test]
    fn faixa_do_destinatario_tem_titulo_em_negrito() {
        let faixa = linhas_destinatario_abaixo("LOJA X", "11222333000181");
        assert_eq!(faixa.len(), 2);
        assert_eq!(faixa[0], ("Destinatario:".to_string(), true));
        assert_eq!(faixa[1].0, "LOJA X - CNPJ: 11.222.333/0001-81");
        assert!(!faixa[1].1, "o conteúdo sai em corpo normal");

        // Sem destinatário não há faixa — nem o título.
        assert!(linhas_destinatario_abaixo("", "").is_empty());
        assert!(linhas_destinatario_abaixo("  ", " ").is_empty());
    }

    #[test]
    fn sem_destinatario_o_qr_side_nao_gasta_linha_abaixo_do_codigo() {
        // O aviso fica na coluna lateral; a faixa de baixo nem existe, então o cupom
        // anônimo não pode ficar mais alto que o mínimo do QR.
        let mut anonimo = base_params("1");
        anonimo.qr_side = true;
        let mut identificado = base_params("1");
        identificado.qr_side = true;
        identificado.dest_x_nome = "LOJA X".to_string();
        identificado.dest_cpf_cnpj = "22333444000195".to_string();

        let a = build_receipt(anonimo).unwrap();
        let b = build_receipt(identificado).unwrap();

        // Identificar o cliente acrescenta a faixa inferior — logo, mais bytes de raster.
        assert!(b.len() > a.len(), "a faixa de baixo não apareceu");
    }

    #[test]
    fn clamp_chars_garante_a_cota_mesmo_com_xml_fora_do_schema() {
        // Nota autorizada nunca passa de 60 — mas XML torto não pode furar a cota.
        let torto = "X".repeat(120);
        assert_eq!(clamp_chars(torto, XNOME_MAX_CHARS).chars().count(), XNOME_MAX_CHARS);
        // Dentro do teto, nada muda.
        assert_eq!(clamp_chars("LOJA X".to_string(), XNOME_MAX_CHARS), "LOJA X");
    }

    #[test]
    fn qr_side_com_cliente_identificado_desenha_o_bloco_e_nao_vaza_texto() {
        // O bloco lateral vira pixel dentro da faixa raster: nome e documento não podem
        // sobrar como texto ESC/POS solto. E identificar o cliente tem de mudar a imagem
        // — era justamente o que não acontecia quando o documento era descartado.
        let mut anonimo = base_params("1");
        anonimo.qr_side = true;

        let mut identificado = base_params("1");
        identificado.qr_side = true;
        identificado.dest_x_nome =
            "COMERCIO E DISTRIBUICAO DE ALIMENTOS DO VALE DO RIBEIRA LTDA".to_string();
        // CNPJ diferente do emitente (base_params), senão o cabeçalho do cupom
        // — que imprime o do emitente como texto — falsearia o teste de vazamento.
        identificado.dest_cpf_cnpj = "22333444000195".to_string();

        // Mesmos dados, só o documento entra — a imagem tem de mudar mesmo assim.
        let mut so_nome = base_params("1");
        so_nome.qr_side = true;
        so_nome.dest_x_nome.clone_from(&identificado.dest_x_nome);

        let a = build_receipt(anonimo).unwrap();
        let b = build_receipt(identificado).unwrap();
        let c = build_receipt(so_nome).unwrap();

        assert_ne!(a, b);
        assert_ne!(c, b, "o CPF/CNPJ do cliente precisa aparecer no cupom");

        let texto = String::from_utf8_lossy(&b);
        assert!(!texto.contains("RIBEIRA LTDA"));
        assert!(!texto.contains("22.333.444/0001-95"));
    }

    #[test]
    fn qr_side_cabe_em_58_mm_sem_cortar_rotulo_nem_protocolo() {
        // Em 58 mm a coluna lateral tem 10 a 12 caracteres: "NFC-e Serie/Num:" (16) e o
        // protocolo (15) eram cortados em silêncio por não passarem pela quebra.
        let mut p = base_params("1");
        p.qr_side = true;
        p.paper_width = 58;
        p.paper_dots = 384;
        p.n_prot = "135266376727096".to_string();
        p.qr_code_url = "https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx?p=35260924341498000114650010000103919114792106|2|1|1|a3f1c2d4e5b60718293a4b5c6d7e8f9012345678".to_string();

        let cols = EscPosBuilder::new().paper_width(58).qr_text_cols(&p.qr_code_url);
        assert!((10..=12).contains(&cols), "coluna de 58 mm deu {cols}");

        // Rótulo e protocolo agora cabem porque são quebrados, não cortados.
        assert!(super::super::wrap_hard("NFC-e Serie/Num:", cols)
            .iter()
            .all(|l| l.chars().count() <= cols));
        assert!(super::super::wrap_hard(&p.n_prot, cols).join("") == p.n_prot);

        assert!(!build_receipt(p).unwrap().is_empty());
    }

    #[test]
    fn contingencia_tp_emis_9_gera_cupom_maior_com_aviso() {
        // Mesmos dados, só tp_emis muda — o cupom de contingência deve crescer (banner extra),
        // sem quebrar a geração normal (tp_emis=1).
        let normal = build_receipt(base_params("1")).unwrap();
        let contingencia = build_receipt(base_params("9")).unwrap();
        assert!(!normal.is_empty());
        assert!(contingencia.len() > normal.len());
    }
}
