use crate::nfe::common::cert::Cert;
use crate::nfe::common::ws::nfe_recepcao_evento;
use crate::nfe::connection::WebService;
use crate::nfe::types::cancelar::*;
use anyhow::{Error, Result};
use quick_xml::de;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use quick_xml::Error as XmlError;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;

const TP_EVENTO: &str = "110111";

pub async fn nfe_cancelar(cancelar: NFeCancelar) -> Result<Response, Error> {
    // generate infEvento tag ----------------------------------------------------------------
    let inf_evento_xml = inf_evento_xml(cancelar.clone())?;
    let inf_evento_xml =
        crate::nfe::common::cleaner::Strings::clear_xml_string(inf_evento_xml.as_str());

    // generate digest value and signed_info tag ---------------------------------------------
    let digest_value = crate::nfe::common::cert::DigestValue::sha1(&inf_evento_xml)?;
    let signed_info = signed_info_xml(&digest_value, &cancelar.chave)?;

    // generate signature base64 -------------------------------------------
    let signature_base64 = crate::nfe::common::cert::Sign::xml_string(
        &signed_info,
        &cancelar.cert_path,
        &cancelar.cert_pass,
    )
    .await?;

    // generate x509 certificate clean begin and end and signature tag -----------------------
    let x509_cert = crate::nfe::common::cert::RawPubKey::get_from_file(
        &cancelar.cert_path,
        &cancelar.cert_pass,
    )
    .await?;

    // generate signature tag ----------------------------------------------------------------
    let signature = signature_xml(&signed_info, &signature_base64, &x509_cert)?;

    // generate envelope tag -----------------------------------------------------------------
    let envelope = envelope(&inf_evento_xml, &signature)?;
    let envelope = crate::nfe::common::cleaner::Strings::clear_xml_string(envelope.as_str());

    // create file with xml content
    let mut file = File::create("cancelar.xml")?;
    file.write_all(envelope.as_bytes())?;

    // Selecionar o url do webservice -----------------------
    let url = nfe_recepcao_evento(cancelar.tp_amb, "SP", 55, false)?;

    let cert = Cert::from_pfx(&cancelar.cert_path, &cancelar.cert_pass)?;
    let client = WebService::client(cert.identity)?;

    let content_length = envelope.len();

    let send_envelope = envelope.clone();

    let response = client
        .post(url)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .header("Content-Length", content_length.to_string())
        .body(envelope)
        .send()
        .await?;

    let response = response.text().await?;

    let re = regex::bytes::Regex::new(r"(?s)<infEvento.*?</infEvento>").unwrap();
    let ret_evento = re.captures(response.as_bytes());
    if let Some(captures) = ret_evento {
        let inf_evento = captures
            .get(0)
            .map_or("", |m| std::str::from_utf8(m.as_bytes()).unwrap());

        let ret_evento: std::result::Result<InfEvento, de::DeError> = de::from_str(inf_evento);
        if ret_evento.is_ok() {
            let ret_evento = ret_evento.unwrap();
            let response = Response {
                response: ret_evento,
                send_xml: send_envelope,
                receive_xml: response,
            };
            return Ok(response);
        } else {
            return Err(Error::msg("Erro ao converter xml para struct"));
        }
    } else {
        return Err(Error::msg("Erro ao capturar infEvento"));
    }
}

