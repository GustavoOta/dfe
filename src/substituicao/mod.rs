//! Cancelamento por substituição de NFC-e — evento **tpEvento 110112** (NT 2018.004).
//!
//! Usado quando **duas NFC-e representam a mesma venda** (ex.: uma emitida normalmente e outra
//! em contingência offline): cancela a **duplicada** e **mantém a substituta válida**. Não é
//! devolução. Aplica-se **somente a NFC-e (modelo 65)** e o prazo é de 168h (7 dias).
//!
//! Builder fino (§2.1) sobre a espinha unificada de eventos: monta o `<infEvento>` via
//! [`crate::interno::evento`], assina via [`crate::interno::assinatura`], envelopa via
//! [`crate::interno::evento::env_evento_xml`] e envia via [`crate::interno::transporte`] —
//! exatamente como `cancelar` (evento 110111), diferindo apenas no `detEvento` e no `tpEvento`.
//!
//! # Campos do `detEvento` (ordem do schema `eventoCancSubst_v1.00.xsd`)
//! `descEvento` = "Cancelamento por substituicao" · `cOrgaoAutor` (UF da chave) · `tpAutor` = 1
//! (empresa emitente) · `verAplic` (software emissor) · `nProt` (protocolo da NFC-e **cancelada**)
//! · `xJust` (15–255) · `chNFeRef` (chave da NFC-e **substituta**, que permanece válida).
//!
//! ⚠️ **Semântica crítica:** `chave`/`protocolo` = a NFC-e **cancelada**; `chave_substituta` =
//! a que **fica válida**. Trocar isso cancela a nota errada.
//!
//! > **Roteamento (A6):** o endpoint é resolvido pela UF real da chave (`interno::uf` +
//! > `webservices.json`), cobrindo as 27 UFs (autorizadores próprios + SVAN/SVRS).

use crate::error::{DfeError, Result};
use crate::interno::cert::{DigestValue, RawPubKey, Sign};
use crate::interno::chave_acesso::ChaveAcesso;
use crate::interno::chave_acesso_props::ExtractComposition;
use crate::interno::cleaner::Strings;
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::interno::ws::nfe_recepcao_evento;
use crate::tipos::cancelar::{InfEvento, Response};

const TP_EVENTO: &str = "110112";
const DESC_EVENTO: &str = "Cancelamento por substituicao"; // sem acento, conforme a NT
const TP_AUTOR_EMPRESA_EMITENTE: &str = "1";
const VER_EVENTO: &str = "1.00";
const MODELO_NFCE: &str = "65";

// ─── Builder público ──────────────────────────────────────────────────────────

/// Cancelamento por substituição de NFC-e (evento 110112). Retorna
/// [`crate::tipos::cancelar::Response`] — o `retEvento` tem a mesma forma do cancelamento comum.
///
/// # Exemplo
/// ```no_run
/// use dfe::SubstituicaoBuilder;
/// # async fn ex() -> Result<(), dfe::DfeError> {
/// let resp = SubstituicaoBuilder::new()
///     .cert("./cert.pfx", "senha")
///     .tp_amb(2)
///     .chave("35...")            // NFC-e CANCELADA (44 dígitos, modelo 65)
///     .protocolo("135...")       // protocolo da NFC-e cancelada
///     .chave_substituta("35...") // NFC-e que permanece válida
///     .ver_aplic("MeuPDV-1.0")
///     .justificativa("Falha na conexao com a internet no momento da venda")
///     .send()
///     .await?;
/// println!("cStat: {}", resp.response.c_stat);
/// # Ok(()) }
/// ```
pub struct SubstituicaoBuilder {
    cert_path:        Option<String>,
    cert_pass:        Option<String>,
    tp_amb:           Option<u8>,
    chave:            Option<String>,
    protocolo:        Option<String>,
    chave_substituta: Option<String>,
    ver_aplic:        Option<String>,
    justificativa:    Option<String>,
}

impl SubstituicaoBuilder {
    pub fn new() -> Self {
        Self {
            cert_path: None, cert_pass: None, tp_amb: None, chave: None,
            protocolo: None, chave_substituta: None, ver_aplic: None, justificativa: None,
        }
    }

    /// Certificado A1 (`.pfx`) e senha.
    pub fn cert(mut self, path: &str, pass: &str) -> Self {
        self.cert_path = Some(path.to_string());
        self.cert_pass = Some(pass.to_string());
        self
    }

