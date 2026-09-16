use crate::error::{DfeError, Result};
use crate::xml_extractor::{XmlExtractor, XmlExtractorSignature};

use super::nfce::{
    doc_label_cnpj_cpf, format_brl, format_chave_acesso, format_cnpj_cpf, format_datetime,
    format_decimal_br, pad_lr, pag_type_name, wrap_text,
};
use super::EscPosBuilder;

const SPACING_NORMAL: u8 = 30;
const SPACING_DIVIDER: u8 = 4;

/// Tamanho do módulo do QR em dots. A NT pede no mínimo 25 × 25 mm: a 203 DPI
/// (8 dots/mm), um QR de versão 5 (37 módulos) com módulo 6 dá ~28 mm.
const QR_MODULE_DOTS: u8 = 6;

/// Largura mínima do papel exigida pela NT 2026.003.
const PAPEL_MINIMO_MM: u8 = 56;

/// Consulta de NF-e modelo 55 quando o XML não traz `infNFeSupl/urlChave`.
const PORTAL_NACIONAL: &str = "www.nfe.fazenda.gov.br/portal";

/// Builder fluente do **DANFE Simplificado – Tipo 2** da **NF-e** (modelo 55) em
/// impressora térmica ESC/POS.
///
/// Leiaute do Ajuste SINIEF 13/2026 e da **NT 2026.003**, com as nove divisões na
/// ordem da norma: cabeçalho, itens, totais, IBS/CBS/IS (III-A), chave de acesso,
/// QR Code, consumidor, identificação da NF-e, mensagem fiscal e mensagem do
/// contribuinte.
///
/// A norma proíbe imprimir informação que não esteja no XML (fora protocolo,
/// `cMsg` e `xMsg`). Por isso o QR Code e o bloco IBS/CBS/IS **só saem quando o XML
/// os traz**: nota sem `infNFeSupl/qrCode` sai sem QR, e nota sem `IBSCBSTot` sai
/// sem o bloco III-A. Pelo mesmo motivo, este cupom não leva o crédito da crate.
///
/// # Exemplo
///
/// ```no_run
/// use dfe::EscPosDanfeNFeBuilder;
///
/// # fn example(xml_autorizado: String) -> Result<(), dfe::DfeError> {
/// let bytes = EscPosDanfeNFeBuilder::new()
///     .xml(xml_autorizado) // nfeProc como string, ou caminho terminado em ".xml"
///     .paper_width(80)
///     .columns(42)         // Bematech MP-4200 TH, Epson TM-T88V…
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct EscPosDanfeNFeBuilder {
    xml: Option<String>,
    paper_width: u8,
    cols: Option<usize>,
}

impl EscPosDanfeNFeBuilder {
    /// Cria um builder para papel de 80 mm.
    pub fn new() -> Self {
        Self { xml: None, paper_width: 80, cols: None }
    }

    /// String XML do `nfeProc` autorizado, ou caminho de arquivo terminado em `".xml"`.
    pub fn xml(mut self, src: impl Into<String>) -> Self {
        self.xml = Some(src.into());
        self
    }

    /// Largura do papel em milímetros: `80` ou `58`. Padrão: `80`.
    /// Abaixo de 56 mm o [`build`](Self::build) recusa — é o mínimo da NT.
    pub fn paper_width(mut self, mm: u8) -> Self {
        self.paper_width = mm;
        self
    }

    /// Sobrescreve o número de colunas de texto.
    ///
    /// Padrão: 48 para papel 80 mm · 32 para 58 mm. Use quando o modelo da
    /// impressora imprime menos colunas que o padrão (ex.: MP-4200 TH → 42).
    pub fn columns(mut self, cols: usize) -> Self {
        self.cols = Some(cols);
        self
    }