fn inf_evento_xml(cancelar: NFeCancelar) -> Result<String, XmlError> {
    // Sequencial do lote de eventos que deve ser autoincrementado
    // TODO: implementar autoincremento
    let lote_seq = lote_seq_generate();
    let inf_evento_id = format!("ID{}{}{:>02}", TP_EVENTO, cancelar.chave, lote_seq);

    let chave_composition =
        crate::nfe::common::chave_acesso::ChaveAcesso::extract_composition(&cancelar.chave)
            .map_err(|e| {
                XmlError::Io(std::sync::Arc::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )))
            })?;
    let c_orgao = &chave_composition.uf_code;
    let tp_amb = &cancelar.tp_amb.to_string();
    let doc = &chave_composition.doc;
    let ch_nfe = &cancelar.chave;
    let dh_evento = crate::nfe::common::dates::get_current_date_time();
    let dh_evento = dh_evento.as_str();
    let n_seq_evento = lote_seq.to_string();
    let n_seq_evento = n_seq_evento.as_str();
    let ver_evento = "1.00";
    let desc_evento = "Cancelamento";
    let n_prot = &cancelar.protocolo;
    let x_just = &cancelar.justificativa;

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .create_element("infEvento")
        .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
        .with_attribute(("Id", inf_evento_id.as_str())) // add an attribute
        .write_inner_content(|writer| {
            writer
                .create_element("cOrgao")
                .write_text_content(BytesText::new(c_orgao))?;
            writer
                .create_element("tpAmb")
                .write_text_content(BytesText::new(tp_amb))?;
            writer
                .create_element("CNPJ")
                .write_text_content(BytesText::new(doc))?;
            writer
                .create_element("chNFe")
                .write_text_content(BytesText::new(ch_nfe))?;
            writer
                .create_element("dhEvento")
                .write_text_content(BytesText::new(dh_evento))?;
            writer
                .create_element("tpEvento")
                .write_text_content(BytesText::new(TP_EVENTO))?;
            writer
                .create_element("nSeqEvento")
                .write_text_content(BytesText::new(n_seq_evento))?;
            writer
                .create_element("verEvento")
                .write_text_content(BytesText::new(ver_evento))?;
            writer
                .create_element("detEvento")
                .with_attribute(("versao", ver_evento))
                .write_inner_content(|writer| {
                    writer
                        .create_element("descEvento")
                        .write_text_content(BytesText::new(desc_evento))?;
                    writer
                        .create_element("nProt")
                        .write_text_content(BytesText::new(n_prot))?;
                    writer
                        .create_element("xJust")
                        .write_text_content(BytesText::new(x_just))?;
                    Ok(())
                })?;
            Ok(())
        })?;

    let string = writer.into_inner().into_inner();
    let string = String::from_utf8(string).unwrap();
    let mut file = File::create("inf_evento.xml")?;
    file.write_all(string.as_bytes())?;

    Ok(string)
}

fn signed_info_xml(digest_: &str, chave: &str) -> Result<String, XmlError> {
    // Sequencial do lote de eventos que deve ser autoincrementado
    let lote_seq = lote_seq_generate();
    let reference_uri = format!("#ID{}{}{:>02}", TP_EVENTO, chave, lote_seq);
    let reference_uri = reference_uri.as_str();

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .create_element("SignedInfo")
        .with_attribute(("xmlns", "http://www.w3.org/2000/09/xmldsig#"))
        .write_inner_content(|writer| {
            writer
                .create_element("CanonicalizationMethod")
                .with_attribute((
                    "Algorithm",
                    "http://www.w3.org/TR/2001/REC-xml-c14n-20010315",
                ))
                .write_inner_content(|_writer| Ok(()))?;
            writer
                .create_element("SignatureMethod")
                .with_attribute(("Algorithm", "http://www.w3.org/2000/09/xmldsig#rsa-sha1"))
                .write_inner_content(|_writer| Ok(()))?;
            writer
                .create_element("Reference")
                .with_attribute(("URI", reference_uri))
                .write_inner_content(|writer| {
                    writer
                        .create_element("Transforms")
                        .write_inner_content(|writer| {
                            writer
                                .create_element("Transform")
                                .with_attribute((
                                    "Algorithm",
                                    "http://www.w3.org/2000/09/xmldsig#enveloped-signature",
                                ))
                                .write_inner_content(|_writer| Ok(()))?;
                            writer
                                .create_element("Transform")
                                .with_attribute((
                                    "Algorithm",
                                    "http://www.w3.org/TR/2001/REC-xml-c14n-20010315",
                                ))
                                .write_inner_content(|_writer| Ok(()))?;
                            Ok(())
                        })?;
                    writer
                        .create_element("DigestMethod")
                        .with_attribute(("Algorithm", "http://www.w3.org/2000/09/xmldsig#sha1"))
                        .write_inner_content(|_writer| Ok(()))?;
                    writer
                        .create_element("DigestValue")
                        .write_text_content(BytesText::new(digest_))?;
                    Ok(())
                })?;
            Ok(())
        })?;

    let string = writer.into_inner().into_inner();
    let string = String::from_utf8(string).unwrap();
    let string = crate::nfe::common::cleaner::Strings::clear_xml_string(string.as_str());
    Ok(string)
}