    /// 1 = Produção | 2 = Homologação
    pub fn tp_amb(mut self, v: u8) -> Self { self.tp_amb = Some(v); self }

    /// Chave de acesso (44 dígitos) da NFC-e que será **cancelada** (a duplicada).
    pub fn chave(mut self, v: &str) -> Self { self.chave = Some(v.to_string()); self }

    /// Número do protocolo de autorização da NFC-e **cancelada**.
    pub fn protocolo(mut self, v: &str) -> Self { self.protocolo = Some(v.to_string()); self }

    /// Chave de acesso (44 dígitos) da NFC-e **substituta** — a que permanece válida (`chNFeRef`).
    pub fn chave_substituta(mut self, v: &str) -> Self { self.chave_substituta = Some(v.to_string()); self }

    /// Nome/versão do software emissor (`verAplic`) — obrigatório pela NT.
    pub fn ver_aplic(mut self, v: &str) -> Self { self.ver_aplic = Some(v.to_string()); self }

    /// Justificativa do cancelamento (`xJust`, 15–255 caracteres).
    pub fn justificativa(mut self, v: &str) -> Self { self.justificativa = Some(v.to_string()); self }

    pub async fn send(self) -> Result<Response> {
        let cert_path = self.cert_path.ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass = self.cert_pass.ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let tp_amb    = self.tp_amb.ok_or_else(|| DfeError::Configuracao("tp_amb não informado".to_string()))?;
        let chave     = self.chave.ok_or_else(|| DfeError::Validacao("chave não informada".to_string()))?;
        let protocolo = self.protocolo.ok_or_else(|| DfeError::Validacao("protocolo não informado".to_string()))?;
        let chave_substituta = self.chave_substituta.ok_or_else(|| DfeError::Validacao("chave_substituta não informada".to_string()))?;
        let ver_aplic = self.ver_aplic.ok_or_else(|| DfeError::Validacao("ver_aplic não informado".to_string()))?;
        let justificativa = self.justificativa.ok_or_else(|| DfeError::Validacao("justificativa não informada".to_string()))?;

        // ── Validações da NT 2018.004 (antes de qualquer I/O) ──
        if chave.len() != 44 {
            return Err(DfeError::Validacao("chave deve ter 44 dígitos".to_string()));
        }
        if ver_aplic.trim().is_empty() {
            return Err(DfeError::Validacao("ver_aplic (verAplic) é obrigatório".to_string()));
        }
        // NT 2018.004: xJust 15–255.
        let justificativa = justificativa.trim().to_string();
        let just_len = justificativa.chars().count();
        if just_len < 15 {
            return Err(DfeError::Validacao("justificativa deve ter no mínimo 15 caracteres".to_string()));
        }
        if just_len > 255 {
            return Err(DfeError::Validacao("justificativa deve ter no máximo 255 caracteres".to_string()));
        }

        let comp = ChaveAcesso::extract_composition(&chave)?;
        // NT 2018.004: o evento 110112 aplica-se somente a NFC-e (modelo 65).
        if comp.modelo != MODELO_NFCE {
            return Err(DfeError::Validacao(
                "Cancelamento por substituição aplica-se somente a NFC-e (modelo 65)".to_string(),
            ));
        }

        substituir(cert_path, cert_pass, tp_amb, comp, chave, protocolo, chave_substituta, ver_aplic, justificativa).await
    }
}

impl Default for SubstituicaoBuilder {
    fn default() -> Self { Self::new() }
}

// ─── Lógica interna ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
async fn substituir(
    cert_path: String, cert_pass: String,
    tp_amb: u8, comp: ExtractComposition,
    chave: String, protocolo: String, chave_substituta: String,
    ver_aplic: String, justificativa: String,
) -> Result<Response> {
    let inf_evento_xml = inf_evento_xml(tp_amb, &comp, &chave, &protocolo, &chave_substituta, &ver_aplic, &justificativa)?;
    let inf_evento_xml = Strings::clear_xml_string(&inf_evento_xml);

    let digest_value = DigestValue::sha1(&inf_evento_xml)?;
    // nSeqEvento fixo em 1 para o cancelamento por substituição (NT 2018.004).
    let reference_uri = format!("#ID{}{}{:>02}", TP_EVENTO, chave, 1u32);
    let signed_info  = crate::interno::assinatura::signed_info_xml(&reference_uri, &digest_value);

    let signature_base64 = Sign::xml_string(&signed_info, &cert_path, &cert_pass).await?;
    let x509_cert        = RawPubKey::get_from_file(&cert_path, &cert_pass).await?;
    let signature        = crate::interno::assinatura::signature_xml(&signed_info, &signature_base64, &x509_cert);
    let envelope         = crate::interno::evento::env_evento_xml(&inf_evento_xml, &signature)?;
    let envelope         = Strings::clear_xml_string(&envelope);

    // A6: UF real derivada da chave (cOrgaoAutor = UF). Só NFC-e (modelo 65).
    let uf = crate::interno::uf::sigla_por_codigo(&comp.uf_code)?;
    let url = nfe_recepcao_evento(tp_amb, uf, 65, false)?;

    let send_envelope = envelope.clone();
    let response = MtlsTransport::new(&cert_path, &cert_pass)
        .send_soap(url, envelope)
        .await?;

    let parsed: InfEvento = crate::interno::evento::parse_ret_evento(&response)?;
    Ok(Response { response: parsed, send_xml: send_envelope, receive_xml: response })
}

