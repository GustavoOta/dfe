use crate::error::{DfeError, Result};
use crate::xml_extractor::{XmlExtractor, XmlExtractorSignature};

use super::EscPosBuilder;

/// Builder fluente para impressão de **NFC-e** (modelo 65) em impressoras térmicas via ESC/POS.
///
/// Lê um XML `nfeProc` autorizado e gera os bytes prontos para envio à impressora.
/// Suporta papéis de 80 mm e 58 mm e dois layouts de QR Code.
///
/// # Exemplo
///
/// ```no_run
/// use dfe::EscPosNFCeBuilder;
///
/// # fn example() -> Result<(), dfe::DfeError> {
/// let bytes = EscPosNFCeBuilder::new()
///     .xml("./nota_nfce.xml")
///     .paper_width(80)
///     .build()?;
///
/// std::fs::write("\\\\.\\COM3", &bytes).unwrap();
/// # Ok(())
/// # }
/// ```
pub struct EscPosNFCeBuilder {
    xml: Option<String>,
    qr_side: bool,
    paper_width: u8,
}

impl EscPosNFCeBuilder {
    /// Cria um builder com QR Code centralizado e papel de 80 mm.
    pub fn new() -> Self {
        Self { xml: None, qr_side: false, paper_width: 80 }
    }

    /// String XML do `nfeProc` ou caminho de arquivo terminado em `".xml"`.
    pub fn xml(mut self, src: impl Into<String>) -> Self {
        self.xml = Some(src.into());
        self
    }

    /// QR Code alinhado à esquerda (compacto). Padrão: centralizado.
    pub fn qr_side(mut self) -> Self {
        self.qr_side = true;
        self
    }

    /// Largura do papel em milímetros. Use `80` ou `58`. Padrão: `80`.
    pub fn paper_width(mut self, mm: u8) -> Self {
        self.paper_width = mm;
        self
    }

