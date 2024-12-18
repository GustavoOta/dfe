mod dest;
mod det;
mod emit;
mod ide;
mod inf_adic;
mod pag;
mod total;
mod transp;

use crate::nfe;
use crate::nfe::common::cert::{self, Cert};
use crate::nfe::common::dates::get_current_date_time;
use crate::nfe::common::ws::nfe_autorizacao;
use crate::nfe::connection::WebService;
use crate::nfe::types::autorizacao4::*;
use crate::nfe::types::chave_acesso_props::ChaveAcessoProps;
use anyhow::{Error, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use dest::dest_process;
use det::det_process;
use emit::{EmitProcess, EnderEmitProcess};
use ide::*;
use inf_adic::inf_adic_process;
use nfe::common::chave_acesso::ChaveAcesso;
use openssl::pkcs12::Pkcs12;
use openssl::sign::Signer;
use pag::pag_process;
use regex::Regex;
use serde_xml_rs::to_string;
//use sha1::digest;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use total::total_process;
use transp::transp_process;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InfProt {
    #[serde(rename = "tpAmb")]
    pub tp_amb: i32,
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    #[serde(rename = "chNFe")]
    pub ch_nfe: String,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: String,
    #[serde(rename = "nProt", skip_serializing_if = "Option::is_none")]
    pub n_prot: Option<String>,
    #[serde(rename = "digVal", skip_serializing_if = "Option::is_none")]
    pub dig_val: Option<String>,
    #[serde(rename = "cStat")]
    pub c_stat: i32,
    #[serde(rename = "xMotivo")]
    pub x_motivo: String,
}
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TagInfProt {
    #[serde(rename = "infProt")]
    pub inf_prot: InfProt,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Response {
    pub protocolo: TagInfProt,
    pub xml: String,
}

pub async fn emit(nfe: NFe) -> Result<Response, Error> {
    let codigo_numerico = if let Some(codigo_numerico) = nfe.ide.c_nf {
        codigo_numerico
    } else {
        ChaveAcesso::gerar_codigo_numerico()
    };

    // atribua a doc a condicao que atribui o valor de nfe.emit.cnpj se some ou nfe.emit.cpf se some
    let doc = if let Some(cnpj) = nfe.emit.cnpj.clone() {
        cnpj
    } else if let Some(cpf) = nfe.emit.cpf.clone() {
        cpf
    } else {
        return Err(Error::msg("CNPJ ou CPF do emitente não informado"));
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

    let dh_emi = if let Some(dh_emi) = nfe.ide.dh_emi {
        dh_emi
    } else {
        get_current_date_time()
    };

    let dh_sai_ent = if let Some(dh_sai_ent) = nfe.ide.dh_sai_ent {
        dh_sai_ent
    } else {
        get_current_date_time()
    };

    // Gerar o XML da NF-e
    let mut ide = IdeProcess {
        c_uf: nfe.ide.c_uf,
        c_nf: Some(codigo_numerico.clone()),
        nat_op: nfe.ide.nat_op.clone(),
        ind_pag: nfe.ide.ind_pag,
        mod_: nfe.ide.mod_,
        serie: nfe.ide.serie,
        n_nf: nfe.ide.n_nf,
        dh_emi: Some(dh_emi.clone()),
        dh_sai_ent: Some(dh_sai_ent),
        tp_nf: nfe.ide.tp_nf,
        id_dest: nfe.ide.id_dest,
        c_mun_fg: nfe.ide.c_mun_fg,
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
        ide = IdeProcess {
            c_uf: nfe.ide.c_uf,
            c_nf: Some(codigo_numerico),
            nat_op: nfe.ide.nat_op,
            ind_pag: nfe.ide.ind_pag,
            mod_: nfe.ide.mod_,
            serie: nfe.ide.serie,
            n_nf: nfe.ide.n_nf,
            dh_emi: Some(dh_emi),
            dh_sai_ent: None,
            tp_nf: nfe.ide.tp_nf,
            id_dest: nfe.ide.id_dest,
            c_mun_fg: nfe.ide.c_mun_fg,
            tp_imp: nfe.ide.tp_imp,
            tp_emis: nfe.ide.tp_emis,
            c_dv: Some(dv),
            tp_amb: nfe.ide.tp_amb,
            fin_nfe: nfe.ide.fin_nfe,
            ind_final: nfe.ide.ind_final,
            ind_pres: nfe.ide.ind_pres,
            proc_emi: nfe.ide.proc_emi,
            ver_proc: nfe.ide.ver_proc,
        };
    }

    let emit = EmitProcess {
        cnpj: nfe.emit.cnpj.clone(),
        cpf: nfe.emit.cpf.clone(),
        x_nome: nfe.emit.x_nome,
        x_fant: nfe.emit.x_fant,
        ender_emit: EnderEmitProcess {
            x_lgr: nfe.emit.x_lgr,
            nro: nfe.emit.nro,
            x_bairro: nfe.emit.x_bairro,
            c_mun: nfe.emit.c_mun,
            x_mun: nfe.emit.x_mun,
            uf: nfe.emit.uf,
            cep: nfe.emit.cep,
            c_pais: nfe.emit.c_pais.unwrap_or(0),
            x_pais: nfe.emit.x_pais.unwrap_or("".to_string()),
        },
        ie: nfe.emit.ie,
        crt: nfe.emit.crt,
    };

    let dest = dest_process(nfe.dest)?;

    let dets = det_process(nfe.det)?;
    let mut det_string = String::new();
    for (i, det) in dets.iter().enumerate() {
        let prod = to_string(&det.prod).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do Detalhe: {:?}", e);
            return String::new();
        });
        let imposto = to_string(&det.imposto).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do Detalhe: {:?}", e);
            return String::new();
        });
        det_string.push_str(&format!(
            r#"<det nItem="{}">{}{}</det>"#,
            i + 1,
            prod,
            imposto
        ));
    }

    let total = total_process(nfe.total)?;

    let transp = transp_process(nfe.transp)?;

    let pag = pag_process(nfe.pag)?;

    let inf_adic = inf_adic_process(nfe.inf_adic)?;

    let dest_string = to_string(&dest);
    let dest_string = match dest_string {
        Ok(dest_string) => dest_string,
        Err(e) => {
            println!("Erro ao gerar o XML do Destinatário: {:?}", e);
            return Err(Error::msg("Erro ao gerar o XML do Destinatário"));
        }
    };

    // remover de dest string <CPF><CPF> e substituir por <CPF>
    let re = Regex::new(r"<CPF><CPF>").unwrap();
    let dest_string = re.replace_all(&dest_string, "<CPF>").to_string();

    // remover de dest string <CNPJ><CNPJ> e substituir por <CNPJ>
    let re = Regex::new(r"<CNPJ><CNPJ>").unwrap();
    let dest_string = re.replace_all(&dest_string, "<CNPJ>").to_string();

    // remover de dest string <idEstrangeiro><idEstrangeiro> e substituir por <idEstrangeiro>
    let re = Regex::new(r"<idEstrangeiro><idEstrangeiro>").unwrap();
    let dest_string = re.replace_all(&dest_string, "<idEstrangeiro>").to_string();

    // remover de dest string </CPF></CPF> e substituir por ""
    let re = Regex::new(r"</enderDest></CPF>").unwrap();
    let dest_string = re.replace_all(&dest_string, "</enderDest>").to_string();

    // remover de dest string </CNPJ></CNPJ> e substituir por ""
    let re = Regex::new(r"</enderDest></CNPJ>").unwrap();
    let dest_string = re.replace_all(&dest_string, "</enderDest>").to_string();

    // remover de dest string </CNPJ></dest> e substituir por "</dest>"
    let re = Regex::new(r"</CNPJ></dest>").unwrap();
    let dest_string = re.replace_all(&dest_string, "</dest>").to_string();

    // remover de dest string </CPF></dest> e substituir por "</dest>"
    let re = Regex::new(r"</CPF></dest>").unwrap();
    let dest_string = re.replace_all(&dest_string, "</dest>").to_string();

    // remover de dest string </idEstrangeiro></idEstrangeiro> e substituir por ""
    let re = Regex::new(r"</enderDest></idEstrangeiro>").unwrap();
    let dest_string = re.replace_all(&dest_string, "</enderDest>").to_string();

    let xml = format!(
        "<infNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" Id=\"NFe{}\" versao=\"4.00\">{}{}{}{}{}{}{}{}{}",
        chave_acesso,
        to_string(&ide).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do IDE: {:?}", e);
            return String::new();
        }),
        to_string(&emit).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do Emitente: {:?}", e);
            return String::new();
        }),
        dest_string,
        det_string,
        to_string(&total).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do Total: {:?}", e);
            return String::new();
        }),
        to_string(&transp).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do Transporte: {:?}", e);
            return String::new();
        }),
        to_string(&pag).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do Pagamento: {:?}", e);
            return String::new();
        }),
        to_string(&inf_adic).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML das Informações Adicionais: {:?}", e);
            return String::new();
        }),
        "</infNFe>"
    );

    use nfe::common::cleaner::Strings;
    let xml = Strings::clear_xml_string(&xml);

    // generate digest value OK ---------------------------------------------
    let digest_value = crate::nfe::common::cert::DigestValue::sha1(&xml);

    // generate x509 certificate clean begin and end OK ----------------------
    let x509_cert = cert::raw_pub_key(&nfe.cert_path, &nfe.cert_pass)
        .await
        .unwrap();

    // adicionar os campos de assinatura ------------------------------------
    //<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">
    let mut digest_nodes = "".to_string()
        + "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">"
        + "<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>"
        + "<SignatureMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\"></SignatureMethod>"
        + "<Reference URI=\"#NFe"
        + &chave_acesso
        + "\">"
        + "<Transforms>"
        + "<Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>"
        + "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform>"
        + "</Transforms>"
        + "<DigestMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#sha1\"></DigestMethod>"
        + "<DigestValue>"
        + &digest_value
        + "</DigestValue>"
        + "</Reference>"
        + "</SignedInfo>";

    // clean digest nodes use regex para remover \
    digest_nodes = digest_nodes.replace("\n", "");
    digest_nodes = digest_nodes.replace("\r", "");
    digest_nodes = digest_nodes.replace("\t", "");
    digest_nodes = digest_nodes.replace(" /", "/");
    digest_nodes = digest_nodes.replace("\\", "");
    // Remove espaços em branco entre '>' e '<'
    let re = Regex::new(r">\s+<").unwrap();
    digest_nodes = re.replace_all(&digest_nodes, "><").to_string();

    // generate signature base64 -------------------------------------------
    //openssl_sign($digest_value, $encryptedData, $this->resource, $algorithm)
    let signature_base64 = openssl_sign(&digest_nodes, &nfe.cert_path, &nfe.cert_pass)
        .await
        .unwrap();
    let signature_base64 = STANDARD.encode(&signature_base64);

    // adicionar a assinatura ao XML ---------------------------------------
    // att signature
    let signature_nodes =
        digest_nodes + "<SignatureValue>" + &signature_base64 + "</SignatureValue>";

    // add x509 certificate ------------------------------------------------
    let signed_xml = "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">".to_string()
        + &signature_nodes
        + "<KeyInfo>"
        + "<X509Data>"
        + "<X509Certificate>"
        + &x509_cert
        + "</X509Certificate>"
        + "</X509Data>"
        + "</KeyInfo>"
        + "</Signature>";

    // add on top of xml ----------------------------------------------------
    let xml = "<NFe xmlns=\"http://www.portalfiscal.inf.br/nfe\">".to_string()
        + &xml
        + &signed_xml
        + "</NFe>";

    // clean /n and /r white space /t ---------------------------------------
    let xml = xml.replace("\n", "");
    let xml = xml.replace("\r", "");
    let xml = xml.replace("\t", "");
    let xml = xml.replace(" /", "/");
    // Remove espaços em branco entre '>' e '<'
    let re = Regex::new(r">\s+<").unwrap();
    let xml = re.replace_all(&xml, "><").to_string();

    // remove <?xml version=\"1.0\" encoding=\"utf-8\"?> ---------------------
    let xml = xml.replace("<?xml version=\"1.0\" encoding=\"utf-8\"?>", "");

    // validação do xml ----------------------------------------------------
    use crate::nfe::common::validation::is_xml_valid;
    let is_valid = is_xml_valid(&xml, "./dfe/shema/PL_009p_NT2024_003_v1.02/nfe_v4.00.xsd");
    if is_valid.is_err() {
        // save xml_validation_error.xml
        let mut file =
            File::create("./xml_validation_error.xml").expect("Não foi possível criar o arquivo");
        file.write_all(xml.as_bytes())
            .expect("Não foi possível escrever o arquivo");

        return Err(Error::msg(format!(
            "Erro ao validar o XML: {:?} -> XML Gerado: {:?}",
            is_valid.err(),
            xml
        )));
    }

    let signed_xml = xml.clone();

    // envelope -------------------------------------------------------------
    let lote_ini = r#"<enviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><idLote>100</idLote><indSinc>1</indSinc>"#;
    let lote_fim = "</enviNFe>";

    let xml = format!(
        r#"<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4">{}{}{}</nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        lote_ini, &xml, lote_fim
    );

    // Enviar a NF-e para a SEFAZ de homologacao de SP -----------------------
    //let url = "https://homologacao.nfe.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx";
    //let url = nfe_autorizacao(&nfe.ide.tp_amb, &nfe.ide.uf, &nfe.ide.mod_);

    let url = nfe_autorizacao(nfe.ide.tp_amb, "SP", nfe.ide.mod_, false)?;

    let cert = Cert::from_pfx(&nfe.cert_path, &nfe.cert_pass)
        .expect("Não foi possível carregar o certificado");
    let client = WebService::client(cert.identity).unwrap();

    // Verificar a declaração de codificação no XML
    let xml_with_declaration = if xml.starts_with("<?xml") {
        xml.to_string()
    } else {
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml)
    };

    // Calcular o comprimento do corpo da requisição
    let content_length = xml_with_declaration.len();

    let response = client
        .post(url)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .header("Content-Length", content_length.to_string())
        .body(xml_with_declaration)
        .send()
        .await?;

    if response.status().is_success() {
        let result = xml_result(&response.text().await?, signed_xml)?;
        if result.protocolo.inf_prot.c_stat != 100 {
            return Ok(result);
        } else {
            // build protocolo into xml
            let protocolo = format!(
                r#"</NFe><protNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><infProt><tpAmb>{}</tpAmb><verAplic>{}</verAplic><chNFe>{}</chNFe><dhRecbto>{}</dhRecbto><nProt>{}</nProt><digVal>{}</digVal><cStat>{}</cStat><xMotivo>{}</xMotivo></infProt></protNFe></nfeProc>"#,
                result.protocolo.inf_prot.tp_amb,
                result.protocolo.inf_prot.ver_aplic,
                result.protocolo.inf_prot.ch_nfe,
                result.protocolo.inf_prot.dh_recbto,
                result
                    .protocolo
                    .inf_prot
                    .n_prot
                    .clone()
                    .unwrap_or("".to_string()),
                result
                    .protocolo
                    .inf_prot
                    .dig_val
                    .clone()
                    .unwrap_or("".to_string()),
                result.protocolo.inf_prot.c_stat,
                result.protocolo.inf_prot.x_motivo
            );

            // insert protocolo into xml string
            let xml = result.xml.replace("</NFe>", &protocolo);
            let xml = r#"<?xml version="1.0" encoding="UTF-8"?><nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">"#
                .to_string()
                + &xml;
            // save to root as nfe_request.xml
            let mut file = File::create("D:/Projetos/dfe/nfe_request.xml")
                .expect("Não foi possível criar o arquivo");
            file.write_all(xml.as_bytes())
                .expect("Não foi possível escrever o arquivo");

            // clear \ from <?xml version=\"1.0\" encoding=\"UTF-8\"?>
            let xml = xml.replace("\\", "");

            // update response xml
            let response = Response {
                protocolo: result.protocolo,
                xml,
            };
            Ok(response)
        }
    } else {
        return Err(Error::msg(format!(
            "Erro na Requisição: {:?} -> Body: {:?}",
            response.status(),
            response.text().await?
        )));
    }
}

