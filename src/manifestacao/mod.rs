use crate::error::{DfeError, Result};
use crate::interno::cert::{Cert, DigestValue, Sign};
use crate::interno::connection::WebService;
use crate::tipos::manifestacao::{InfEvento, Manifestacao, OperacaoNaoRealizada, Response};
use chrono::Local;
use quick_xml::de;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::fs::{self, File};
use std::io::Cursor;
use std::io::Write;
use std::path::PathBuf;

const TP_EVENTO_CONFIRMACAO_OPERACAO: &str = "210200";
const TP_EVENTO_CIENCIA_OPERACAO: &str = "210210";
const TP_EVENTO_DESCONHECIMENTO_OPERACAO: &str = "210220";
const TP_EVENTO_OPERACAO_NAO_REALIZADA: &str = "210240";
const VER_EVENTO: &str = "1.00";
// Manifestacao do destinatario e processada pelo Ambiente Nacional.
const C_ORGAO_AMBIENTE_NACIONAL: &str = "91";
const MANIFESTACAO_LOG_REQUESTS_DIR: &str = "./distribuicao-logs/requests";
const MANIFESTACAO_LOG_RESPONSES_DIR: &str = "./distribuicao-logs/responses";
const MANIFESTACAO_LOG_ERRORS_DIR: &str = "./distribuicao-logs/errors";

struct ManifestacaoLogPaths {
    dir_requests: PathBuf,
    dir_responses: PathBuf,
    dir_errors: PathBuf,
    base_name: String,
}

fn manifestacao_log_paths(cnpj: &str, desc_evento: &str) -> ManifestacaoLogPaths {
    let now = Local::now();
    let cnpj_clean: String = cnpj.chars().filter(|c| c.is_alphanumeric()).collect();
    let ano = now.format("%Y").to_string();
    let mes = now.format("%m").to_string();
    let op_name = desc_evento.to_lowercase().replace(' ', "-");
    let time = now.format("%H-%M-%S").to_string();
    let suffix = [cnpj_clean.as_str(), ano.as_str(), mes.as_str()];
    let build = |base: &str| suffix.iter().fold(PathBuf::from(base), |p, s| p.join(s));
    ManifestacaoLogPaths {
        dir_requests: build(MANIFESTACAO_LOG_REQUESTS_DIR),
        dir_responses: build(MANIFESTACAO_LOG_RESPONSES_DIR),
        dir_errors: build(MANIFESTACAO_LOG_ERRORS_DIR),
        base_name: format!("{}-{}", op_name, time),
    }
}

pub async fn nfe_confirmacao_operacao(params: Manifestacao) -> Result<Response> {
    enviar_manifestacao(
        &params,
        TP_EVENTO_CONFIRMACAO_OPERACAO,
        "Confirmacao da Operacao",
        None,
    )
    .await
}

pub async fn nfe_ciencia_operacao(params: Manifestacao) -> Result<Response> {
    enviar_manifestacao(
        &params,
        TP_EVENTO_CIENCIA_OPERACAO,
        "Ciencia da Operacao",
        None,
    )
    .await
}

pub async fn nfe_desconhecimento_operacao(params: Manifestacao) -> Result<Response> {
    enviar_manifestacao(
        &params,
        TP_EVENTO_DESCONHECIMENTO_OPERACAO,
        "Desconhecimento da Operacao",
        None,
    )
    .await
}

pub async fn nfe_operacao_nao_realizada(params: OperacaoNaoRealizada) -> Result<Response> {
    if params.justificativa.trim().is_empty() {
        return Err(DfeError::Validacao(
            "A justificativa e obrigatoria para Operacao nao Realizada".to_string(),
        ));
    }

    let base = Manifestacao {
        cert_path: params.cert_path.clone(),
        cert_pass: params.cert_pass.clone(),
        cnpj: params.cnpj.clone(),
        tp_amb: params.tp_amb,
        mod_: params.mod_,
        chave: params.chave.clone(),
    };

    enviar_manifestacao(
        &base,
        TP_EVENTO_OPERACAO_NAO_REALIZADA,
        "Operacao nao Realizada",
        Some(params.justificativa.as_str()),
    )
    .await
}