    /// Lê o XML, extrai os dados da NF-e e gera os bytes ESC/POS.
    ///
    /// # Erros
    ///
    /// Retorna [`DfeError`] se o XML for inválido, o arquivo não existir, o
    /// documento não for NF-e (modelo 55) ou o papel for menor que 56 mm.
    pub fn build(self) -> Result<Vec<u8>> {
        if self.paper_width < PAPEL_MINIMO_MM {
            return Err(DfeError::Configuracao(format!(
                "DANFE Simplificado Tipo 2 exige papel de pelo menos {PAPEL_MINIMO_MM} mm, recebeu {} mm",
                self.paper_width
            )));
        }
        let params = self.extract_params()?;
        build_receipt(params)
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

        let mod_ = inf.ide.mod_.as_deref().unwrap_or("");
        if mod_ != "55" {
            return Err(DfeError::Configuracao(format!(
                "EscPosDanfeNFeBuilder espera modelo 55 (NF-e), recebeu modelo {mod_}"
            )));
        }

        let chave_acesso = inf
            .id
            .as_deref()
            .unwrap_or("")
            .strip_prefix("NFe")
            .unwrap_or(inf.id.as_deref().unwrap_or(""))
            .to_string();

        let prot = nfe_proc.prot_nfe.inf_prot.as_ref();
        let n_prot = prot.and_then(|p| p.n_prot.clone()).unwrap_or_default();
        let dh_recbto = prot.and_then(|p| p.dh_recbto.clone()).unwrap_or_default();

        let emit = &inf.emit;
        let ender = &emit.ender_emit;

        let dest = inf.dest.as_ref();
        let dest_doc = dest
            .and_then(|d| d.cnpj.clone().or_else(|| d.cpf.clone()))
            .unwrap_or_default();
        let dest_x_nome = dest.and_then(|d| d.x_nome.clone()).unwrap_or_default();

        let icms_tot = inf.total.icms_tot.as_ref();
        let campo = |f: fn(&crate::xml_extractor::structs::ICMSTot) -> Option<String>| {
            icms_tot.and_then(f).unwrap_or_default()
        };

        // Mesma regra do cupom da NFC-e: a soma dos itens vence o total, que nem
        // todo emissor preenche.
        let v_tot_trib_itens: f64 = inf
            .det
            .iter()
            .filter_map(|det| det.imposto.v_tot_trib.as_deref())
            .map(num)
            .sum();
        let v_tot_trib = if v_tot_trib_itens > 0.0 {
            format!("{:.2}", v_tot_trib_itens)
        } else {
            campo(|t| t.v_tot_trib.clone())
        };

        let ibs_cbs = inf.total.ibs_cbs_tot.as_ref();
        let reforma = Reforma {
            v_cbs: ibs_cbs.and_then(|t| t.g_cbs.as_ref()).and_then(|g| g.v_cbs.clone()),
            v_ibs_uf: ibs_cbs
                .and_then(|t| t.g_ibs.as_ref())
                .and_then(|g| g.g_ibs_uf.as_ref())
                .and_then(|u| u.v_ibs_uf.clone()),
            v_ibs_mun: ibs_cbs
                .and_then(|t| t.g_ibs.as_ref())
                .and_then(|g| g.g_ibs_mun.as_ref())
                .and_then(|m| m.v_ibs_mun.clone()),
            v_is: inf.total.is_tot.as_ref().and_then(|t| t.v_is.clone()),
        };

        let supl = nfe_proc.nfe.inf_nfe_supl.as_ref();
        let qr_code = supl
            .and_then(|s| s.qr_code.clone())
            .map(|q| q.trim().to_string())
            .filter(|q| !q.is_empty());
        let url_chave = supl
            .and_then(|s| s.url_chave.clone())
            .filter(|u| !u.trim().is_empty())
            .unwrap_or_else(|| PORTAL_NACIONAL.to_string());

        let items = inf
            .det
            .iter()
            .map(|det| {
                let p = &det.prod;
                Item {
                    c_prod: p.c_prod.clone().unwrap_or_default(),
                    x_prod: p.x_prod.clone().unwrap_or_default(),
                    q_com: p.q_com.clone().unwrap_or_default(),
                    u_com: p.u_com.clone().unwrap_or_default(),
                    v_un_com: p.v_un_com.clone().unwrap_or_default(),
                    v_prod: p.v_prod.clone().unwrap_or_default(),
                }
            })
            .collect();

        Ok(BuildParams {
            chave_acesso,
            n_prot,
            dh_recbto,
            emit_x_nome: emit.x_nome.clone().unwrap_or_default(),
            emit_cnpj: emit.cnpj.clone().unwrap_or_default(),
            emit_ie: emit.ie.clone().unwrap_or_default(),
            emit_uf: ender.uf.clone().unwrap_or_default(),
            emit_x_lgr: ender.x_lgr.clone().unwrap_or_default(),
            emit_nro: ender.nro.clone().unwrap_or_default(),
            emit_x_bairro: ender.x_bairro.clone().unwrap_or_default(),
            emit_x_mun: ender.x_mun.clone().unwrap_or_default(),
            tp_amb: inf.ide.tp_amb.clone().unwrap_or_default(),
            tp_emis: inf.ide.tp_emis.clone().unwrap_or_default(),
            serie: inf.ide.serie.clone().unwrap_or_default(),
            n_nf: inf.ide.n_nf.clone().unwrap_or_default(),
            dh_emi: inf.ide.dh_emi.clone().unwrap_or_default(),
            dest_doc,
            dest_x_nome,
            v_prod: campo(|t| t.v_prod.clone()),
            v_desc: campo(|t| t.v_desc.clone()),
            v_nf: campo(|t| t.v_nf.clone()),
            v_tot_trib,
            v_troco: inf.pag.v_troco.clone().unwrap_or_else(|| "0.00".to_string()),
            payments: inf
                .pag
                .det_pag
                .iter()
                .map(|d| (d.t_pag.clone().unwrap_or_default(), d.v_pag.clone().unwrap_or_default()))
                .collect(),
            reforma,
            inf_ad_fisco: inf.inf_adic.inf_ad_fisco.clone().unwrap_or_default(),
            inf_cpl: inf.inf_adic.inf_cpl.clone().unwrap_or_default(),
            qr_code,
            url_chave,
            items,
            paper_width: self.paper_width,
            cols: self.cols,
        })
    }
}