    /// Lê o XML, extrai os dados da NFC-e e gera os bytes ESC/POS.
    ///
    /// # Erros
    ///
    /// Retorna [`DfeError`](crate::DfeError) se o XML for inválido, o arquivo não existir
    /// ou o documento não for NFC-e (modelo 65).
    pub fn build(self) -> Result<Vec<u8>> {
        let src = self.xml.ok_or_else(|| DfeError::Configuracao("XML não informado".to_string()))?;

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

        // Chave de acesso (sem prefixo "NFe")
        let chave_acesso = inf
            .id
            .as_deref()
            .unwrap_or("")
            .strip_prefix("NFe")
            .unwrap_or(inf.id.as_deref().unwrap_or(""))
            .to_string();

        // Protocolo
        let prot = &nfe_proc.prot_nfe.inf_prot;
        let n_prot = prot.as_ref().and_then(|p| p.n_prot.clone()).unwrap_or_default();
        let dh_recbto = prot.as_ref().and_then(|p| p.dh_recbto.clone()).unwrap_or_default();

        // Emitente
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

        // Ide
        let ide = &inf.ide;
        let tp_amb = ide.tp_amb.clone().unwrap_or_default();
        let serie = ide.serie.clone().unwrap_or_default();
        let n_nf = ide.n_nf.clone().unwrap_or_default();
        let dh_emi = ide.dh_emi.clone().unwrap_or_default();

        // Destinatário
        let dest = &inf.dest;
        let dest_cpf_cnpj = dest
            .as_ref()
            .and_then(|d| d.cnpj.clone().or_else(|| d.cpf.clone()))
            .unwrap_or_default();
        let dest_x_nome = dest.as_ref().and_then(|d| d.x_nome.clone()).unwrap_or_default();

        // Totais
        let icms_tot = inf.total.icms_tot.as_ref();
        let v_nf = icms_tot.and_then(|t| t.v_nf.clone()).unwrap_or_default();
        let v_desc = icms_tot.and_then(|t| t.v_desc.clone()).unwrap_or_default();
        let v_prod_total = icms_tot.and_then(|t| t.v_prod.clone()).unwrap_or_default();
        // Soma v_tot_trib de cada item; fallback para ICMSTot (XML pode tê-lo zerado)
        let v_tot_trib_items: f64 = inf.det.iter()
            .filter_map(|det| det.imposto.v_tot_trib.as_deref())
            .filter_map(|s| s.parse::<f64>().ok())
            .sum();
        let v_tot_trib = if v_tot_trib_items > 0.0 {
            format!("{:.2}", v_tot_trib_items)
        } else {
            icms_tot.and_then(|t| t.v_tot_trib.clone()).unwrap_or_default()
        };
        let v_troco = inf.pag.v_troco.clone().unwrap_or_default();

        // Pagamentos
        let payments: Vec<(String, String)> = inf
            .pag
            .det_pag
            .iter()
            .map(|d| (d.t_pag.clone().unwrap_or_default(), d.v_pag.clone().unwrap_or_default()))
            .collect();

        // Informações adicionais
        let inf_cpl = inf.inf_adic.inf_cpl.clone().unwrap_or_default();

        // QR Code URL e URL de consulta
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

        // Itens
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

        build_receipt(BuildParams {
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
}

fn build_receipt(p: BuildParams) -> Result<Vec<u8>> {
    let cols: usize = if p.paper_width >= 80 { 48 } else { 32 };
    let mut b = EscPosBuilder::new().paper_width(p.paper_width);

    // ── Aviso de homologação ──────────────────────────────────────────────────
    if p.tp_amb == "2" {
        b = b
            .align_center()
            .bold(true)
            .text("AMBIENTE DE HOMOLOGACAO\n")
            .text("SEM VALOR FISCAL\n")
            .bold(false);
    }

    // ── Título ────────────────────────────────────────────────────────────────
    b = b.align_center().text("Documento Auxiliar da NFC-e\n").divider();

    // ── Emitente ──────────────────────────────────────────────────────────────
    let emit_name = if p.emit_x_nome.trim().is_empty() {
        "EMITENTE NAO INFORMADO".to_string()
    } else {
        p.emit_x_nome.clone()
    };
    for line in wrap_text(&emit_name, cols) {
        b = b.align_center().text(format!("{line}\n"));
    }

    let ie_label = if p.emit_ie.is_empty() { "Isento".to_string() } else { p.emit_ie.clone() };
    b = b.text(format!(
        "CNPJ: {}  IE: {}  {}\n",
        format_cnpj_cpf(&p.emit_cnpj),
        ie_label,
        p.emit_uf,
    ));

    if !p.emit_x_lgr.is_empty() {
        b = b.text(format!("{}, {} - {}\n", p.emit_x_lgr, p.emit_nro, p.emit_x_bairro));
        b = b.text(format!("{}/{}\n", p.emit_x_mun, p.emit_uf));
    }

    // ── Cabeçalho e itens ─────────────────────────────────────────────────────
    b = b.divider().text(items_header(cols)).divider();

    for item in &p.items {
        let desc = truncate_str(&format!("{} {}", item.n_item, item.x_prod), cols);
        b = b.text(format!("{desc}\n"));

        let left = format!(
            "   {} {}  R$ {}",
            format_decimal_br(&item.q_com),
            item.u_com,
            format_brl(&item.v_un_com),
        );
        let right = format!("R$ {}", format_brl(&item.v_prod));
        b = b.text(pad_lr(&left, &right, cols));
    }

    // ── Totais ────────────────────────────────────────────────────────────────
    b = b.divider().text(format!("Qtd. Itens: {}\n", p.items.len()));

    let v_desc_f: f64 = p.v_desc.replace(',', ".").parse().unwrap_or(0.0);
    if v_desc_f > 0.0 {
        b = b.text(pad_lr("Subtotal", &format!("R$ {}", format_brl(&p.v_prod_total)), cols));
        b = b.text(pad_lr("Desconto", &format!("-R$ {}", format_brl(&p.v_desc)), cols));
    }

    b = b
        .bold(true)
        .text(pad_lr("TOTAL", &format!("R$ {}", format_brl(&p.v_nf)), cols))
        .bold(false);

    // ── Pagamentos ────────────────────────────────────────────────────────────
    b = b.divider().text(pad_lr("FORMA DE PAGAMENTO", "VALOR", cols));
    for (t_pag, v_pag) in &p.payments {
        b = b.text(pad_lr(pag_type_name(t_pag), &format!("R$ {}", format_brl(v_pag)), cols));
    }

    let v_troco_f: f64 = p.v_troco.replace(',', ".").parse().unwrap_or(0.0);
    if v_troco_f > 0.0 {
        b = b.text(pad_lr("Troco", &format!("R$ {}", format_brl(&p.v_troco)), cols));
    }

    // ── Chave de acesso + código de barras (acima do QR) ─────────────────────
    b = b.divider()
        .align_center()
        .text("Consulte pela Chave de Acesso em\n");
    if !p.url_chave.is_empty() {
        b = b.text(format!("{}\n", p.url_chave));
    }
    b = b.bold(true).text("CHAVE DE ACESSO\n").bold(false);
    let chave_fmt = format_chave_acesso(&p.chave_acesso);
    for chunk in wrap_text(&chave_fmt, cols) {
        b = b.text(format!("{chunk}\n"));
    }
    b = b.barcode_128(&p.chave_acesso);

    // ── QR Code ───────────────────────────────────────────────────────────────
    if p.qr_side {
        // QR à esquerda; protocolo + NF-e No/Serie/data ao lado (mesma âncora vertical)
        b = b.align_left().qr_code(&p.qr_code_url, 3);
        if !p.n_prot.is_empty() {
            b = b.text(format!("Protocolo: {}\n", p.n_prot));
            b = b.text(format!("{}\n", format_datetime(&p.dh_recbto)));
        }
        b = b.text(format!("NF-e {:>09} Serie {}\n", p.n_nf, p.serie));
        b = b.text(format!("{}\n", format_datetime(&p.dh_emi)));
        b = b.divider();
    } else {
        // QR centralizado; NF-e No/Serie/data abaixo do protocolo
        b = b.align_center().qr_code(&p.qr_code_url, 5);
        b = b.divider();
        if !p.n_prot.is_empty() {
            b = b
                .text("PROTOCOLO DE AUTORIZACAO\n")
                .text(format!("{} - {}\n", p.n_prot, format_datetime(&p.dh_recbto)));
        }
        b = b.text(format!(
            "NF-e No {:>09}  Serie {}  {}\n",
            p.n_nf, p.serie, format_datetime(&p.dh_emi),
        ));
        b = b.divider();
    }

    // ── Consumidor ────────────────────────────────────────────────────────────
    if !p.dest_cpf_cnpj.is_empty() || !p.dest_x_nome.is_empty() {
        b = b.align_center();
        if p.dest_cpf_cnpj.is_empty() {
            b = b.text("Consumidor nao identificado\n");
        } else {
            b = b.text(format!("CPF/CNPJ: {}\n", format_cnpj_cpf(&p.dest_cpf_cnpj)));
        }
        if !p.dest_x_nome.is_empty() {
            b = b.text(format!("{}\n", p.dest_x_nome));
        }
    }

    // ── Informações adicionais ────────────────────────────────────────────────
    if !p.inf_cpl.is_empty() {
        b = b.divider().align_left().text("INFORMACOES ADICIONAIS\n");
        for line in wrap_text(&p.inf_cpl, cols) {
            b = b.text(format!("{line}\n"));
        }
    }

    // ── Tributos aproximados (rodapé) ─────────────────────────────────────────
    let v_tot_trib_f: f64 = p.v_tot_trib.replace(',', ".").parse().unwrap_or(0.0);
    if v_tot_trib_f > 0.0 {
        b = b
            .divider()
            .align_center()
            .text(format!(
                "Valor Aproximado dos Tributos R$ {} (Fonte: IBPT)\n",
                format_brl(&p.v_tot_trib),
            ));
    }

    b = b.feed(3).cut();
    Ok(b.build())
}

// ── Helpers de layout ─────────────────────────────────────────────────────────

/// Texto à esquerda + valor à direita, com espaços no meio. Termina com \n.
fn pad_lr(left: &str, right: &str, cols: usize) -> String {
    let ll = left.chars().count();
    let rl = right.chars().count();
    if ll + rl >= cols {
        return format!("{left}\n{right}\n");
    }
    let spaces = cols - ll - rl;
    format!("{left}{}{right}\n", " ".repeat(spaces))
}

fn items_header(cols: usize) -> String {
    if cols >= 48 {
        let w = cols - 4;
        format!("{:<3} {:<width$}\n", "#", "DESCRICAO", width = w)
    } else {
        let w = cols - 3;
        format!("{:<2} {:<width$}\n", "#", "DESCRICAO", width = w)
    }
}

// ── Helpers de formatação (espelho de pdf_builder_nfce_80mm) ─────────────────

fn format_brl(value: &str) -> String {
    let v: f64 = value.replace(',', ".").parse().unwrap_or(0.0);
    let formatted = format!("{:.2}", v);
    let parts: Vec<&str> = formatted.split('.').collect();
    let digits: Vec<char> = parts[0].chars().collect();
    let with_dots: String = digits.iter().rev().enumerate().fold(String::new(), |mut acc, (i, &c)| {
        if i > 0 && i % 3 == 0 {
            acc.insert(0, '.');
        }
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

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = chars[..max_chars - 3].iter().collect();
        format!("{truncated}...")
    }
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
        _ => "Outros",
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
        // 44 dígitos → 11 grupos → 10 espaços
        assert_eq!(fmt.chars().filter(|&c| c == ' ').count(), 10);
    }

    #[test]
    fn pad_lr_correct_width() {
        let line = pad_lr("TOTAL", "R$ 100,00", 48);
        // Remove \n e verifica largura
        assert_eq!(line.trim_end_matches('\n').chars().count(), 48);
    }

    #[test]
    fn pad_lr_overflow_no_panic() {
        // Labels maiores que cols — não deve entrar em pânico
        let line = pad_lr("TEXTO MUITO LONGO QUE EXCEDE", "VALOR TAMBEM LONGO", 20);
        assert!(!line.is_empty());
    }

    #[test]
    fn builder_rejects_model_55() {
        // XML minimalista com modelo 55 — ou falha no parse ou no check de modelo;
        // em ambos os casos deve retornar Err.
        let xml = r#"<nfeProc><NFe><infNFe Id="NFe35000000000000000000550010000000011234567890"><ide><mod>55</mod></ide><emit><CNPJ>00000000000000</CNPJ><enderEmit/></emit><det/><total/><transp/><pag/><infAdic/></infNFe></NFe><protNFe/></nfeProc>"#;
        let result = EscPosNFCeBuilder::new().xml(xml).build();
        assert!(result.is_err());
    }
}
