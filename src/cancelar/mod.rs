use crate::error::{DfeError, Result};
use crate::interno::cert::{Cert, DigestValue, RawPubKey, Sign};
use crate::interno::chave_acesso::ChaveAcesso;
use crate::interno::cleaner::Strings;
use crate::interno::connection::WebService;
use crate::interno::dates::get_current_date_time;
use crate::interno::ws::nfe_recepcao_evento;
use crate::tipos::cancelar::{InfEvento, Response};
use quick_xml::de;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;

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
    let envelope         = envelope_xml(&inf_evento_xml, &signature)?;
    let envelope         = Strings::clear_xml_string(&envelope);

    let mut file = File::create("cancelar.xml")?;
    file.write_all(envelope.as_bytes())?;

    let url    = nfe_recepcao_evento(tp_amb, "SP", mod_, false)?;
    let cert   = Cert::from_pfx(&cert_path, &cert_pass)?;
    let client = WebService::client(cert.identity)?;

    let send_envelope = envelope.clone();
    let response = client
        .post(url)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .header("Content-Length", envelope.len().to_string())
        .body(envelope)
        .send()
        .await?;

    let response = response.text().await?;

    let mut file = File::create("cancelar_response.xml")?;
    file.write_all(response.as_bytes())?;

    let re = regex::bytes::Regex::new(r"(?s)<infEvento.*?</infEvento>").unwrap();
    match re.captures(response.as_bytes()) {
        Some(captures) => {
            let inf_evento = captures.get(0)
                .map_or("", |m| std::str::from_utf8(m.as_bytes()).unwrap());
            match de::from_str::<InfEvento>(inf_evento) {
                Ok(parsed) => Ok(Response { response: parsed, send_xml: send_envelope, receive_xml: response }),
                Err(_) => Err(DfeError::Xml("Erro ao converter xml para struct".to_string())),
            }
        }
        None => Err(DfeError::Xml("Erro ao capturar infEvento".to_string())),
    }
}

fn inf_evento_xml(chave: &str, tp_amb: u8, protocolo: &str, justificativa: &str) -> Result<String> {
    let lote_seq      = 1u32;
    let inf_evento_id = format!("ID{}{}{:>02}", TP_EVENTO, chave, lote_seq);
    let chave_comp    = ChaveAcesso::extract_composition(chave)
        .map_err(|e| DfeError::Xml(e.to_string()))?;
    let c_orgao       = &chave_comp.uf_code;
    let tp_amb_str    = tp_amb.to_string();
    let dh_evento     = get_current_date_time();
    let n_seq_evento  = lote_seq.to_string();
    let ver_evento    = "1.00";

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.create_element("infEvento")
        .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
        .with_attribute(("Id", inf_evento_id.as_str()))
        .write_inner_content(|w| {
            w.create_element("cOrgao")   .write_text_content(BytesText::new(c_orgao))?;
            w.create_element("tpAmb")    .write_text_content(BytesText::new(&tp_amb_str))?;
            w.create_element("CNPJ")     .write_text_content(BytesText::new(&chave_comp.doc))?;
            w.create_element("chNFe")    .write_text_content(BytesText::new(chave))?;
            w.create_element("dhEvento") .write_text_content(BytesText::new(&dh_evento))?;
            w.create_element("tpEvento") .write_text_content(BytesText::new(TP_EVENTO))?;
            w.create_element("nSeqEvento").write_text_content(BytesText::new(&n_seq_evento))?;
            w.create_element("verEvento").write_text_content(BytesText::new(ver_evento))?;
            w.create_element("detEvento")
                .with_attribute(("versao", ver_evento))
                .write_inner_content(|w| {
                    w.create_element("descEvento").write_text_content(BytesText::new("Cancelamento"))?;
                    w.create_element("nProt")     .write_text_content(BytesText::new(protocolo))?;
                    w.create_element("xJust")     .write_text_content(BytesText::new(justificativa))?;
                    Ok(())
                })?;
            Ok(())
        })?;

    let xml = String::from_utf8(writer.into_inner().into_inner())?;
    let mut f = File::create("inf_evento.xml")?;
    f.write_all(xml.as_bytes())?;
    Ok(xml)
}