impl Default for EscPosDanfeNFeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tipos internos ────────────────────────────────────────────────────────────

struct Item {
    c_prod: String,
    x_prod: String,
    q_com: String,
    u_com: String,
    v_un_com: String,
    v_prod: String,
}

/// Divisão III-A. Cada valor é `None` quando o XML não traz o campo.
#[derive(Default)]
struct Reforma {
    v_cbs: Option<String>,
    v_ibs_uf: Option<String>,
    v_ibs_mun: Option<String>,
    v_is: Option<String>,
}

impl Reforma {
    fn vazia(&self) -> bool {
        self.v_cbs.is_none() && self.v_ibs_uf.is_none() && self.v_ibs_mun.is_none() && self.v_is.is_none()
    }
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
    dest_doc: String,
    dest_x_nome: String,
    v_prod: String,
    v_desc: String,
    v_nf: String,
    v_tot_trib: String,
    v_troco: String,
    payments: Vec<(String, String)>,
    reforma: Reforma,
    inf_ad_fisco: String,
    inf_cpl: String,
    qr_code: Option<String>,
    url_chave: String,
    items: Vec<Item>,
    paper_width: u8,
    cols: Option<usize>,
}

// ── Builder do cupom ──────────────────────────────────────────────────────────

fn build_receipt(p: BuildParams) -> Result<Vec<u8>> {
    let cols: usize = p.cols.unwrap_or(if p.paper_width >= 80 { 48 } else { 32 });
    let contingencia = contingencia_pendente(&p.tp_emis);

    let mut b = EscPosBuilder::new()
        .paper_width(p.paper_width)
        .columns(cols)
        .line_spacing(SPACING_NORMAL);

    // ── Divisão I — cabeçalho ─────────────────────────────────────────────────
    let emit_name = if p.emit_x_nome.trim().is_empty() {
        "EMITENTE NAO INFORMADO".to_string()
    } else {
        p.emit_x_nome.clone()
    };
    b = b.align_center().bold(true);
    for line in wrap_text(&emit_name, cols) {
        b = b.text(format!("{line}\n"));
    }
    b = b.bold(false);

    let ie = if p.emit_ie.is_empty() { "Isento" } else { p.emit_ie.as_str() };
    for line in wrap_text(&format!("CNPJ: {}  IE: {}", format_cnpj_cpf(&p.emit_cnpj), ie), cols) {
        b = b.text(format!("{line}\n"));
    }
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
    }
    if !p.emit_x_mun.is_empty() {
        b = b.text(format!("{}/{}\n", p.emit_x_mun, p.emit_uf));
    }

    b = divider(b)
        .bold(true)
        .text("DANFE Simplificado Tipo 2\n")
        .bold(false)
        .text("Documento Auxiliar da NF-e - modelo 55\n");
    if contingencia {
        b = aviso_contingencia(b, cols);
    }

    // ── Divisão II — produtos ─────────────────────────────────────────────────
    b = divider(b)
        .align_left()
        .bold(true)
        .text("CÓD  DESCRIÇÃO\n")
        .text(pad_lr("QTD UN x VL UNIT", "VL TOTAL", cols))
        .bold(false);
    b = divider(b);

    for item in &p.items {
        // Descrição inteira, quebrada em linhas: o documento não pode cortar o que o XML diz.
        for line in wrap_text(&format!("{} {}", item.c_prod, item.x_prod), cols) {
            b = b.text(format!("{line}\n"));
        }
        let left = format!(
            "  {} {} x {}",
            format_decimal_br(&item.q_com),
            item.u_com,
            format_brl(&item.v_un_com)
        );
        b = b.text(pad_lr(&left, &format_brl(&item.v_prod), cols));
    }

    // ── Divisão III — totais ──────────────────────────────────────────────────
    b = divider(b)
        .text(pad_lr("Qtd. total de itens", &p.items.len().to_string(), cols))
        .text(pad_lr("Valor total R$", &format_brl(&p.v_prod), cols));

    // Acréscimo é o que leva o vProd até o vNF além do desconto (frete, seguro,
    // outras despesas, ST, IPI): sai da própria conta do XML, sem somar campo a campo.
    let acrescimos = num(&p.v_nf) - num(&p.v_prod) + num(&p.v_desc);
    if acrescimos >= 0.005 {
        b = b.text(pad_lr("Acréscimos R$", &format_brl(&format!("{acrescimos:.2}")), cols));
    }
    if num(&p.v_desc) > 0.0 {
        b = b.text(pad_lr("Descontos R$", &format!("- {}", format_brl(&p.v_desc)), cols));
    }
    b = b
        .bold(true)
        .text(pad_lr("Valor a pagar R$", &format_brl(&p.v_nf), cols))
        .bold(false)
        .text("\n")
        .bold(true)
        .text(pad_lr("FORMA DE PAGAMENTO", "VALOR PAGO R$", cols))
        .bold(false);
    for (t_pag, v_pag) in &p.payments {
        b = b.text(pad_lr(pag_type_name(t_pag), &format_brl(v_pag), cols));
    }
    // Troco é obrigatório na NT, mesmo zerado.
    b = b.text(pad_lr("Troco R$", &format_brl(&p.v_troco), cols));

    // ── Divisão III-A — IBS / CBS / IS (só com o grupo no XML) ────────────────
    if !p.reforma.vazia() {
        b = divider(b).align_center().bold(true).text("IBS / CBS / IS\n").bold(false).align_left();
        let linhas = [
            ("CBS R$", &p.reforma.v_cbs),
            ("IBS estadual R$", &p.reforma.v_ibs_uf),
            ("IBS municipal R$", &p.reforma.v_ibs_mun),
            ("IS R$", &p.reforma.v_is),
        ];
        for (rotulo, valor) in linhas {
            if let Some(v) = valor {
                b = b.text(pad_lr(rotulo, &format_brl(v), cols));
            }
        }
    }

    // ── Divisão IV — consulta pela chave de acesso ────────────────────────────
    b = divider(b)
        .align_center()
        .text("Consulte pela Chave de Acesso em\n");
    for line in wrap_text(&p.url_chave, cols) {
        b = b.text(format!("{line}\n"));
    }
    // 11 grupos de 4 = 54 caracteres: só cabem numa linha na fonte B (~64 col. em 80 mm).
    let cols_b: usize = if p.paper_width >= 80 { 64 } else { 42 };
    b = b.font_b(true);
    for chunk in wrap_text(&format_chave_acesso(&p.chave_acesso), cols_b) {
        b = b.text(format!("{chunk}\n"));
    }
    b = b.font_b(false).barcode_128(&p.chave_acesso);

    // ── Divisão V — QR Code (só com infNFeSupl/qrCode no XML) ─────────────────
    if let Some(qr) = &p.qr_code {
        b = b.align_center().qr_code(qr, QR_MODULE_DOTS).text("\n");
    }

    // ── Divisão VI — consumidor ───────────────────────────────────────────────
    b = divider(b).align_center();
    if p.dest_doc.is_empty() {
        b = b.text("CONSUMIDOR NÃO IDENTIFICADO\n");
    } else {
        b = b.text(format!(
            "CONSUMIDOR {}: {}\n",
            doc_label_cnpj_cpf(&p.dest_doc),
            format_cnpj_cpf(&p.dest_doc)
        ));
    }
    for line in wrap_text(&p.dest_x_nome, cols) {
        b = b.text(format!("{line}\n"));
    }

    // ── Divisão VII — identificação da NF-e ───────────────────────────────────
    b = divider(b)
        .bold(true)
        .text(format!("NF-e nº {} Série {:0>3}\n", format_numero_nf(&p.n_nf), p.serie))
        .bold(false)
        .text(format!("Emissão: {}\n", format_datetime(&p.dh_emi)));
    if !p.n_prot.is_empty() {
        b = b
            .bold(true)
            .text("Protocolo de autorização\n")
            .bold(false)
            .text(format!("{} {}\n", p.n_prot, format_datetime(&p.dh_recbto)));
    }

    // ── Divisão VIII — mensagem fiscal ────────────────────────────────────────
    let homologacao = p.tp_amb == "2";
    if homologacao || contingencia || !p.inf_ad_fisco.trim().is_empty() {
        b = divider(b).align_center();
        if homologacao {
            // Duas linhas fixas: deixar o wrap decidir separa "SEM VALOR" de "FISCAL".
            b = b.bold(true);
            for line in wrap_text("EMITIDA EM AMBIENTE DE HOMOLOGAÇÃO", cols)
                .into_iter()
                .chain(wrap_text("SEM VALOR FISCAL", cols))
            {
                b = b.text(format!("{line}\n"));
            }
            b = b.bold(false);
        }
        if contingencia {
            // Segundo dos dois locais que a NT exige para o aviso.
            b = aviso_contingencia(b, cols);
        }
        if !p.inf_ad_fisco.trim().is_empty() {
            b = b.align_left();
            for line in wrap_text(&p.inf_ad_fisco, cols) {
                b = b.text(format!("{line}\n"));
            }
        }
    }

    // ── Divisão IX — mensagem do contribuinte ─────────────────────────────────
    if !p.inf_cpl.trim().is_empty() {
        b = divider(b).align_left();
        for line in wrap_text(&p.inf_cpl, cols) {
            b = b.text(format!("{line}\n"));
        }
    }
    if num(&p.v_tot_trib) > 0.0 {
        b = divider(b)
            .align_center()
            .text(format!("Valor Aprox. dos Tributos R$ {}\n", format_brl(&p.v_tot_trib)));
    }

    b = b.line_spacing_default().feed(6).cut();
    Ok(b.build())
}

