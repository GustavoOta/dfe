mod det;
mod det_process;
mod emit;
mod flag;
mod ide;
mod inf_adic;
pub mod pag;
mod total;
mod transp;

// A7 (§2.1): o god file foi dividido na espinha de módulo — `types` (structs), `xml` (montagem +
// assinatura), `service` (orquestração/envio) e `parser` (resposta). O `mod.rs` fica só com o
// Builder público e as re-exports.
mod parser;
mod service;
mod types;
mod xml;

use crate::error::{DfeError, Result};
use crate::tipos::{Dest, Det, Emit, Entrega, Ide, InfAdic, Pag, Total, Transp};
use rust_decimal::Decimal;

use types::NFeInterno;
pub use types::{InfProt, Response, TagInfProt};
pub use service::transmitir_xml_assinado;

// Validações de contingência (`tpEmis != 1`), comuns a `gerar_xml()` e `emitir()` — fail-fast,
// sem I/O, mesmo padrão do `SubstituicaoBuilder` (ver `substituicao/mod.rs`).
fn validar_contingencia(ide: &Ide) -> Result<()> {
    // tpEmis=9 (contingência off-line) é exclusivo de NFC-e (NT 2025.001) — não existe pra NF-e mod. 55.
    if ide.tp_emis == 9 && ide.mod_ != 65 {
        return Err(DfeError::Validacao(
            "tp_emis=9 (contingência off-line) só é válido para NFC-e (mod=65)".to_string(),
        ));
    }
    if ide.tp_emis != 1 {
        let dh_cont = ide.dh_cont.as_deref().unwrap_or("").trim();
        if dh_cont.is_empty() {
            return Err(DfeError::Validacao(
                "dh_cont (dhCont) é obrigatório quando tp_emis != 1".to_string(),
            ));
        }
        // XSD (leiauteNFe_v4.00.xsd): xJust 15-256 caracteres.
        let x_just_len = ide.x_just.as_deref().unwrap_or("").trim().chars().count();
        if x_just_len < 15 {
            return Err(DfeError::Validacao(
                "x_just (xJust) deve ter no mínimo 15 caracteres quando tp_emis != 1".to_string(),
            ));
        }
        if x_just_len > 256 {
            return Err(DfeError::Validacao(
                "x_just (xJust) deve ter no máximo 256 caracteres quando tp_emis != 1".to_string(),
            ));
        }
    }
    Ok(())
}

// ─── Builder público ──────────────────────────────────────────────────────────
/// Builder fluente para emissão de **NF-e** (modelo 55) e **NFC-e** (modelo 65).
///
/// Monte a nota chamando os métodos de configuração em qualquer ordem e finalize
/// com [`NFeBuilder::emitir`], que valida, assina e transmite para a SEFAZ.
/// Use [`NFeBuilder::gerar_xml`] para apenas gerar e validar o XML sem enviar à SEFAZ.
///
/// # Exemplo
///
/// ```no_run
/// use dfe::{NFeBuilder, DfeError};
/// use dfe::tipos::{Det, Emit, Icms, Ide, Pag, Pis, Cofins, Total, Transp};
///
/// # async fn example() -> Result<(), DfeError> {
/// let resp = NFeBuilder::new()
///     .cert("./cert.pfx", "senha")
///     .ide(Ide { c_uf: 35, mod_: 55, serie: 1, n_nf: 1, tp_amb: 2, ..Default::default() })
///     .emitente(Emit { cnpj: Some("11111111111111".into()), ..Default::default() })
///     .itens(vec![Det {
///         icms: Icms::sn102(0, "400"),
///         pis: Pis::Nt { cst: "07".into() },
///         cofins: Cofins::Nt { cst: "07".into() },
///         ..Default::default()
///     }])
///     .total(Total::default())
///     .transporte(Transp::default())
///     .pagamento(Pag::default())
///     .emitir()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub struct NFeBuilder {
    cert_path: Option<String>,
    cert_pass: Option<String>,
    ide: Option<Ide>,
    emitente: Option<Emit>,
    destinatario: Option<Dest>,
    itens: Vec<Det>,
    total: Option<Total>,
    transporte: Option<Transp>,
    pagamento: Option<Pag>,
    informacoes_adicionais: Option<InfAdic>,
    id_csc: Option<String>,
    csc: Option<String>,
    active_ibs_cbs: Option<String>,
    desconto_rateio: Option<Decimal>,
    frete_rateio: Option<Decimal>,
    entrega: Option<Entrega>,
    referencias: Vec<String>,
}

impl NFeBuilder {
    /// Cria um builder vazio. Chame os métodos de configuração antes de [`emitir`](Self::emitir).
    pub fn new() -> Self {
        Self {
            cert_path: None, cert_pass: None, ide: None, emitente: None,
            destinatario: None, itens: Vec::new(), total: None, transporte: None,
            pagamento: None, informacoes_adicionais: None, id_csc: None, csc: None,
            active_ibs_cbs: None, desconto_rateio: None, frete_rateio: None,
            entrega: None, referencias: Vec::new(),
        }
    }

