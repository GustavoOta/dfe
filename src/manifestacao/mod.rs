//! Manifestação do destinatário — evento de NF-e processado pelo **Ambiente Nacional**.
//!
//! A4b: a API pública é o [`ManifestacaoBuilder`] (antes eram funções livres
//! `nfe_*_operacao(Manifestacao)`). O builder é **fino** (Princípio 6): valida a entrada e
//! delega à mesma espinha de `cancelar` — monta o `infEvento` via [`crate::interno::evento`],
//! assina via [`crate::interno::assinatura`] e envia via [`crate::interno::transporte`].
//! Não escreve nada no diretório de trabalho; os XMLs de envio/resposta voltam em
//! [`Response::send_xml`]/[`Response::receive_xml`].
//!
//! Referência: NT 2014.002 (eventos de manifestação do destinatário — tpEvento 210200/210210/
//! 210220/210240).

use crate::error::{DfeError, Result};
use crate::interno::cert::{DigestValue, RawPubKey, Sign};
use crate::interno::cleaner::Strings;
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::tipos::manifestacao::{InfEvento, Response};

const TP_EVENTO_CONFIRMACAO_OPERACAO: &str = "210200";
const TP_EVENTO_CIENCIA_OPERACAO: &str = "210210";
const TP_EVENTO_DESCONHECIMENTO_OPERACAO: &str = "210220";
const TP_EVENTO_OPERACAO_NAO_REALIZADA: &str = "210240";
const VER_EVENTO: &str = "1.00";
// Manifestacao do destinatario e processada pelo Ambiente Nacional.
const C_ORGAO_AMBIENTE_NACIONAL: &str = "91";

// ─── Builder público ──────────────────────────────────────────────────────────

/// Registra a manifestação do destinatário de uma NF-e.
///
/// Padrão fluente: `new()` → setters → um método terminal por tipo de evento
/// (`ciencia_operacao`, `confirmacao_operacao`, `desconhecimento_operacao`,
/// `operacao_nao_realizada`), que assina e envia ao Ambiente Nacional.
///
/// # Exemplo
/// ```no_run
/// use dfe::ManifestacaoBuilder;
/// # async fn ex() -> Result<(), dfe::DfeError> {
/// let resp = ManifestacaoBuilder::new()
///     .cert("./cert.pfx", "senha")
///     .cnpj("11222333000181")
///     .tp_amb(2)
///     .chave("35000000000000000000550010000000001000000001")
///     .ciencia_operacao()
///     .await?;
/// println!("cStat: {}", resp.response.c_stat);
/// # Ok(()) }
/// ```
pub struct ManifestacaoBuilder {
    cert_path: Option<String>,
    cert_pass: Option<String>,
    cnpj:      Option<String>,
    tp_amb:    Option<u8>,
    chave:     Option<String>,
}

impl ManifestacaoBuilder {
    pub fn new() -> Self {
        Self { cert_path: None, cert_pass: None, cnpj: None, tp_amb: None, chave: None }
    }

    /// Certificado A1 (`.pfx`) e sua senha.
    pub fn cert(mut self, path: &str, pass: &str) -> Self {
        self.cert_path = Some(path.to_string());
        self.cert_pass = Some(pass.to_string());
        self
    }

    /// CNPJ do destinatário que manifesta (autor do evento).
    pub fn cnpj(mut self, v: &str) -> Self { self.cnpj = Some(v.to_string()); self }

    /// 1 = Produção | 2 = Homologação
    pub fn tp_amb(mut self, v: u8) -> Self { self.tp_amb = Some(v); self }

    /// Chave de acesso de 44 dígitos da NF-e manifestada.
    pub fn chave(mut self, v: &str) -> Self { self.chave = Some(v.to_string()); self }

    /// Ciência da Operação (tpEvento 210210).
    pub async fn ciencia_operacao(self) -> Result<Response> {
        self.enviar(TP_EVENTO_CIENCIA_OPERACAO, "Ciencia da Operacao", None).await
    }

    /// Confirmação da Operação (tpEvento 210200).
    pub async fn confirmacao_operacao(self) -> Result<Response> {
        self.enviar(TP_EVENTO_CONFIRMACAO_OPERACAO, "Confirmacao da Operacao", None).await
    }

    /// Desconhecimento da Operação (tpEvento 210220).
    pub async fn desconhecimento_operacao(self) -> Result<Response> {
        self.enviar(TP_EVENTO_DESCONHECIMENTO_OPERACAO, "Desconhecimento da Operacao", None).await
    }

    /// Operação não Realizada (tpEvento 210240). A `justificativa` é obrigatória.
    pub async fn operacao_nao_realizada(self, justificativa: &str) -> Result<Response> {
        if justificativa.trim().is_empty() {
            return Err(DfeError::Validacao(
                "A justificativa e obrigatoria para Operacao nao Realizada".to_string(),
            ));
        }
        self.enviar(
            TP_EVENTO_OPERACAO_NAO_REALIZADA,
            "Operacao nao Realizada",
            Some(justificativa.to_string()),
        )
        .await
    }

