//! Orquestração da emissão: valida a flag, monta+assina (via [`super::xml`]), envia (via
//! [`crate::interno::transporte`]), parseia (via [`super::parser`]) e compõe o `nfeProc`
//! autorizado. Extraído do `mod.rs` na fase A7 (§2.1).

use crate::error::{DfeError, Result};
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::interno::ws::nfe_autorizacao;

use super::flag::{FlagAutorizacao, FlagAutorizacaoEnum};
use super::parser::xml_result;
use super::types::{NFeAssinada, NFeInterno, Response};
use super::xml::build_signed_xml;

/// Monta, assina e valida — sem tocar na rede. A chave sai do `Id` do próprio XML assinado
/// (nunca recalculada a partir dos dados de entrada: o `cNF` é sorteado durante a montagem,
/// então recalcular daria uma chave diferente da que foi assinada).
pub(super) async fn assinar_nfe(nfe: NFeInterno) -> Result<NFeAssinada> {
    let signed = build_signed_xml(nfe).await?;
    let chave = extrair_chave(&signed.validated_xml)?;
    Ok(NFeAssinada {
        chave,
        xml: signed.validated_xml,
        cert_path: signed.cert_path,
        cert_pass: signed.cert_pass,
        ide_mod: signed.ide_mod,
        ide_tp_amb: signed.ide_tp_amb,
        ide_c_uf: signed.ide_c_uf,
    })
}

/// Lê a chave de acesso do atributo `Id` do `infNFe` (`Id="NFe{44 posições}"`).
fn extrair_chave(xml: &str) -> Result<String> {
    let marcador = "Id=\"NFe";
    let inicio = xml
        .find(marcador)
        .ok_or_else(|| DfeError::Xml("Id do infNFe não encontrado no XML assinado".to_string()))?
        + marcador.len();
    let chave: String = xml[inicio..].chars().take(44).collect();
    if chave.len() != 44 || !chave.chars().all(|c| c.is_ascii_digit()) {
        return Err(DfeError::Xml(format!(
            "chave de acesso inválida no Id do infNFe: [{chave}]"
        )));
    }
    Ok(chave)
}

pub(super) async fn transmitir_assinada(assinada: &NFeAssinada) -> Result<Response> {
    let flag = FlagAutorizacao::start().await.map_err(DfeError::Validacao)?;
    match flag {
        FlagAutorizacaoEnum::Ready => {}
        _ => return Err(DfeError::Validacao(format!(
            "Flag de autorização inválida para emissão: [{:?}].", flag
        ))),
    }

    let id_lote = 100;
    let xml_envelope = format!(
        r#"<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4"><enviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><idLote>{}</idLote><indSinc>1</indSinc>{}</enviNFe></nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        id_lote, &assinada.xml
    );

    // A6: UF real derivada do cUF do ide (antes fixo em "SP"). O QR Code da NFC-e (build_signed_xml)
    // ainda usa URL de SP — dataset de consulta QR por UF é follow-up separado (ver CLAUDE.md).
    let uf = crate::interno::uf::sigla_por_codigo(&format!("{:02}", assinada.ide_c_uf))?;
    let url = nfe_autorizacao(assinada.ide_tp_amb, uf, assinada.ide_mod, false)?;

    let xml_with_declaration = if xml_envelope.starts_with("<?xml") {
        xml_envelope.clone()
    } else {
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml_envelope)
    };

    // Envelope efetivamente enviado — devolvido na Response (send_xml), não gravado no CWD.
    let send_xml = xml_with_declaration.clone();

    // O transporte trata status HTTP != 2xx como DfeError::Webservice.
    let receive_xml = MtlsTransport::new(&assinada.cert_path, &assinada.cert_pass)
        .send_soap(url, xml_with_declaration)
        .await?;

    let mut result = xml_result(&receive_xml, assinada.xml.clone())?;
    result.send_xml = send_xml;
    result.receive_xml = receive_xml;
    Ok(montar_resposta_autorizada(result))
}

pub(super) async fn emit_nfe(nfe: NFeInterno) -> Result<Response> {
    let assinada = assinar_nfe(nfe).await?;
    transmitir_assinada(&assinada).await
}