    /// Caminho do certificado A1 (`.pfx`) e sua senha. **Obrigatório.**
    pub fn cert(mut self, path: &str, pass: &str) -> Self {
        self.cert_path = Some(path.to_string()); self.cert_pass = Some(pass.to_string()); self
    }
    /// Identificação do documento (`<ide>`). **Obrigatório.**
    pub fn ide(mut self, ide: Ide)       -> Self { self.ide = Some(ide); self }
    /// Dados do emitente (`<emit>`). **Obrigatório.**
    pub fn emitente(mut self, e: Emit)   -> Self { self.emitente = Some(e); self }
    /// Dados do destinatário (`<dest>`). Obrigatório para NF-e modelo 55.
    pub fn destinatario(mut self, d: Dest) -> Self { self.destinatario = Some(d); self }
    /// Lista de itens (`<det>`). **Obrigatório.** Totais calculados automaticamente.
    pub fn itens(mut self, itens: Vec<Det>) -> Self { self.itens.extend(itens); self }
    /// Totais globais (`<total>`). **Obrigatório.** Informe apenas frete, seguro e ST; demais campos são auto-calculados.
    pub fn total(mut self, t: Total)     -> Self { self.total = Some(t); self }
    /// Dados de transporte (`<transp>`). **Obrigatório.**
    pub fn transporte(mut self, t: Transp) -> Self { self.transporte = Some(t); self }
    /// Forma de pagamento (`<pag>`). **Obrigatório.**
    pub fn pagamento(mut self, p: Pag)   -> Self { self.pagamento = Some(p); self }
    /// Informações adicionais (`<infAdic>`). Opcional.
    pub fn informacoes_adicionais(mut self, i: InfAdic) -> Self { self.informacoes_adicionais = Some(i); self }
    /// ID do CSC (Código de Segurança do Contribuinte). **Obrigatório para NFC-e.**
    pub fn id_csc(mut self, id: &str)    -> Self { self.id_csc = Some(id.to_string()); self }
    /// Valor do CSC. **Obrigatório para NFC-e.**
    pub fn csc(mut self, csc: &str)      -> Self { self.csc = Some(csc.to_string()); self }
    /// Ativa IBS/CBS (reforma tributária). Passe o código de classificação tributária.
    pub fn active_ibs_cbs(mut self, f: &str) -> Self { self.active_ibs_cbs = Some(f.to_string()); self }
    /// Desconto global rateado proporcionalmente nos itens.
    pub fn desconto_rateio(mut self, v: Decimal) -> Self { self.desconto_rateio = Some(v); self }

    /// Frete total a ratear proporcionalmente ao `vProd` de cada item (emite `det/prod/vFrete`
    /// e soma em `ICMSTot/vFrete`). Ausente/zero → nenhum `<vFrete>` por item (XML inalterado).
    /// A BC do ICMS já deve vir com a parcela de frete embutida (a crate não recalcula BC).
    pub fn frete_rateio(mut self, v: Decimal) -> Self { self.frete_rateio = Some(v); self }

    /// Local de entrega (`<entrega>`) — endereço de entrega no delivery (`indPres = 4`).
    /// Ausente → grupo não emitido (XML inalterado).
    pub fn entrega(mut self, e: Entrega) -> Self { self.entrega = Some(e); self }
    /// Adiciona uma chave de acesso referenciada (`<NFref><refNFe>`). Use para devolução (finNFe=4).
    pub fn referencia(mut self, chave: &str) -> Self { self.referencias.push(chave.to_string()); self }

    /// Gera e valida o XML da NF-e sem enviar à SEFAZ.
    ///
    /// Útil para validação prévia (ex.: NF-e de devolução antes da emissão).
    /// Retorna o XML assinado e validado pelo XSD oficial.
    pub async fn gerar_xml(self) -> crate::error::Result<String> {
        let cert_path  = self.cert_path.ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass  = self.cert_pass.ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let ide        = self.ide.ok_or_else(|| DfeError::Validacao("ide não informado".to_string()))?;
        let emitente   = self.emitente.ok_or_else(|| DfeError::Validacao("emitente não informado".to_string()))?;
        let total      = self.total.ok_or_else(|| DfeError::Validacao("total não informado".to_string()))?;
        let transporte = self.transporte.ok_or_else(|| DfeError::Validacao("transporte não informado".to_string()))?;
        let pagamento  = self.pagamento.ok_or_else(|| DfeError::Validacao("pagamento não informado".to_string()))?;

        validar_contingencia(&ide)?;

        if self.itens.is_empty() {
            return Err(DfeError::Validacao("pelo menos um item (det) deve ser informado".to_string()));
        }

        let signed = xml::build_signed_xml(NFeInterno {
            cert_path, cert_pass, id_csc: self.id_csc, csc: self.csc,
            ide, emit: emitente, dest: self.destinatario,
            det: self.itens, total, transp: transporte, pag: pagamento,
            inf_adic: self.informacoes_adicionais,
            active_ibs_cbs: self.active_ibs_cbs,
            desconto_rateio: self.desconto_rateio,
            frete_rateio: self.frete_rateio,
            entrega: self.entrega,
            referencias: self.referencias,
        }).await?;

        Ok(signed.validated_xml)
    }