fn divider(b: EscPosBuilder) -> EscPosBuilder {
    b.line_spacing(SPACING_DIVIDER).divider().line_spacing(SPACING_NORMAL)
}

fn aviso_contingencia(b: EscPosBuilder, cols: usize) -> EscPosBuilder {
    let mut b = b.align_center().bold(true);
    for line in wrap_text("EMITIDA EM CONTINGÊNCIA", cols)
        .into_iter()
        .chain(wrap_text("Pendente de autorização", cols))
    {
        b = b.text(format!("{line}\n"));
    }
    b.bold(false)
}

/// Modalidades em que o DANFE sai **antes** da autorização da SEFAZ: FS (2),
/// EPEC (4), FS-DA (5) e off-line (9). SVC-AN/SVC-RS (6/7) são autorizações
/// normais, só que em outro ambiente — não levam aviso.
fn contingencia_pendente(tp_emis: &str) -> bool {
    matches!(tp_emis, "2" | "4" | "5" | "9")
}

/// `504` → `000.000.504`.
fn format_numero_nf(n_nf: &str) -> String {
    let n = format!("{:0>9}", n_nf.trim());
    if n.len() != 9 {
        return n;
    }
    format!("{}.{}.{}", &n[0..3], &n[3..6], &n[6..9])
}

