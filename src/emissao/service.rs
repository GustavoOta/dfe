//! Orquestração da emissão: valida a flag, monta+assina (via [`super::xml`]), envia (via
//! [`crate::interno::transporte`]), parseia (via [`super::parser`]) e compõe o `nfeProc`
//! autorizado. Extraído do `mod.rs` na fase A7 (§2.1).

use crate::error::{DfeError, Result};
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::interno::ws::nfe_autorizacao;

use super::flag::{FlagAutorizacao, FlagAutorizacaoEnum};
use super::parser::xml_result;
use super::types::{NFeInterno, Response};
use super::xml::build_signed_xml;

pub(super) async fn emit_nfe(nfe: NFeInterno) -> Result<Response> {
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

    // A6: UF real derivada do cUF do ide (antes fixo em "SP"). O QR Code da NFC-e (build_signed_xml)
    // ainda usa URL de SP — dataset de consulta QR por UF é follow-up separado (ver CLAUDE.md).
    let uf = crate::interno::uf::sigla_por_codigo(&format!("{:02}", signed.ide_c_uf))?;
    let url = nfe_autorizacao(signed.ide_tp_amb, uf, signed.ide_mod, false)?;

    let xml_with_declaration = if xml_envelope.starts_with("<?xml") {
        xml_envelope.clone()
    } else {
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml_envelope)
    };

    // Envelope efetivamente enviado — devolvido na Response (send_xml), não gravado no CWD.
    let send_xml = xml_with_declaration.clone();

    // O transporte trata status HTTP != 2xx como DfeError::Webservice.
    let receive_xml = MtlsTransport::new(&signed.cert_path, &signed.cert_pass)
        .send_soap(url, xml_with_declaration)
        .await?;

    let mut result = xml_result(&receive_xml, signed.validated_xml)?;
    result.send_xml = send_xml;
    result.receive_xml = receive_xml;
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
    Ok(Response {
        protocolo: result.protocolo,
        xml: nfe_proc_xml.replace("\\", ""),
        send_xml: result.send_xml,
        receive_xml: result.receive_xml,
    })
}