fn signed_info_xml(digest: &str, chave: &str) -> Result<String> {
    let lote_seq     = 1u32;
    let reference_uri = format!("#ID{}{}{:>02}", TP_EVENTO, chave, lote_seq);

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.create_element("SignedInfo")
        .with_attribute(("xmlns", "http://www.w3.org/2000/09/xmldsig#"))
        .write_inner_content(|w| {
            w.create_element("CanonicalizationMethod")
                .with_attribute(("Algorithm", "http://www.w3.org/TR/2001/REC-xml-c14n-20010315"))
                .write_inner_content(|_| Ok(()))?;
            w.create_element("SignatureMethod")
                .with_attribute(("Algorithm", "http://www.w3.org/2000/09/xmldsig#rsa-sha1"))
                .write_inner_content(|_| Ok(()))?;
            w.create_element("Reference")
                .with_attribute(("URI", reference_uri.as_str()))
                .write_inner_content(|w| {
                    w.create_element("Transforms").write_inner_content(|w| {
                        w.create_element("Transform")
                            .with_attribute(("Algorithm", "http://www.w3.org/2000/09/xmldsig#enveloped-signature"))
                            .write_inner_content(|_| Ok(()))?;
                        w.create_element("Transform")
                            .with_attribute(("Algorithm", "http://www.w3.org/TR/2001/REC-xml-c14n-20010315"))
                            .write_inner_content(|_| Ok(()))?;
                        Ok(())
                    })?;
                    w.create_element("DigestMethod")
                        .with_attribute(("Algorithm", "http://www.w3.org/2000/09/xmldsig#sha1"))
                        .write_inner_content(|_| Ok(()))?;
                    w.create_element("DigestValue").write_text_content(BytesText::new(digest))?;
                    Ok(())
                })?;
            Ok(())
        })?;

    let xml = String::from_utf8(writer.into_inner().into_inner())?;
    Ok(Strings::clear_xml_string(&xml))
}

fn signature_xml(signed_info: &str, signed_value: &str, certificate: &str) -> Result<String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.create_element("Signature")
        .with_attribute(("xmlns", "http://www.w3.org/2000/09/xmldsig#"))
        .write_inner_content(|w| {
            w.create_element("SIGNED_INFO_REPLACER").write_empty()?;
            w.create_element("SignatureValue").write_text_content(BytesText::new(signed_value))?;
            w.create_element("KeyInfo").write_inner_content(|w| {
                w.create_element("X509Data").write_inner_content(|w| {
                    w.create_element("X509Certificate").write_text_content(BytesText::new(certificate))?;
                    Ok(())
                })?;
                Ok(())
            })?;
            Ok(())
        })?;
    let xml = String::from_utf8(writer.into_inner().into_inner())?;
    Ok(xml.replace("<SIGNED_INFO_REPLACER/>", signed_info))
}

fn envelope_xml(inf_evento: &str, signature: &str) -> Result<String> {
    let lote_id = {
        let date = chrono::Local::now();
        format!("{}{}", date.format("%Y%m%d%H%M%S"), rand::random::<u8>() % 10)
    };

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer.create_element("soap12:Envelope")
        .with_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"))
        .with_attribute(("xmlns:xsd", "http://www.w3.org/2001/XMLSchema"))
        .with_attribute(("xmlns:soap12", "http://www.w3.org/2003/05/soap-envelope"))
        .write_inner_content(|w| {
            w.create_element("soap12:Body").write_inner_content(|w| {
                w.create_element("nfeDadosMsg")
                    .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4"))
                    .write_inner_content(|w| {
                        w.create_element("envEvento")
                            .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
                            .with_attribute(("versao", "1.00"))
                            .write_inner_content(|w| {
                                w.create_element("idLote").write_text_content(BytesText::new(&lote_id))?;
                                w.create_element("evento")
                                    .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
                                    .with_attribute(("versao", "1.00"))
                                    .write_inner_content(|w| {
                                        w.create_element("REPLACER_INF_EVENTO").write_empty()?;
                                        w.create_element("REPLACER_SIGNATURE").write_empty()?;
                                        Ok(())
                                    })?;
                                Ok(())
                            })?;
                        Ok(())
                    })?;
                Ok(())
            })?;
            Ok(())
        })?;

    let xml = String::from_utf8(writer.into_inner().into_inner())?;
    let xml = xml
        .replace("<REPLACER_INF_EVENTO/>", inf_evento)
        .replace("<REPLACER_SIGNATURE/>", signature);
    Ok("<?xml version=\"1.0\" encoding=\"utf-8\"?>".to_string() + &xml)
}
