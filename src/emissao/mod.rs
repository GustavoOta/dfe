mod det;
mod det_process;
mod emit;
mod flag;
mod ide;
mod inf_adic;
pub mod pag;
mod total;
mod transp;

use crate::error::{DfeError, Result};
use crate::interno::cert::{Cert, DigestValue, RawPubKey, Sign};
use crate::interno::chave_acesso::ChaveAcesso;
use crate::interno::chave_acesso_props::ChaveAcessoProps;
use crate::interno::cleaner;
use crate::interno::cleaner::Strings;
use crate::interno::connection::WebService;
use crate::interno::dates::get_current_date_time;
use crate::interno::dest_xml::DestTAG;
use crate::interno::validation::is_xml_valid;
use crate::interno::ws::nfe_autorizacao;
use crate::tipos::{Dest, Det, Emit, Ide, InfAdic, Pag, Total, Transp};
use det::det_process;
use emit::{EmitProcess, EnderEmitProcess};
use flag::FlagAutorizacao;
use flag::FlagAutorizacaoEnum;
use ide::*;
use inf_adic::inf_adic_process;
use pag::pag_process;
use quick_xml::se::to_string;
use regex::Regex;
use rust_decimal::Decimal;
use std::fs::File;
use std::io::Write;
use total::total_process;
use transp::transp_process;

// Struct interna de montagem — não exposta como API pública
struct NFeInterno {
    pub cert_path: String,
    pub cert_pass: String,
    pub id_csc: Option<String>,
    pub csc: Option<String>,
    pub ide: Ide,
    pub emit: Emit,
    pub dest: Option<Dest>,
    pub det: Vec<Det>,
    pub total: Total,
    pub transp: Transp,
    pub pag: Pag,
    pub inf_adic: Option<InfAdic>,
    pub active_ibs_cbs: Option<String>,
    pub desconto_rateio: Option<Decimal>,
    pub referencias: Vec<String>,
}

/// Resposta da emissão de NF-e ou NFC-e retornada por [`NFeBuilder::emitir`].
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Response {
    /// Protocolo de autorização da SEFAZ.
    pub protocolo: TagInfProt,
    /// XML `nfeProc` autorizado — deve ser persistido em disco.
    pub xml: String,
}

/// Dados do protocolo de autorização (`<infProt>`).
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InfProt {
    /// Ambiente: `1` = Produção · `2` = Homologação.
    #[serde(rename = "tpAmb")]   pub tp_amb: i32,
    /// Versão do aplicativo da SEFAZ.
    #[serde(rename = "verAplic")] pub ver_aplic: String,
    /// Chave de acesso da NF-e (44 dígitos).
    #[serde(rename = "chNFe")]   pub ch_nfe: String,
    /// Data e hora do recebimento pela SEFAZ (ISO 8601).
    #[serde(rename = "dhRecbto")] pub dh_recbto: String,
    /// Número do protocolo de autorização. Presente somente quando `c_stat == 100`.
    #[serde(rename = "nProt", skip_serializing_if = "Option::is_none")] pub n_prot: Option<String>,
    /// Digest SHA-1 do XML assinado (base64).
    #[serde(rename = "digVal", skip_serializing_if = "Option::is_none")] pub dig_val: Option<String>,
    /// Código de status da SEFAZ. `100` = autorizado.
    #[serde(rename = "cStat")]   pub c_stat: i32,
    /// Descrição do status retornado pela SEFAZ.
    #[serde(rename = "xMotivo")] pub x_motivo: String,
}

/// Wrapper XML em torno de [`InfProt`] (`<protNFe><infProt>…`).
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TagInfProt {
    /// Dados do protocolo.
    #[serde(rename = "infProt")]
    pub inf_prot: InfProt,
}

// XML assinado e validado + metadados necessários para o envio SEFAZ
struct SignedNfe {
    nfe_xml: String,
    validated_xml: String,
    cert_path: String,
    cert_pass: String,
    ide_mod: u32,
    ide_tp_amb: u8,
}