fn xml_result(response: &str, signed_xml: String) -> Result<Response, Error> {
    // Extract the response body with the infProt tag and its content
    let re = Regex::new(r#"<protNFe versao="4.00">(.*?)</protNFe>"#)?;
    let prot_nfe = re.captures(&response).unwrap().get(0).unwrap().as_str();

    let tag_inf_prot: TagInfProt = serde_xml_rs::from_str(&prot_nfe)?;

    Ok(Response {
        protocolo: tag_inf_prot,
        xml: signed_xml,
    })
}

async fn openssl_sign(data: &str, pfx_path: &str, password: &str) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::new();
    let mut pfx = File::open(pfx_path).expect("Não foi possível abrir o arquivo do certificado");
    pfx.read_to_end(&mut buf)
        .expect("Não foi possível ler o arquivo do certificado");

    let pkcs12 = Pkcs12::from_der(&buf).expect("Não foi possível carregar o certificado");

    // assinar data
    let pkey = pkcs12
        .parse2(password)
        .expect("Não foi possível carregar o certificado");

    let pkey = pkey.pkey.expect("Chave privada não encontrada");

    let mut signer = Signer::new(openssl::hash::MessageDigest::sha1(), &pkey)
        .expect("Não foi possível criar o assinador");
    signer
        .update(data.as_bytes())
        .expect("Não foi possível assinar o XML");

    let signature = signer
        .sign_to_vec()
        .expect("Não foi possível assinar o XML");

    Ok(signature)
}