fn num(s: &str) -> f64 {
    s.replace(',', ".").parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XML_55: &str = include_str!("../../tests/fixtures/xml/nfe55_autorizada.xml");

    /// Texto legível do buffer — acentos saem em CP850, então só vale conferir ASCII.
    fn texto(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).into_owned()
    }

    fn gerar(xml: &str) -> Vec<u8> {
        EscPosDanfeNFeBuilder::new().xml(xml).build().expect("DANFE deveria sair")
    }

    fn tem_qr(bytes: &[u8]) -> bool {
        bytes.windows(3).any(|w| w == [0x1D, 0x28, 0x6B])
    }

    #[test]
    fn fixture_sai_com_a_expressao_obrigatoria_e_os_dados_da_nota() {
        let t = texto(&gerar(XML_55));
        assert!(t.contains("DANFE Simplificado Tipo 2"));
        assert!(t.contains("modelo 55"));
        assert!(t.contains("EMPRESA FICTICIA LTDA"), "cabeçalho usa a razão social");
        assert!(t.contains("2 PRODUTO TESTE UNITARIO"), "item leva código e descrição");
        assert!(t.contains("20,000 UN x 45,00"));
        assert!(t.contains("900,00"));
        assert!(t.contains("000.000.504"));
        assert!(t.contains("CONSUMIDOR CNPJ: 11.222.333/0001-81"));
    }

    #[test]
    fn chave_sai_em_11_grupos_de_4() {
        let t = texto(&gerar(XML_55));
        assert!(t.contains("3526 0300 0000 0000 0191 5500 1000 0005 0410 0000 0001"));
    }

    #[test]
    fn troco_sai_mesmo_zerado() {
        let t = texto(&gerar(XML_55));
        assert!(t.contains("Troco R$"));
    }

    #[test]
    fn bloco_ibs_cbs_sai_quando_o_xml_traz_o_grupo() {
        let t = texto(&gerar(XML_55));
        assert!(t.contains("IBS / CBS / IS"));
        assert!(t.contains("7,45"), "CBS");
        assert!(t.contains("0,83"), "IBS estadual");
        assert!(!t.contains("IS R$"), "sem ISTot no XML, sem linha de IS");
    }

    #[test]
    fn bloco_ibs_cbs_some_quando_o_xml_nao_traz() {
        let ini = XML_55.find("<IBSCBSTot>").unwrap();
        let fim = XML_55.find("</IBSCBSTot>").unwrap() + "</IBSCBSTot>".len();
        let xml = format!("{}{}", &XML_55[..ini], &XML_55[fim..]);
        let t = texto(&gerar(&xml));
        assert!(!t.contains("IBS / CBS / IS"));
        assert!(!t.contains("IBS municipal"));
    }

    #[test]
    fn qr_so_sai_quando_o_xml_tem_infnfesupl() {
        // NF-e 55 emitida hoje não leva infNFeSupl: imprimir um QR inventado violaria a NT.
        let sem = gerar(XML_55);
        assert!(!tem_qr(&sem));
        assert!(texto(&sem).contains(PORTAL_NACIONAL));

        let xml = XML_55.replace(
            "<Signature ",
            "<infNFeSupl><qrCode>https://qr.teste/?p=1</qrCode><urlChave>https://consulta.teste</urlChave></infNFeSupl><Signature ",
        );
        let com = gerar(&xml);
        assert!(tem_qr(&com));
        assert!(texto(&com).contains("https://consulta.teste"));
    }

    #[test]
    fn recusa_nfce() {
        let xml = XML_55.replace("<mod>55</mod>", "<mod>65</mod>");
        assert!(EscPosDanfeNFeBuilder::new().xml(xml).build().is_err());
    }

    #[test]
    fn recusa_papel_abaixo_de_56_mm() {
        assert!(EscPosDanfeNFeBuilder::new().xml(XML_55).paper_width(48).build().is_err());
        assert!(EscPosDanfeNFeBuilder::new().xml(XML_55).paper_width(58).build().is_ok());
    }

    #[test]
    fn homologacao_leva_o_aviso_sem_valor_fiscal() {
        assert!(!texto(&gerar(XML_55)).contains("SEM VALOR FISCAL"));
        let xml = XML_55.replace("<tpAmb>1</tpAmb>", "<tpAmb>2</tpAmb>");
        assert!(texto(&gerar(&xml)).contains("SEM VALOR FISCAL"));
    }

    #[test]
    fn contingencia_avisa_em_dois_lugares() {
        let xml = XML_55.replace("<tpEmis>1</tpEmis>", "<tpEmis>5</tpEmis>");
        let t = texto(&gerar(&xml));
        assert_eq!(t.matches("EMITIDA EM CONTING").count(), 2);
        assert_eq!(texto(&gerar(XML_55)).matches("EMITIDA EM CONTING").count(), 0);
    }

    #[test]
    fn svc_nao_e_contingencia_pendente() {
        assert!(!contingencia_pendente("6"));
        assert!(!contingencia_pendente("7"));
        assert!(contingencia_pendente("2"));
        assert!(contingencia_pendente("5"));
    }

    #[test]
    fn mensagens_fiscal_e_do_contribuinte() {
        let xml = XML_55.replace("<infCpl>", "<infAdFisco>MENSAGEM DO FISCO</infAdFisco><infCpl>");
        let t = texto(&gerar(&xml));
        assert!(t.contains("MENSAGEM DO FISCO"));
        assert!(t.contains("INFORMACOES COMPLEMENTARES DE TESTE"));
        assert!(t.contains("Valor Aprox. dos Tributos R$ 72,00"));
    }

    #[test]
    fn nao_leva_credito_da_crate() {
        // A NT proíbe informação que não esteja no XML.
        assert!(!texto(&gerar(XML_55)).contains("Gerado por"));
    }

    #[test]
    fn colunas_forcadas_mudam_o_separador() {
        let bytes = EscPosDanfeNFeBuilder::new().xml(XML_55).columns(42).build().unwrap();
        let t = texto(&bytes);
        assert!(t.contains(&format!("{}\n", "-".repeat(42))));
        assert!(!t.contains(&"-".repeat(43)));
    }

    #[test]
    fn acrescimo_sai_da_conta_do_xml() {
        let xml = XML_55
            .replace("<vFrete>0.00</vFrete>", "<vFrete>10.00</vFrete>")
            .replace("<vNF>900.00</vNF>", "<vNF>910.00</vNF>");
        let t = texto(&gerar(&xml));
        assert!(t.contains("10,00"));
        assert!(t.contains("910,00"));
        assert!(!texto(&gerar(XML_55)).contains("scimos R$"), "sem acréscimo, sem linha");
    }

    #[test]
    fn numero_da_nf_com_pontos() {
        assert_eq!(format_numero_nf("504"), "000.000.504");
        assert_eq!(format_numero_nf("123456789"), "123.456.789");
    }
}