    async fn enviar(
        self,
        tp_evento: &str,
        desc_evento: &str,
        justificativa: Option<String>,
    ) -> Result<Response> {
        let cert_path = self.cert_path.ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass = self.cert_pass.ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let cnpj      = self.cnpj.ok_or_else(|| DfeError::Validacao("cnpj não informado".to_string()))?;
        let tp_amb    = self.tp_amb.ok_or_else(|| DfeError::Configuracao("tp_amb não informado".to_string()))?;
        let chave     = self.chave.ok_or_else(|| DfeError::Validacao("chave não informada".to_string()))?;

        let lote_seq = 1u32;

        let inf_evento_xml =
            inf_evento_xml(&cnpj, tp_amb, &chave, tp_evento, desc_evento, justificativa.as_deref(), lote_seq)?;
        let inf_evento_xml = Strings::clear_xml_string(&inf_evento_xml);

        let digest_value = DigestValue::sha1(&inf_evento_xml)?;
        let signed_info  = signed_info_xml(&digest_value, tp_evento, &chave, lote_seq)?;

        let signature_base64 = Sign::xml_string(&signed_info, &cert_path, &cert_pass).await?;
        let x509_cert        = RawPubKey::get_from_file(&cert_path, &cert_pass).await?;
        let signature        = signature_xml(&signed_info, &signature_base64, &x509_cert)?;
        let envelope         = crate::interno::evento::env_evento_xml(&inf_evento_xml, &signature)?;
        let envelope         = Strings::clear_xml_string(&envelope);

        // Manifestação usa sempre o RecepcaoEvento do Ambiente Nacional (independe de UF/modelo).
        let url = recepcao_evento_ambiente_nacional(tp_amb)?;

        let send_envelope = envelope.clone();
        let response = MtlsTransport::new(&cert_path, &cert_pass)
            .send_soap(url, envelope)
            .await?;

        let parsed: InfEvento = crate::interno::evento::parse_ret_evento(&response)?;
        Ok(Response {
            response: parsed,
            send_xml: send_envelope,
            receive_xml: response,
        })
    }
}

impl Default for ManifestacaoBuilder {
    fn default() -> Self { Self::new() }
}

// ─── Lógica interna ───────────────────────────────────────────────────────────

fn inf_evento_xml(
    cnpj: &str,
    tp_amb: u8,
    chave: &str,
    tp_evento: &str,
    desc_evento: &str,
    justificativa: Option<&str>,
    lote_seq: u32,
) -> Result<String> {
    // A4: delega à montagem genérica em `interno::evento`. Para manifestação, cOrgao = 91
    // (Ambiente Nacional) e o CNPJ vem do payload; detEvento = descEvento + xJust opcional.
    let campos: Vec<(&str, &str)> = match justificativa {
        Some(j) => vec![("xJust", j)],
        None => vec![],
    };
    crate::interno::evento::inf_evento_xml(&crate::interno::evento::InfEvento {
        c_orgao: C_ORGAO_AMBIENTE_NACIONAL,
        tp_amb,
        cnpj,
        chave,
        tp_evento,
        n_seq_evento: lote_seq,
        ver_evento: VER_EVENTO,
        desc_evento,
        det_campos: &campos,
    })
}

fn recepcao_evento_ambiente_nacional(tp_amb: u8) -> Result<&'static str> {
    match tp_amb {
        1 => Ok("https://www1.nfe.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx"),
        2 => Ok("https://hom1.nfe.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx"),
        _ => Err(DfeError::Validacao(
            "tpAmb invalido para RecepcaoEvento no Ambiente Nacional".to_string(),
        )),
    }
}

fn signed_info_xml(digest_: &str, tp_evento: &str, chave: &str, lote_seq: u32) -> Result<String> {
    // A3b: delega à montagem unificada em `interno::assinatura`.
    let reference_uri = format!("#ID{}{}{:>02}", tp_evento, chave, lote_seq);
    Ok(crate::interno::assinatura::signed_info_xml(&reference_uri, digest_))
}

fn signature_xml(signe_info: &str, signed_value: &str, certificate: &str) -> Result<String> {
    // A3b: delega à montagem unificada em `interno::assinatura`.
    Ok(crate::interno::assinatura::signature_xml(signe_info, signed_value, certificate))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAVE: &str = "35000000000000000000550010000000001000000001";
    const CNPJ: &str = "11222333000181";

    // Golden capturado do output original (impl Writer); após a unificação em interno::assinatura
    // (A3b) e a migração para o builder (A4b) deve permanecer idêntico — prova de byte-identidade.
    #[test]
    fn golden_signed_info_xml() {
        let out = signed_info_xml("RGlnZXN0Rml4bw==", "210200", CHAVE, 1).unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn golden_signature_xml() {
        let out = signature_xml("<SignedInfo>FIXO</SignedInfo>", "U2lnRml4bw==", "Q2VydEZpeG8=").unwrap();
        insta::assert_snapshot!(out);
    }

    // Golden do infEvento (via interno::evento, A4a); dhEvento (volátil) é redigido.
    #[test]
    fn golden_inf_evento_sem_just() {
        let out = inf_evento_xml(CNPJ, 2, CHAVE, "210210", "Ciencia da Operacao", None, 1).unwrap();
        insta::with_settings!({filters => vec![(r"<dhEvento>.*?</dhEvento>", "<dhEvento>[DH]</dhEvento>")]}, {
            insta::assert_snapshot!(out);
        });
    }

    #[test]
    fn golden_inf_evento_com_just() {
        let out = inf_evento_xml(
            CNPJ,
            2,
            CHAVE,
            "210240",
            "Operacao nao Realizada",
            Some("Mercadoria nao recebida para teste"),
            1,
        )
        .unwrap();
        insta::with_settings!({filters => vec![(r"<dhEvento>.*?</dhEvento>", "<dhEvento>[DH]</dhEvento>")]}, {
            insta::assert_snapshot!(out);
        });
    }
}
