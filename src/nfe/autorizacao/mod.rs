mod dest;
mod det;
mod det_process;
mod emit;
mod ide;
mod inf_adic;
pub mod pag;
mod total;
mod transp;

use crate::nfe;
use anyhow::{Error, Result};
use dest::dest_process;
use det::det_process;
use emit::{EmitProcess, EnderEmitProcess};
use ide::*;
use inf_adic::inf_adic_process;
use nfe::common::cert::Cert;
use nfe::common::chave_acesso::ChaveAcesso;
use nfe::common::cleaner;
use nfe::common::cleaner::Strings;
use nfe::common::dates::get_current_date_time;
use nfe::common::validation::is_xml_valid;
use nfe::common::ws::nfe_autorizacao;
use nfe::connection::WebService;
use nfe::types::autorizacao4::*;
use nfe::types::chave_acesso_props::ChaveAcessoProps;
use pag::pag_process;
use regex::Regex;
use serde_xml_rs::to_string;
use std::fs::File;
use std::io::Write;
use total::total_process;
use transp::transp_process;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Response {
    pub protocolo: TagInfProt,
    pub xml: String,
}

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

pub async fn emit(nfe: NFe) -> Result<Response, Error> {
    let codigo_numerico = ChaveAcesso::gerar_codigo_numerico(nfe.ide.c_nf);

    // atribua a doc a condicao que atribui o valor de nfe.emit.cnpj se some ou nfe.emit.cpf se some
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
        tp_amb: nfe.ide.tp_amb.clone(),
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
            mod_: nfe.ide.mod_.clone(),
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
            tp_amb: nfe.ide.tp_amb.clone(),
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
    let dest_string = match nfe.dest {
        Some(ref dest) => {
            let dest = dest_process(dest.clone())?;
            let dest_string = to_string(&dest)?;
            corrigir_tags_dest(dest_string)
        }
        None => {
            // Se não houver destinatário, retornar uma string vazia
            String::new()
        }
    };

    let dets = det_process(nfe.det, nfe.ide.mod_, nfe.ide.tp_amb)?;
    let mut det_string = String::new();
    for (i, det) in dets.iter().enumerate() {
        let prod = to_string(&det.prod).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do DetProcess: {:?}", e);
            return String::new();
        });
        let imposto = to_string(&det.imposto).unwrap_or_else(|e| {
            println!("Erro ao gerar o XML do DetProcess: {:?}", e);
            return String::new();
        });
        let inf_ad_prod = &det.inf_ad_prod;
        let inf_ad_prod = if let Some(inf_ad_prod) = inf_ad_prod {
            format!("<infAdProd>{}</infAdProd>", inf_ad_prod)
        } else {
            "".to_string()
        };
        det_string.push_str(&format!(
            r#"<det nItem="{}">{}{}{}</det>"#,
            i + 1,
            prod,
            imposto,
            inf_ad_prod
        ));
    }

    let total = total_process(nfe.total)?;
    let transp = transp_process(nfe.transp)?;
    let pag = pag_process(nfe.pag)?;
    let inf_adic = inf_adic_process(nfe.inf_adic)?;

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

    let xml = Strings::clear_xml_string(&xml);

    // generate digest value OK ---------------------------------------------
    let digest_value = crate::nfe::common::cert::DigestValue::sha1(&xml)?;

    // generate x509 certificate clean begin and end OK ----------------------
    let x509_cert =
        crate::nfe::common::cert::RawPubKey::get_from_file(&nfe.cert_path, &nfe.cert_pass).await?;

    // adicionar os campos de assinatura ------------------------------------
    //<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">
    let mut signed_info = "".to_string()
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
    signed_info = cleaner::Strings::clear_xml_string(&signed_info);

    // generate signature base64 -------------------------------------------
    let signature_base64 =
        crate::nfe::common::cert::Sign::xml_string(&signed_info, &nfe.cert_path, &nfe.cert_pass)
            .await?;

    // adicionar a assinatura ao XML ---------------------------------------
    let signature_nodes =
        signed_info + "<SignatureValue>" + &signature_base64 + "</SignatureValue>";

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

    // QRcode ------------------------------------------------------
    let mut qrcode = String::new();
    if nfe.ide.mod_ == 65 {
        // Exemplo de montagem do QR Code NFC-e (ajuste os campos conforme sua lógica)
        let mut url_base = String::new();
        if nfe.ide.tp_amb == 2 {
            url_base = "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx".to_string();
        } else if nfe.ide.tp_amb == 1 {
            url_base = "https://www.nfce.fazenda.sp.gov.br/qrcode".to_string();
        }

        let chave = &chave_acesso;
        let versao_qr = "2";
        let ambiente = nfe.ide.tp_amb.to_string();

        let id_csc = match nfe.id_csc {
            Some(id) => id,
            None => {
                return Err(Error::msg("ID do CSC não foi informado."));
            }
        };
        let csc = match nfe.csc {
            Some(c) => c,
            None => {
                return Err(Error::msg("CSC não foi informado."));
            }
        };
        let c_hash = qrcode_hash(&chave_acesso, &versao_qr, &ambiente, &id_csc, &csc)?;
        let url_qrcode = format!("{url_base}?p={chave}|{versao_qr}|{ambiente}|{id_csc}|{c_hash}");

        let consulta_homologacao = "https://www.homologacao.nfce.fazenda.sp.gov.br/consulta";
        let consulta_producao = "https://www.nfce.fazenda.sp.gov.br/consulta";
        let url_consulta = if nfe.ide.tp_amb == 2 {
            format!("{consulta_homologacao}")
        } else {
            format!("{consulta_producao}")
        };
        qrcode = format!(
            r#"<infNFeSupl>
                <qrCode><![CDATA[{url_qrcode}]]></qrCode>
                <urlChave>{url_consulta}</urlChave>
            </infNFeSupl>"#
        );
        // limpar qrcode
        qrcode = cleaner::Strings::clear_xml_string(&qrcode);
    }

    // add on top of xml ----------------------------------------------------
    let xml = "<NFe xmlns=\"http://www.portalfiscal.inf.br/nfe\">".to_string()
        + &xml
        + &qrcode
        + &signed_xml
        + "</NFe>";

    // salvar o XML assinado no arquivo nfe_request.xml ---------------------
    let mut file = File::create("./nfe_request.xml")
        .expect("Não foi possível criar o arquivo nfe_request.xml");
    file.write_all(&xml.as_bytes())
        .expect("Não foi possível escrever o arquivo nfe_request.xml");

    // validação do xml ----------------------------------------------------
    let signed_xml = match is_xml_valid(&xml, "./dfe/shema/PL_009p_NT2024_003_v1.03/nfe_v4.00.xsd")
    {
        Ok(xml) => xml,
        Err(e) => {
            //println!("Erro de validação XML: {}", e);
            return Err(Error::msg(format!("{}", e.to_string())));
        }
    };

    // envelope -------------------------------------------------------------
    // TODO: Identificador de controle do Lote de envio do Evento.
    // Número sequencial autoincremental único para identificação
    // do Lote. A responsabilidade de gerar e controlar é exclusiva
    // do autor do evento. O Web Service não faz qualquer uso
    // deste identificador u<1-15>
    let id_lote = 100;
    let lote_ini = format!(
        r#"<enviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><idLote>{}</idLote><indSinc>1</indSinc>"#,
        id_lote
    );
    let lote_fim = "</enviNFe>";

    let xml = format!(
        r#"<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4">{}{}{}</nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        lote_ini, &xml, lote_fim
    );

    // Selecionar o url do webservice -----------------------
    let url = nfe_autorizacao(nfe.ide.tp_amb, "SP", nfe.ide.mod_, false)?;

    let cert = Cert::from_pfx(&nfe.cert_path, &nfe.cert_pass)?;
    let client = WebService::client(cert.identity)?;

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
            // TODO: BUG Corrigir para retornar erro ao tentar salvar o arquivo que foi bem sucedido
            let mut file =
                File::create("./nfe_request.xml").expect("Não foi possível criar o arquivo");
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
    let re = Regex::new(r#"<protNFe versao="4.00">(.*?)</protNFe>"#)?;
    let prot_nfe = re.captures(&response).unwrap().get(0).unwrap().as_str();

    let tag_inf_prot: TagInfProt = serde_xml_rs::from_str(&prot_nfe)?;

    Ok(Response {
        protocolo: tag_inf_prot,
        xml: signed_xml,
    })
}
// TODO - ??? GAMBIARRA ??? Usar o quick_xml para gerar o XML.
// O serde_xml_rs para deserializar a resposta é bom
// mas para gerar é melhor o quick_xml
fn corrigir_tags_dest(string: String) -> String {
    // remover de dest string <CPF><CPF> e substituir por <CPF>
    let re = Regex::new(r"<CPF><CPF>").unwrap();
    let dest_string = re.replace_all(&string, "<CPF>").to_string();

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

    dest_string
}

fn qrcode_hash(
    chave_acesso: &str,
    versao_qr: &str,
    ambiente: &str,
    id_csc: &str,
    csc: &str,
) -> Result<String, Error> {
    use sha1::{Digest, Sha1};

    // Passo 1: Concatenar parâmetros separados por "|"
    let dados = format!("{chave_acesso}|{versao_qr}|{ambiente}|{id_csc}");

    // Passo 2: Adicionar o CSC ao final da string
    let dados_csc = format!("{dados}{csc}");

    // Passo 3: Aplicar SHA-1 e converter para hexadecimal
    let mut hasher = Sha1::new();
    hasher.update(dados_csc.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = format!("{:x}", hash);

    Ok(hash_hex)
}