    /// Valida, assina e transmite a NF-e/NFC-e para a SEFAZ.
    ///
    /// Retorna [`Response`] com o protocolo de autorização e o XML `nfeProc`.
    /// Em ambiente de homologação (`tp_amb = 2`), o `x_prod` do primeiro item
    /// é substituído automaticamente pelo texto exigido pela SEFAZ.
    ///
    /// # Erros
    ///
    /// Retorna [`DfeError`] se algum campo obrigatório estiver ausente,
    /// a assinatura falhar ou a SEFAZ retornar erro de transmissão.
    pub async fn emitir(self) -> Result<Response> {
        let cert_path  = self.cert_path.ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass  = self.cert_pass.ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let ide        = self.ide.ok_or_else(|| DfeError::Validacao("ide não informado".to_string()))?;
        let emitente   = self.emitente.ok_or_else(|| DfeError::Validacao("emitente não informado".to_string()))?;
        let total      = self.total.ok_or_else(|| DfeError::Validacao("total não informado".to_string()))?;
        let transporte = self.transporte.ok_or_else(|| DfeError::Validacao("transporte não informado".to_string()))?;
        let pagamento  = self.pagamento.ok_or_else(|| DfeError::Validacao("pagamento não informado".to_string()))?;

        validar_contingencia(&ide)?;

        if self.itens.is_empty() {
            return Err(DfeError::Validacao("pelo menos um item (det) deve ser informado".to_string()));
        }

        service::emit_nfe(NFeInterno {
            cert_path, cert_pass, id_csc: self.id_csc, csc: self.csc,
            ide, emit: emitente, dest: self.destinatario,
            det: self.itens, total, transp: transporte, pag: pagamento,
            inf_adic: self.informacoes_adicionais,
            active_ibs_cbs: self.active_ibs_cbs,
            desconto_rateio: self.desconto_rateio,
            frete_rateio: self.frete_rateio,
            entrega: self.entrega,
            referencias: self.referencias,
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ide_contingencia(mod_: u32, dh_cont: Option<&str>, x_just: Option<&str>) -> Ide {
        Ide {
            mod_,
            tp_emis: 9,
            dh_cont: dh_cont.map(str::to_string),
            x_just: x_just.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn tp_emis_9_so_e_valido_para_nfce() {
        let ide = ide_contingencia(55, Some("2026-07-08T10:00:00-03:00"), Some("Falha de comunicação com a SEFAZ"));
        let err = validar_contingencia(&ide).unwrap_err();
        assert!(matches!(err, DfeError::Validacao(m) if m.contains("NFC-e")));
    }

    #[test]
    fn tp_emis_9_exige_dh_cont() {
        let ide = ide_contingencia(65, None, Some("Falha de comunicação com a SEFAZ"));
        let err = validar_contingencia(&ide).unwrap_err();
        assert!(matches!(err, DfeError::Validacao(m) if m.contains("dh_cont")));
    }

    #[test]
    fn tp_emis_9_exige_x_just_minimo_15_chars() {
        let ide = ide_contingencia(65, Some("2026-07-08T10:00:00-03:00"), Some("curta demais"));
        let err = validar_contingencia(&ide).unwrap_err();
        assert!(matches!(err, DfeError::Validacao(m) if m.contains("mínimo")));
    }

    #[test]
    fn tp_emis_9_rejeita_x_just_acima_de_256_chars() {
        let ide = ide_contingencia(65, Some("2026-07-08T10:00:00-03:00"), Some(&"a".repeat(257)));
        let err = validar_contingencia(&ide).unwrap_err();
        assert!(matches!(err, DfeError::Validacao(m) if m.contains("máximo")));
    }

    #[test]
    fn tp_emis_9_com_dh_cont_e_x_just_validos_passa() {
        let ide = ide_contingencia(65, Some("2026-07-08T10:00:00-03:00"), Some("Falha de comunicação com a SEFAZ"));
        assert!(validar_contingencia(&ide).is_ok());
    }

    #[test]
    fn tp_emis_1_nao_exige_dh_cont_nem_x_just() {
        let ide = Ide { tp_emis: 1, ..Default::default() };
        assert!(validar_contingencia(&ide).is_ok());
    }
}
