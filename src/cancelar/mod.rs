use crate::error::{DfeError, Result};
use crate::interno::cert::{DigestValue, RawPubKey, Sign};
use crate::interno::chave_acesso::ChaveAcesso;
use crate::interno::cleaner::Strings;
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::interno::ws::nfe_recepcao_evento;
use crate::tipos::cancelar::{InfEvento, Response};

const TP_EVENTO: &str = "110111";

// ─── Builder público ──────────────────────────────────────────────────────────

pub struct CancelarBuilder {
    cert_path:    Option<String>,
    cert_pass:    Option<String>,
    tp_amb:       Option<u8>,
    mod_:         Option<u32>,
    chave:        Option<String>,
    protocolo:    Option<String>,
    justificativa: Option<String>,
}

impl CancelarBuilder {
    pub fn new() -> Self {
        Self {
            cert_path: None, cert_pass: None, tp_amb: None, mod_: None,
            chave: None, protocolo: None, justificativa: None,
        }
    }

    pub fn cert(mut self, path: &str, pass: &str) -> Self {
        self.cert_path = Some(path.to_string());
        self.cert_pass = Some(pass.to_string());
        self
    }

    /// 1 = Produção | 2 = Homologação
    pub fn tp_amb(mut self, v: u8) -> Self { self.tp_amb = Some(v); self }

    /// Modelo do documento: 55 = NF-e | 65 = NFC-e (padrão: 55)
    pub fn mod_(mut self, v: u32) -> Self { self.mod_ = Some(v); self }

    /// Chave de acesso de 44 dígitos
    pub fn chave(mut self, v: &str) -> Self { self.chave = Some(v.to_string()); self }

    /// Número do protocolo de autorização
    pub fn protocolo(mut self, v: &str) -> Self { self.protocolo = Some(v.to_string()); self }

    /// Justificativa do cancelamento (mínimo 15 caracteres)
    pub fn justificativa(mut self, v: &str) -> Self { self.justificativa = Some(v.to_string()); self }

    pub async fn send(self) -> Result<Response> {
        let cert_path     = self.cert_path    .ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass     = self.cert_pass    .ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let tp_amb        = self.tp_amb       .ok_or_else(|| DfeError::Configuracao("tp_amb não informado".to_string()))?;
        let chave         = self.chave        .ok_or_else(|| DfeError::Validacao("chave não informada".to_string()))?;
        let protocolo     = self.protocolo    .ok_or_else(|| DfeError::Validacao("protocolo não informado".to_string()))?;
        let justificativa = self.justificativa.ok_or_else(|| DfeError::Validacao("justificativa não informada".to_string()))?;
        let mod_          = self.mod_.unwrap_or(55);

        if justificativa.len() < 15 {
            return Err(DfeError::Validacao("justificativa deve ter no mínimo 15 caracteres".to_string()));
        }

        cancelar_nfe(cert_path, cert_pass, tp_amb, mod_, chave, protocolo, justificativa).await
    }
}

// ─── Lógica interna ───────────────────────────────────────────────────────────

async fn cancelar_nfe(
    cert_path: String, cert_pass: String,
    tp_amb: u8, mod_: u32,
    chave: String, protocolo: String, justificativa: String,
) -> Result<Response> {
    let inf_evento_xml = inf_evento_xml(&chave, tp_amb, &protocolo, &justificativa)?;
    let inf_evento_xml = Strings::clear_xml_string(&inf_evento_xml);

    let digest_value = DigestValue::sha1(&inf_evento_xml)?;
    let signed_info  = signed_info_xml(&digest_value, &chave)?;

    let signature_base64 = Sign::xml_string(&signed_info, &cert_path, &cert_pass).await?;
    let x509_cert        = RawPubKey::get_from_file(&cert_path, &cert_pass).await?;
    let signature        = signature_xml(&signed_info, &signature_base64, &x509_cert)?;
    let envelope         = crate::interno::evento::env_evento_xml(&inf_evento_xml, &signature)?;
    let envelope         = Strings::clear_xml_string(&envelope);

    // A6: UF real derivada da chave (2 primeiros dígitos = cUF), antes fixo em "SP".
    let uf = crate::interno::uf::sigla_por_codigo(chave.get(0..2).unwrap_or(""))?;
    let url = nfe_recepcao_evento(tp_amb, uf, mod_, false)?;

    let send_envelope = envelope.clone();
    let response = MtlsTransport::new(&cert_path, &cert_pass)
        .send_soap(url, envelope)
        .await?;

    let parsed: InfEvento = crate::interno::evento::parse_ret_evento(&response)?;
    Ok(Response { response: parsed, send_xml: send_envelope, receive_xml: response })
}

fn inf_evento_xml(chave: &str, tp_amb: u8, protocolo: &str, justificativa: &str) -> Result<String> {
    // A4: delega à montagem genérica em `interno::evento`. cOrgao/CNPJ vêm da chave;
    // detEvento de cancelamento = descEvento + nProt + xJust.
    let comp = ChaveAcesso::extract_composition(chave).map_err(|e| DfeError::Xml(e.to_string()))?;
    crate::interno::evento::inf_evento_xml(&crate::interno::evento::InfEvento {
        c_orgao: &comp.uf_code,
        tp_amb,
        cnpj: &comp.doc,
        chave,
        tp_evento: TP_EVENTO,
        n_seq_evento: 1,
        ver_evento: "1.00",
        desc_evento: "Cancelamento",
        det_campos: &[("nProt", protocolo), ("xJust", justificativa)],
    })
}

fn signed_info_xml(digest: &str, chave: &str) -> Result<String> {
    // A3b: delega à montagem unificada em `interno::assinatura` (antes duplicada aqui).
    let lote_seq = 1u32;
    let reference_uri = format!("#ID{}{}{:>02}", TP_EVENTO, chave, lote_seq);
    Ok(crate::interno::assinatura::signed_info_xml(&reference_uri, digest))
}

fn signature_xml(signed_info: &str, signed_value: &str, certificate: &str) -> Result<String> {
    // A3b: delega à montagem unificada em `interno::assinatura`.
    Ok(crate::interno::assinatura::signature_xml(signed_info, signed_value, certificate))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Chave de acesso fixa (modelo 55) para caracterização determinística das funções de
    // montagem do XML de evento — exatamente o que a fase A3b (interno::assinatura) vai unificar.
    const CHAVE: &str = "35000000000000000000550010000000001000000001";

    #[test]
    fn golden_signed_info_xml() {
        let out = signed_info_xml("RGlnZXN0VmFsdWVGaXhv", CHAVE).unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn golden_signature_xml() {
        let out = signature_xml(
            "<SignedInfo>CONTEUDO_FIXO</SignedInfo>",
            "U2lnbmF0dXJlVmFsdWVGaXhv",
            "Q2VydGlmaWNhZG9EZXJGaXhv",
        )
        .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn golden_inf_evento_xml() {
        // dhEvento vem de get_current_date_time() (volátil) → redigido no golden.
        let out = inf_evento_xml(
            CHAVE,
            2,
            "135000000000001",
            "Cancelamento de teste para caracterizacao offline",
        )
        .unwrap();
        insta::with_settings!({filters => vec![
            (r"<dhEvento>.*?</dhEvento>", "<dhEvento>[DH]</dhEvento>"),
        ]}, {
            insta::assert_snapshot!(out);
        });
    }
    // O golden do envelope envEvento migrou para `interno::evento::tests::golden_env_evento_xml`
    // (envelope unificado na fase de eventos).
}