async fn enviar_manifestacao(
    params: &Manifestacao,
    tp_evento: &str,
    desc_evento: &str,
    justificativa: Option<&str>,
) -> Result<Response> {
    let lote_seq = lote_seq_generate();

    let log_paths = manifestacao_log_paths(&params.cnpj, desc_evento);
    fs::create_dir_all(&log_paths.dir_requests)?;
    fs::create_dir_all(&log_paths.dir_responses)?;
    fs::create_dir_all(&log_paths.dir_errors)?;
    let path_inf_evento = log_paths
        .dir_requests
        .join(format!("{}-inf-evento.xml", log_paths.base_name));
    let path_envio = log_paths
        .dir_requests
        .join(format!("{}-envio.xml", log_paths.base_name));
    let path_resposta = log_paths
        .dir_responses
        .join(format!("{}-resposta.xml", log_paths.base_name));
    let path_erro = log_paths
        .dir_errors
        .join(format!("{}-erro.txt", log_paths.base_name));

    macro_rules! log_err {
        ($expr:expr, $msg:literal) => {
            $expr.map_err(|e| {
                let msg = format!("{}: {}", $msg, e);
                let _ = fs::write(&path_erro, &msg);
                DfeError::Webservice(msg)
            })?
        };
    }

    let inf_evento_xml = log_err!(
        inf_evento_xml(
            params,
            tp_evento,
            desc_evento,
            justificativa,
            lote_seq,
            &path_inf_evento
        ),
        "Erro ao gerar XML do infEvento"
    );
    let inf_evento_xml =
        crate::interno::cleaner::Strings::clear_xml_string(inf_evento_xml.as_str());

    let digest_value = log_err!(
        DigestValue::sha1(&inf_evento_xml),
        "Erro ao calcular DigestValue SHA1"
    );
    let signed_info = log_err!(
        signed_info_xml(&digest_value, tp_evento, &params.chave, lote_seq),
        "Erro ao gerar XML do SignedInfo"
    );

    let signature_base64 = log_err!(
        Sign::xml_string(
            &signed_info,
            &params.cert_path,
            &params.cert_pass
        )
        .await,
        "Erro ao assinar XML"
    );

    let x509_cert = log_err!(
        crate::interno::cert::RawPubKey::get_from_file(&params.cert_path, &params.cert_pass)
            .await,
        "Erro ao extrair chave publica do certificado"
    );

    let signature = log_err!(
        signature_xml(&signed_info, &signature_base64, &x509_cert),
        "Erro ao gerar XML da Signature"
    );
    let envelope_xml = log_err!(
        envelope(&inf_evento_xml, &signature),
        "Erro ao gerar envelope SOAP"
    );
    let envelope_xml =
        crate::interno::cleaner::Strings::clear_xml_string(envelope_xml.as_str());

    log_err!(
        File::create(&path_envio).and_then(|mut f| f.write_all(envelope_xml.as_bytes())),
        "Erro ao salvar XML de envio"
    );

    // Usa sempre o endpoint de RecepcaoEvento do Ambiente Nacional para tpEvento de manifestacao.
    let url = log_err!(
        recepcao_evento_ambiente_nacional(params.tp_amb),
        "Erro ao resolver URL do webservice"
    );

    let cert = log_err!(
        Cert::from_pfx(&params.cert_path, &params.cert_pass),
        "Erro ao carregar certificado PKCS12"
    );
    let client = log_err!(
        WebService::client(cert.identity),
        "Erro ao construir cliente HTTP"
    );

    let response = log_err!(
        client
            .post(url)
            .header("Content-Type", "application/soap+xml; charset=utf-8")
            .header("Content-Length", envelope_xml.len().to_string())
            .body(envelope_xml.clone())
            .send()
            .await,
        "Erro ao enviar requisicao ao webservice"
    );

    let status = response.status();
    let response = log_err!(response.text().await, "Erro ao ler corpo da resposta HTTP");

    if !status.is_success() {
        let msg = format!(
            "Falha HTTP ao enviar manifestacao para {}: status={} resposta_inicio={}",
            url,
            status,
            response.chars().take(220).collect::<String>()
        );
        let _ = fs::write(&path_erro, &msg);
        return Err(DfeError::Webservice(msg));
    }

    log_err!(
        File::create(&path_resposta).and_then(|mut f| f.write_all(response.as_bytes())),
        "Erro ao salvar XML de resposta"
    );

    let re = regex::bytes::Regex::new(r"(?s)<infEvento.*?</infEvento>").unwrap();
    let ret_evento = re.captures(response.as_bytes());
    if let Some(captures) = ret_evento {
        let inf_evento = captures
            .get(0)
            .map_or("", |m| std::str::from_utf8(m.as_bytes()).unwrap());

        let ret_evento: std::result::Result<InfEvento, de::DeError> = de::from_str(inf_evento);

        if let Ok(parsed) = ret_evento {
            return Ok(Response {
                response: parsed,
                send_xml: envelope_xml,
                receive_xml: response,
            });
        }

        let msg = "Erro ao converter xml para struct".to_string();
        let _ = fs::write(&path_erro, &msg);
        return Err(DfeError::Xml(msg));
    }

    let msg = format!(
        "Erro ao capturar infEvento. Resposta_inicio={}",
        response.chars().take(220).collect::<String>()
    );
    let _ = fs::write(&path_erro, &msg);
    Err(DfeError::Xml(msg))
}