// Constrói e assina o XML da NF-e sem enviar à SEFAZ
async fn build_signed_xml(nfe: NFeInterno) -> Result<SignedNfe> {
    let cert_path = nfe.cert_path.clone();
    let cert_pass = nfe.cert_pass.clone();
    let ide_mod = nfe.ide.mod_;
    let ide_tp_amb = nfe.ide.tp_amb;
    let id_csc = nfe.id_csc.clone();
    let csc = nfe.csc.clone();
    let inf_adic = nfe.inf_adic.clone();
    let referencias = nfe.referencias.clone();

    let codigo_numerico = ChaveAcesso::gerar_codigo_numerico(nfe.ide.c_nf.clone());
    let doc = match (nfe.emit.cnpj.as_ref(), nfe.emit.cpf.as_ref()) {
        (Some(cnpj), _) => cnpj.clone(),
        (None, Some(cpf)) => cpf.clone(),
        (None, None) => String::new(),
    };

    let ch_acc = ChaveAcesso::gerar_chave_acesso(ChaveAcessoProps {
        uf: nfe.ide.c_uf,
        doc,
        modelo: nfe.ide.mod_,
        serie: nfe.ide.serie,
        numero: nfe.ide.n_nf,
        tp_emis: nfe.ide.tp_emis,
        codigo_numerico: codigo_numerico.clone(),
    });
    let chave_acesso = ch_acc.chave;
    let dv = ch_acc.dv;

    let dh_emi = nfe.ide.dh_emi.clone().unwrap_or_else(get_current_date_time);
    let dh_sai_ent = nfe.ide.dh_sai_ent.clone().unwrap_or_else(get_current_date_time);

    let mut ide_process = IdeProcess {
        c_uf: nfe.ide.c_uf,
        c_nf: Some(codigo_numerico.clone()),
        nat_op: nfe.ide.nat_op.clone(),
        ind_pag: nfe.ide.ind_pag,
        mod_: nfe.ide.mod_.clone(),
        serie: nfe.ide.serie,
        n_nf: nfe.ide.n_nf,
        dh_emi: Some(dh_emi.clone()),
        dh_sai_ent: Some(dh_sai_ent),
        tp_nf: nfe.ide.tp_nf,
        id_dest: nfe.ide.id_dest,
        c_mun_fg: nfe.ide.c_mun_fg.clone(),
        tp_imp: nfe.ide.tp_imp,
        tp_emis: nfe.ide.tp_emis,
        c_dv: Some(dv),
        tp_amb: nfe.ide.tp_amb,
        fin_nfe: nfe.ide.fin_nfe,
        ind_final: nfe.ide.ind_final,
        ind_pres: nfe.ide.ind_pres,
        proc_emi: nfe.ide.proc_emi,
        ver_proc: nfe.ide.ver_proc.clone(),
    };
    if nfe.ide.mod_ == 65 {
        ide_process.dh_sai_ent = None;
        ide_process.c_nf = Some(codigo_numerico);
        ide_process.dh_emi = Some(dh_emi);
    }

    // Serializa <ide> e injeta tags <NFref> antes de </ide> quando houver referências
    let nf_refs_xml: String = referencias
        .iter()
        .map(|ch| format!("<NFref><refNFe>{}</refNFe></NFref>", ch))
        .collect();
    let ide_xml = {
        let base = to_string(&ide_process).unwrap_or_default();
        if nf_refs_xml.is_empty() {
            base
        } else {
            base.replace("</ide>", &format!("{}</ide>", nf_refs_xml))
        }
    };

    let emit_process = EmitProcess {
        cnpj: nfe.emit.cnpj.clone(),
        cpf: nfe.emit.cpf.clone(),
        x_nome: nfe.emit.x_nome.clone(),
        x_fant: nfe.emit.x_fant.clone(),
        ender_emit: EnderEmitProcess {
            x_lgr: nfe.emit.x_lgr.clone(),
            nro: nfe.emit.nro.clone(),
            x_bairro: nfe.emit.x_bairro.clone(),
            c_mun: nfe.emit.c_mun.clone(),
            x_mun: nfe.emit.x_mun.clone(),
            uf: nfe.emit.uf.clone(),
            cep: nfe.emit.cep.clone(),
            c_pais: nfe.emit.c_pais,
            x_pais: nfe.emit.x_pais.clone(),
        },
        ie: nfe.emit.ie.clone(),
        crt: nfe.emit.crt,
    };

    let dest_string = DestTAG::build(&nfe.dest, &nfe.ide)?;

    let dets = det_process(
        nfe.det.clone(), nfe.ide.mod_, nfe.ide.tp_amb,
        nfe.desconto_rateio.clone(), nfe.active_ibs_cbs.clone(),
    )?;
    let dets_total = dets.clone();

    let mut det_string = String::new();
    for (i, det) in dets.iter().enumerate() {
        let prod    = to_string(&det.prod).unwrap_or_default();
        let imposto = det.imposto.to_xml();
        let inf_ad  = det.inf_ad_prod.as_ref().map(|v| format!("<infAdProd>{}</infAdProd>", v)).unwrap_or_default();
        det_string.push_str(&format!(r#"<det nItem="{}">{}{}{}</det>"#, i + 1, prod, imposto, inf_ad));
    }

    let total_process_result = total_process(nfe.total.clone(), dets_total, nfe.ide.tp_amb, nfe.active_ibs_cbs.clone())?;
    let v_nf: f64 = total_process_result.icms_tot.v_nf.parse().unwrap_or(0.0);
    let transp_process_result = transp_process(nfe.transp.clone())?;
    let inf_adic_process_result = inf_adic_process(inf_adic)?;

    // pag_process recebe NFeInterno por valor — chamado por último
    let pag_process_result = pag_process(nfe, v_nf)?;

    let xml = format!(
        "<infNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" Id=\"NFe{}\" versao=\"4.00\">{}{}{}{}{}{}{}{}{}",
        chave_acesso,
        ide_xml,
        to_string(&emit_process).unwrap_or_default(),
        dest_string, det_string,
        to_string(&total_process_result).unwrap_or_default(),
        to_string(&transp_process_result).unwrap_or_default(),
        to_string(&pag_process_result).unwrap_or_default(),
        to_string(&inf_adic_process_result).unwrap_or_default(),
        "</infNFe>"
    );

    let xml = Strings::clear_xml_string(&xml);
    let digest_value = DigestValue::sha1(&xml)?;
    let x509_cert = RawPubKey::get_from_file(&cert_path, &cert_pass).await?;

    let mut signed_info = String::new()
        + "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">"
        + "<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>"
        + "<SignatureMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\"></SignatureMethod>"
        + "<Reference URI=\"#NFe" + &chave_acesso + "\">"
        + "<Transforms>"
        + "<Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>"
        + "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform>"
        + "</Transforms>"
        + "<DigestMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#sha1\"></DigestMethod>"
        + "<DigestValue>" + &digest_value + "</DigestValue>"
        + "</Reference></SignedInfo>";
    signed_info = cleaner::Strings::clear_xml_string(&signed_info);

    let signature_base64 = Sign::xml_string(&signed_info, &cert_path, &cert_pass).await?;
    let signature_nodes = signed_info + "<SignatureValue>" + &signature_base64 + "</SignatureValue>";
    let signature_xml = "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">".to_string()
        + &signature_nodes
        + "<KeyInfo><X509Data><X509Certificate>" + &x509_cert
        + "</X509Certificate></X509Data></KeyInfo></Signature>";

    let mut qrcode = String::new();
    if ide_mod == 65 {
        let url_base = if ide_tp_amb == 2 {
            "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx"
        } else {
            "https://www.nfce.fazenda.sp.gov.br/qrcode"
        };
        let versao_qr = "2";
        let ambiente = ide_tp_amb.to_string();
        let id_csc = id_csc.ok_or_else(|| DfeError::Validacao("ID do CSC não foi informado.".to_string()))?;
        let csc = csc.ok_or_else(|| DfeError::Validacao("CSC não foi informado.".to_string()))?;
        let c_hash = qrcode_hash(&chave_acesso, versao_qr, &ambiente, &id_csc, &csc)?;
        let url_consulta = if ide_tp_amb == 2 {
            "https://www.homologacao.nfce.fazenda.sp.gov.br/consulta"
        } else {
            "https://www.nfce.fazenda.sp.gov.br/consulta"
        };
        qrcode = cleaner::Strings::clear_xml_string(&format!(
            r#"<infNFeSupl><qrCode><![CDATA[{url_base}?p={chave_acesso}|{versao_qr}|{ambiente}|{id_csc}|{c_hash}]]></qrCode><urlChave>{url_consulta}</urlChave></infNFeSupl>"#
        ));
    }

    let nfe_xml = "<NFe xmlns=\"http://www.portalfiscal.inf.br/nfe\">".to_string()
        + &xml + &qrcode + &signature_xml + "</NFe>";

    let mut f = File::create("./nfe_request.xml").expect("nfe_request.xml");
    f.write_all(nfe_xml.as_bytes()).expect("write");

    let validated_xml = match is_xml_valid(&nfe_xml) {
        Ok(x) => x,
        Err(e) => return Err(DfeError::Validacao(format!("is_xml_valid: [{}]", e))),
    };

    Ok(SignedNfe { nfe_xml, validated_xml, cert_path, cert_pass, ide_mod, ide_tp_amb })
}

async fn emit_nfe(nfe: NFeInterno) -> Result<Response> {
    let flag = FlagAutorizacao::start().await.map_err(DfeError::Validacao)?;
    match flag {
        FlagAutorizacaoEnum::Ready => {}
        _ => return Err(DfeError::Validacao(format!(
            "Flag de autorização inválida para emissão: [{:?}].", flag
        ))),
    }

    let signed = build_signed_xml(nfe).await?;

    let id_lote = 100;
    let xml_envelope = format!(
        r#"<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4"><enviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><idLote>{}</idLote><indSinc>1</indSinc>{}</enviNFe></nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        id_lote, &signed.nfe_xml
    );

    let url = nfe_autorizacao(signed.ide_tp_amb, "SP", signed.ide_mod, false)?;
    let cert = Cert::from_pfx(&signed.cert_path, &signed.cert_pass)?;
    let client = WebService::client(cert.identity)?;

    let xml_with_declaration = if xml_envelope.starts_with("<?xml") {
        xml_envelope.clone()
    } else {
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml_envelope)
    };

    let mut f = File::create("./nfe_request_envelope.xml").expect("envelope");
    f.write_all(xml_with_declaration.as_bytes()).expect("write");
    f.sync_all().expect("sync");

    let response = client
        .post(url)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .header("Content-Length", xml_with_declaration.len().to_string())
        .body(xml_with_declaration)
        .send()
        .await?;

    if response.status().is_success() {
        let result = xml_result(&response.text().await?, signed.validated_xml)?;
        if result.protocolo.inf_prot.c_stat != 100 {
            return Ok(result);
        }
        let protocolo = format!(
            r#"</NFe><protNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><infProt><tpAmb>{}</tpAmb><verAplic>{}</verAplic><chNFe>{}</chNFe><dhRecbto>{}</dhRecbto><nProt>{}</nProt><digVal>{}</digVal><cStat>{}</cStat><xMotivo>{}</xMotivo></infProt></protNFe></nfeProc>"#,
            result.protocolo.inf_prot.tp_amb, result.protocolo.inf_prot.ver_aplic,
            result.protocolo.inf_prot.ch_nfe, result.protocolo.inf_prot.dh_recbto,
            result.protocolo.inf_prot.n_prot.clone().unwrap_or_default(),
            result.protocolo.inf_prot.dig_val.clone().unwrap_or_default(),
            result.protocolo.inf_prot.c_stat, result.protocolo.inf_prot.x_motivo
        );
        let nfe_proc_xml = r#"<?xml version="1.0" encoding="UTF-8"?><nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#.to_string()
            + &result.xml.replace("</NFe>", &protocolo);
        let mut f = File::create("./nfe_response.xml").expect("response.xml");
        f.write_all(nfe_proc_xml.as_bytes()).expect("write");
        Ok(Response { protocolo: result.protocolo, xml: nfe_proc_xml.replace("\\", "") })
    } else {
        let status = response.status();
        let body = response.text().await?;
        Err(DfeError::Webservice(format!("Erro na Requisição: {:?} -> Body: {:?}", status, body)))
    }
}

fn xml_result(response: &str, validated_xml: String) -> Result<Response> {
    let re = Regex::new(r#"<protNFe versao="4.00">(.*?)</protNFe>"#)
        .map_err(|e| DfeError::Xml(format!("Erro regex: {}", e)))?;
    if let Some(prot_nfe) = re.captures(response).and_then(|c| c.get(0)).map(|m| m.as_str()) {
        let tag_inf_prot: TagInfProt = quick_xml::de::from_str(prot_nfe)
            .map_err(|e| DfeError::Xml(format!("Erro desserializar protocolo: {} — {}", e, prot_nfe)))?;
        return Ok(Response { protocolo: tag_inf_prot, xml: validated_xml });
    }

    // Sem <protNFe>: o SEFAZ rejeitou o LOTE inteiro (ex.: cStat 225 — "Falha no
    // Schema XML do lote de NFe"). Nesse caso o cStat/xMotivo vêm no nível do
    // retEnviNFe, sem protocolo por nota. Monta um protocolo sintético com o
    // status do lote (c_stat != 100) para que a rejeição siga o mesmo caminho de
    // uma rejeição por nota, em vez de mascarar o motivo real como "protNFe not found".
    if let Some(c_stat) = extract_xml_tag(response, "cStat").and_then(|s| s.parse::<i32>().ok()) {
        let inf_prot = InfProt {
            tp_amb: extract_xml_tag(response, "tpAmb").and_then(|s| s.parse().ok()).unwrap_or(0),
            ver_aplic: extract_xml_tag(response, "verAplic").unwrap_or_default(),
            ch_nfe: extract_xml_tag(response, "chNFe").unwrap_or_default(),
            dh_recbto: extract_xml_tag(response, "dhRecbto").unwrap_or_default(),
            n_prot: None,
            dig_val: None,
            c_stat,
            x_motivo: extract_xml_tag(response, "xMotivo").unwrap_or_default(),
        };
        return Ok(Response { protocolo: TagInfProt { inf_prot }, xml: validated_xml });
    }

    // Nem protocolo de nota, nem status de lote — resposta realmente inesperada.
    Err(DfeError::Xml(format!("protNFe not found: {}", response)))
}

/// Extrai o conteúdo da primeira ocorrência de `<tag>…</tag>` no XML de resposta.
/// Usado para ler o status no nível do lote (`retEnviNFe`) quando não há `<protNFe>`.
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let re = Regex::new(&format!(r"(?s)<{0}[^>]*>(.*?)</{0}>", regex::escape(tag))).ok()?;
    re.captures(xml)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_result_autorizacao_extrai_protocolo() {
        let resp = r#"<retEnviNFe versao="4.00"><protNFe versao="4.00"><infProt><tpAmb>2</tpAmb><verAplic>SP_NFCE</verAplic><chNFe>3500000000000000000065010000000011234567890</chNFe><dhRecbto>2026-06-22T11:09:10-03:00</dhRecbto><nProt>135260000000001</nProt><digVal>abc=</digVal><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe></retEnviNFe>"#;
        let r = xml_result(resp, "<xml/>".into()).expect("deve extrair protocolo");
        assert_eq!(r.protocolo.inf_prot.c_stat, 100);
        assert_eq!(r.protocolo.inf_prot.n_prot.as_deref(), Some("135260000000001"));
    }

    #[test]
    fn xml_result_rejeicao_de_lote_vira_protocolo_sintetico() {
        // retEnviNFe sem <protNFe>: o lote inteiro falhou no schema (cStat 225).
        // Antes, isso virava o erro opaco "protNFe not found"; agora deve expor
        // cStat/xMotivo do nível do lote para o consumidor.
        let resp = r#"<retEnviNFe versao="4.00"><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL_009_V400</verAplic><cStat>225</cStat><xMotivo>Rejeição: Falha no Schema XML do lote de NFe</xMotivo><cUF>35</cUF><dhRecbto>2026-06-22T11:09:10-03:00</dhRecbto></retEnviNFe>"#;
        let r = xml_result(resp, "<xml/>".into()).expect("rejeição de lote deve virar Ok com protocolo sintético");
        assert_eq!(r.protocolo.inf_prot.c_stat, 225);
        assert_eq!(r.protocolo.inf_prot.ver_aplic, "SP_NFCE_PL_009_V400");
        assert_eq!(r.protocolo.inf_prot.x_motivo, "Rejeição: Falha no Schema XML do lote de NFe");
        assert!(r.protocolo.inf_prot.n_prot.is_none());
    }

    #[test]
    fn xml_result_sem_status_retorna_erro() {
        let resp = "<html><body>502 Bad Gateway</body></html>";
        let err = xml_result(resp, "<xml/>".into()).unwrap_err();
        assert!(matches!(err, DfeError::Xml(msg) if msg.contains("protNFe not found")));
    }
}

fn qrcode_hash(chave_acesso: &str, versao_qr: &str, ambiente: &str, id_csc: &str, csc: &str) -> Result<String> {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(format!("{chave_acesso}|{versao_qr}|{ambiente}|{id_csc}{csc}").as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
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
    referencias: Vec<String>,
}

impl NFeBuilder {
    /// Cria um builder vazio. Chame os métodos de configuração antes de [`emitir`](Self::emitir).
    pub fn new() -> Self {
        Self {
            cert_path: None, cert_pass: None, ide: None, emitente: None,
            destinatario: None, itens: Vec::new(), total: None, transporte: None,
            pagamento: None, informacoes_adicionais: None, id_csc: None, csc: None,
            active_ibs_cbs: None, desconto_rateio: None, referencias: Vec::new(),
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

        if self.itens.is_empty() {
            return Err(DfeError::Validacao("pelo menos um item (det) deve ser informado".to_string()));
        }

        let signed = build_signed_xml(NFeInterno {
            cert_path, cert_pass, id_csc: self.id_csc, csc: self.csc,
            ide, emit: emitente, dest: self.destinatario,
            det: self.itens, total, transp: transporte, pag: pagamento,
            inf_adic: self.informacoes_adicionais,
            active_ibs_cbs: self.active_ibs_cbs,
            desconto_rateio: self.desconto_rateio,
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

        if self.itens.is_empty() {
            return Err(DfeError::Validacao("pelo menos um item (det) deve ser informado".to_string()));
        }

        emit_nfe(NFeInterno {
            cert_path, cert_pass, id_csc: self.id_csc, csc: self.csc,
            ide, emit: emitente, dest: self.destinatario,
            det: self.itens, total, transp: transporte, pag: pagamento,
            inf_adic: self.informacoes_adicionais,
            active_ibs_cbs: self.active_ibs_cbs,
            desconto_rateio: self.desconto_rateio,
            referencias: self.referencias,
        }).await
    }
}