// cStat 100 = autorizado · 150 = autorizado fora do prazo (§5.7 do CONTINGENCIA_NFCE.md) — mesma
// convenção já usada pelo gravisServer/ACBr. Qualquer outro cStat é rejeição e não monta o nfeProc.
fn autorizada(c_stat: i32) -> bool {
    c_stat == 100 || c_stat == 150
}

// Envolve o `<NFe>` assinado com `<protNFe>`/`<nfeProc>` quando a SEFAZ autorizou (cStat 100/150).
// Compartilhado entre `emit_nfe` (emissão online) e `transmitir_xml_assinado` (transmissão tardia
// de contingência) para não duplicar a montagem do protocolo.
fn montar_resposta_autorizada(result: Response) -> Response {
    if !autorizada(result.protocolo.inf_prot.c_stat) {
        return result;
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
    Response {
        protocolo: result.protocolo,
        xml: nfe_proc_xml.replace("\\", ""),
        send_xml: result.send_xml,
        receive_xml: result.receive_xml,
    }
}

/// Transmite à SEFAZ um XML de NFC-e **já assinado** (ex.: gerado por
/// [`super::NFeBuilder::gerar_xml`] com `tp_emis=9`, contingência off-line), sem reconstruir nem
/// re-assinar — a chave de acesso e a assinatura devem ser exatamente as que já foram impressas
/// para o cliente no momento da emissão em contingência.
///
/// Usar quando a conexão com a SEFAZ volta, para transmitir notas pendentes de autorização.
/// `cStat 100` e `150` (autorizado fora do prazo) são tratados como sucesso — ver §5.7 do
/// `CONTINGENCIA_NFCE.md`.
///
/// `c_uf` é o código IBGE da UF do emitente (o mesmo `Ide.c_uf` usado na emissão original).
/// Contingência off-line é exclusiva de NFC-e — o serviço de autorização é sempre o do modelo 65.
pub async fn transmitir_xml_assinado(
    xml_assinado: &str,
    cert_path: &str,
    cert_pass: &str,
    tp_amb: u8,
    c_uf: u16,
) -> Result<Response> {
    let id_lote = 100;
    let xml_envelope = format!(
        r#"<soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4"><enviNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><idLote>{}</idLote><indSinc>1</indSinc>{}</enviNFe></nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        id_lote, xml_assinado
    );

    let uf = crate::interno::uf::sigla_por_codigo(&format!("{:02}", c_uf))?;
    let url = nfe_autorizacao(tp_amb, uf, 65, false)?;

    let xml_with_declaration = if xml_envelope.starts_with("<?xml") {
        xml_envelope.clone()
    } else {
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", xml_envelope)
    };

    let send_xml = xml_with_declaration.clone();

    let receive_xml = MtlsTransport::new(cert_path, cert_pass)
        .send_soap(url, xml_with_declaration)
        .await?;

    let mut result = xml_result(&receive_xml, xml_assinado.to_string())?;
    result.send_xml = send_xml;
    result.receive_xml = receive_xml;
    Ok(montar_resposta_autorizada(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAVE: &str = "35260911222333000181650010000003251000000325";

    #[test]
    fn extrai_chave_do_id_do_inf_nfe() {
        let xml = format!(
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00" Id="NFe{CHAVE}"><ide></ide></infNFe></NFe>"#
        );
        assert_eq!(extrair_chave(&xml).unwrap(), CHAVE);
    }

    #[test]
    fn rejeita_xml_sem_id() {
        let err = extrair_chave("<NFe><infNFe versao=\"4.00\"></infNFe></NFe>").unwrap_err();
        assert!(matches!(err, DfeError::Xml(_)));
    }

    // Truncado: sem isto, um XML cortado devolveria uma "chave" curta que só seria descoberta
    // como inválida lá na frente, no ledger do consumidor.
    #[test]
    fn rejeita_chave_incompleta() {
        let err = extrair_chave(r#"<infNFe Id="NFe3526091122233300018165001">"#).unwrap_err();
        assert!(matches!(err, DfeError::Xml(_)));
    }
}