fn inf_evento_xml(
    params: &Manifestacao,
    tp_evento: &str,
    desc_evento: &str,
    justificativa: Option<&str>,
    lote_seq: u32,
    log_path: &PathBuf,
) -> Result<String> {
    let inf_evento_id = format!("ID{}{}{:>02}", tp_evento, params.chave, lote_seq);

    // Para manifestacao, cOrgao deve ser 91 (Ambiente Nacional), independentemente da UF da chave.
    let c_orgao = C_ORGAO_AMBIENTE_NACIONAL;
    let tp_amb = &params.tp_amb.to_string();
    // O CNPJ do evento deve vir explicitamente do payload recebido.
    let doc = &params.cnpj;
    let ch_nfe = &params.chave;
    let dh_evento = crate::interno::dates::get_current_date_time();
    let dh_evento = dh_evento.as_str();
    let n_seq_evento = lote_seq.to_string();
    let n_seq_evento = n_seq_evento.as_str();

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .create_element("infEvento")
        .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
        .with_attribute(("Id", inf_evento_id.as_str()))
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
                .write_text_content(BytesText::new(tp_evento))?;
            writer
                .create_element("nSeqEvento")
                .write_text_content(BytesText::new(n_seq_evento))?;
            writer
                .create_element("verEvento")
                .write_text_content(BytesText::new(VER_EVENTO))?;
            writer
                .create_element("detEvento")
                .with_attribute(("versao", VER_EVENTO))
                .write_inner_content(|writer| {
                    writer
                        .create_element("descEvento")
                        .write_text_content(BytesText::new(desc_evento))?;

                    if let Some(justificativa) = justificativa {
                        writer
                            .create_element("xJust")
                            .write_text_content(BytesText::new(justificativa))?;
                    }

                    Ok(())
                })?;
            Ok(())
        })?;

    let string = writer.into_inner().into_inner();
    let string = String::from_utf8(string)?;

    let mut file = File::create(log_path)?;
    file.write_all(string.as_bytes())?;

    Ok(string)
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

fn signed_info_xml(
    digest_: &str,
    tp_evento: &str,
    chave: &str,
    lote_seq: u32,
) -> Result<String> {
    let reference_uri = format!("#ID{}{}{:>02}", tp_evento, chave, lote_seq);
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
    let string = String::from_utf8(string)?;
    let string = crate::interno::cleaner::Strings::clear_xml_string(string.as_str());
    Ok(string)
}

fn signature_xml(
    signe_info: &str,
    signed_value: &str,
    certificate: &str,
) -> Result<String> {
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
    let string = String::from_utf8(string)?;
    let string = string.replace("<SIGNED_INFO_REPLACER/>", signe_info);
    Ok(string)
}

fn envelope(inf_evento: &str, signature: &str) -> Result<String> {
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
    let string = String::from_utf8(string)?;
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
    format!("{}{}", date, random)
}