fn signature_xml(
    signe_info: &str,
    signed_value: &str,
    certificate: &str,
) -> Result<String, XmlError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .create_element("Signature")
        .with_attribute(("xmlns", "http://www.w3.org/2000/09/xmldsig#"))
        .write_inner_content(|writer| {
            writer
                .create_element("SIGNED_INFO_REPLACER")
                .write_empty()?;
            writer
                .create_element("SignatureValue")
                .write_text_content(BytesText::new(signed_value))?;
            writer
                .create_element("KeyInfo")
                .write_inner_content(|writer| {
                    writer
                        .create_element("X509Data")
                        .write_inner_content(|writer| {
                            writer
                                .create_element("X509Certificate")
                                .write_text_content(BytesText::new(certificate))?;
                            Ok(())
                        })?;
                    Ok(())
                })?;
            Ok(())
        })?;

    let string = writer.into_inner().into_inner();
    let string = String::from_utf8(string).unwrap();
    let string = string.replace("<SIGNED_INFO_REPLACER/>", signe_info);
    Ok(string)
}

fn envelope(inf_evento: &str, signature: &str) -> Result<String, XmlError> {
    let lote_id = id_lote_generate();
    let lote_id = lote_id.as_str();
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    writer
        .create_element("soap12:Envelope")
        .with_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"))
        .with_attribute(("xmlns:xsd", "http://www.w3.org/2001/XMLSchema"))
        .with_attribute(("xmlns:soap12", "http://www.w3.org/2003/05/soap-envelope"))
        .write_inner_content(|writer| {
            writer
                .create_element("soap12:Body")
                .write_inner_content(|writer| {
                    writer
                        .create_element("nfeDadosMsg")
                        .with_attribute((
                            "xmlns",
                            "http://www.portalfiscal.inf.br/nfe/wsdl/NFeRecepcaoEvento4",
                        ))
                        .write_inner_content(|writer| {
                            writer
                                .create_element("envEvento")
                                .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
                                .with_attribute(("versao", "1.00"))
                                .write_inner_content(|writer| {
                                    writer
                                        .create_element("idLote")
                                        .write_text_content(BytesText::new(lote_id))?;
                                    writer
                                        .create_element("evento")
                                        .with_attribute((
                                            "xmlns",
                                            "http://www.portalfiscal.inf.br/nfe",
                                        ))
                                        .with_attribute(("versao", "1.00"))
                                        .write_inner_content(|writer| {
                                            writer
                                                .create_element("REPLACER_INF_EVENTO")
                                                .write_empty()?;
                                            writer
                                                .create_element("REPLACER_SIGNATURE")
                                                .write_empty()?;
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

    let string = writer.into_inner().into_inner();
    let string = String::from_utf8(string).unwrap();
    let string = string.replace("<REPLACER_INF_EVENTO/>", inf_evento);
    let string = string.replace("<REPLACER_SIGNATURE/>", signature);
    let string = "<?xml version=\"1.0\" encoding=\"utf-8\"?>".to_string() + &string;
    Ok(string)
}

fn lote_seq_generate() -> u32 {
    1
}

fn id_lote_generate() -> String {
    let date = chrono::Local::now();
    let date = date.format("%Y%m%d%H%M%S").to_string();
    let random = rand::random::<u8>() % 10;
    let id = format!("{}{}", date, random);
    id
}