fn inf_evento_xml(
    tp_amb: u8,
    comp: &ExtractComposition,
    chave: &str,
    protocolo: &str,
    chave_substituta: &str,
    ver_aplic: &str,
    justificativa: &str,
) -> Result<String> {
    // detEvento do 110112 (ordem do schema eventoCancSubst_v1.00.xsd). cOrgaoAutor = UF da chave
    // (mesma UF do cOrgao do infEvento — o evento vai para a SEFAZ autorizadora, não para o AN).
    let det_campos: Vec<(&str, &str)> = vec![
        ("cOrgaoAutor", &comp.uf_code),
        ("tpAutor", TP_AUTOR_EMPRESA_EMITENTE),
        ("verAplic", ver_aplic),
        ("nProt", protocolo),
        ("xJust", justificativa),
        ("chNFeRef", chave_substituta),
    ];
    crate::interno::evento::inf_evento_xml(&crate::interno::evento::InfEvento {
        c_orgao: &comp.uf_code,
        tp_amb,
        cnpj: &comp.doc,
        chave,
        tp_evento: TP_EVENTO,
        n_seq_evento: 1,
        ver_evento: VER_EVENTO,
        desc_evento: DESC_EVENTO,
        det_campos: &det_campos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chave de NFC-e (modelo 65) fixa para caracterização determinística do detEvento 110112.
    const CHAVE_NFCE: &str = "35000000000000000000650010000000001000000001";
    const CHAVE_SUBST: &str = "35000000000000000000650010000000009000000009";
    // Chave de NF-e (modelo 55) — deve ser rejeitada pelo evento 110112.
    const CHAVE_NFE: &str = "35000000000000000000550010000000001000000001";

    fn comp() -> ExtractComposition {
        ChaveAcesso::extract_composition(CHAVE_NFCE).unwrap()
    }

    // Golden do infEvento (nó assinado) do 110112. dhEvento (volátil) é redigido. Trava a ordem
    // e o conteúdo do detEvento conforme a NT 2018.004.
    #[test]
    fn golden_inf_evento_110112() {
        let out = inf_evento_xml(
            2,
            &comp(),
            CHAVE_NFCE,
            "135000000000001",
            CHAVE_SUBST,
            "TestApp-1.0",
            "Falha na conexao com a internet no momento da venda",
        )
        .unwrap();
        insta::with_settings!({filters => vec![(r"<dhEvento>.*?</dhEvento>", "<dhEvento>[DH]</dhEvento>")]}, {
            insta::assert_snapshot!(out);
        });
    }

    fn builder_valido() -> SubstituicaoBuilder {
        SubstituicaoBuilder::new()
            .cert("inexistente.pfx", "x")
            .tp_amb(2)
            .chave(CHAVE_NFCE)
            .protocolo("135000000000001")
            .chave_substituta(CHAVE_SUBST)
            .ver_aplic("TestApp-1.0")
            .justificativa("Falha na conexao com a internet no momento da venda")
    }

    // As validações abaixo devem falhar ANTES de qualquer I/O (cert/rede) — provam o fail-fast.
    #[tokio::test]
    async fn rejeita_modelo_nfe_55() {
        let err = builder_valido().chave(CHAVE_NFE).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_ver_aplic_vazio() {
        let err = builder_valido().ver_aplic("   ").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_justificativa_curta() {
        let err = builder_valido().justificativa("curta").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_chave_tamanho_invalido() {
        let err = builder_valido().chave("35065").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }
}
