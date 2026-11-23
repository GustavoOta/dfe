//! Assinatura XMLDSig da DPS (Manual Integrado do SN NFS-e, §6.1.4).
//!
//! O perfil é o mesmo da NF-e: enveloped, C14N 1.0, RSA-SHA1, digest SHA-1, só o certificado
//! do usuário final no `KeyInfo`, `<Signature xmlns="…xmldsig#">` logo depois do `<infDPS>` e
//! `Reference URI="#" + Id`. Por isso reusa `interno::assinatura` e o CAPI/OpenSSL de
//! `interno::cert`, sem caminho criptográfico próprio.
//!
//! O digest sai do `<infDPS>` na forma canônica: o C14N de um subconjunto declara no elemento
//! raiz o namespace herdado do `<DPS>`. O builder já escreve o resto canônico (texto com escape
//! parcial, nenhum espaço entre tags, nenhum elemento vazio), então a forma canônica é o próprio
//! trecho com o `xmlns` acrescentado — o mesmo que a emissão da NF-e faz com o `<infNFe>`.

use super::{xsd, Ambiente, DpsGerada};
use crate::error::{DfeError, Result};
use crate::interno::assinatura::{signature_xml, signed_info_xml};
use crate::interno::cert::{DigestValue, RawPubKey, Sign};

const NS_NFSE: &str = "http://www.sped.fazenda.gov.br/nfse";
const ABRE_INF: &str = "<infDPS Id=\"";
const FECHA_INF: &str = "</infDPS>";

/// DPS assinada e conferida no XSD do ambiente, pronta para o `POST /nfse`.
///
/// Persistir `ambiente`, `id` e `xml` **antes** do envio: com a resposta perdida, a conciliação
/// é pelo `id` ([`super::NfseClient::consultar_dps`]) e um eventual reenvio usa este mesmo XML
/// ([`DpsAssinada::from_xml`]).
#[derive(Clone, Debug)]
pub struct DpsAssinada {
    ambiente: Ambiente,
    id: String,
    xml: String,
}

impl DpsAssinada {
    pub fn ambiente(&self) -> Ambiente {
        self.ambiente
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn xml(&self) -> &str {
        &self.xml
    }

    /// Reabre uma DPS assinada guardada pelo consumidor (os bytes exatos de [`Self::xml`]).
    ///
    /// Confere a estrutura, o XSD do ambiente lido do `tpAmb` e se o `DigestValue` ainda
    /// corresponde ao `<infDPS>`: XML alterado depois de assinado é recusado. A assinatura RSA
    /// em si quem confere é a SEFIN.
    pub fn from_xml(xml: &str) -> Result<Self> {
        let partes = Partes::de(xml)?;
        let ambiente = match valor_da_tag(partes.inf, "tpAmb") {
            Some("1") => Ambiente::Producao,
            Some("2") => Ambiente::ProducaoRestrita,
            _ => return Err(invalida("tpAmb ausente ou inválido")),
        };
        let resto = &xml[partes.fim_inf..];
        let digest =
            valor_da_tag(resto, "DigestValue").ok_or_else(|| invalida("assinatura ausente"))?;
        if !resto.contains(&format!("<Reference URI=\"#{}\">", partes.id)) {
            return Err(invalida("a assinatura não referencia o infDPS"));
        }
        if DigestValue::sha1(&canonico(partes.inf, partes.id))? != digest {
            return Err(invalida("o infDPS foi alterado depois de assinado"));
        }
        xsd::validar_dps(ambiente, xml)?;
        Ok(Self {
            ambiente,
            id: partes.id.to_string(),
            xml: xml.to_string(),
        })
    }
}

/// Assina a DPS com o A1 (lido do arquivo nesta chamada) e valida o resultado no XSD.
pub(super) async fn assinar(
    dps: &DpsGerada,
    cert_path: &str,
    cert_pass: &str,
) -> Result<DpsAssinada> {
    let partes = Partes::de(&dps.xml)?;
    if partes.id != dps.id {
        return Err(invalida("Id do infDPS diferente do gerado"));
    }
    if dps.xml[partes.fim_inf..].contains("<Signature") {
        return Err(invalida("a DPS já está assinada"));
    }

    let digest = DigestValue::sha1(&canonico(partes.inf, partes.id))?;
    let signed_info = signed_info_xml(&format!("#{}", partes.id), &digest);
    let valor = Sign::xml_string(&signed_info, cert_path, cert_pass).await?;
    let certificado = RawPubKey::get_from_file(cert_path, cert_pass).await?;
    let assinatura = signature_xml(&signed_info, &valor, &certificado);

    let xml = format!(
        "{}{}{}",
        &dps.xml[..partes.fim_inf],
        assinatura,
        &dps.xml[partes.fim_inf..]
    );
    xsd::validar_dps(dps.ambiente, &xml)?;
    Ok(DpsAssinada {
        ambiente: dps.ambiente,
        id: dps.id.clone(),
        xml,
    })
}

/// `<infDPS>` na forma canônica do C14N como subconjunto do documento.
fn canonico(inf: &str, id: &str) -> String {
    let resto = &inf[ABRE_INF.len() + id.len() + 1..];
    format!("<infDPS xmlns=\"{NS_NFSE}\" Id=\"{id}\"{resto}")
}

struct Partes<'a> {
    /// `<infDPS Id="…">…</infDPS>`, como está no documento.
    inf: &'a str,
    id: &'a str,
    /// Posição logo depois do `</infDPS>`, onde entra a `<Signature>`.
    fim_inf: usize,
}

impl<'a> Partes<'a> {
    fn de(xml: &'a str) -> Result<Self> {
        let raiz = format!("<DPS xmlns=\"{NS_NFSE}\" versao=\"");
        if !xml.contains(&raiz) || !xml.trim_end().ends_with("</DPS>") {
            return Err(invalida("raiz <DPS> fora do padrão"));
        }
        let inicio = xml
            .find(ABRE_INF)
            .ok_or_else(|| invalida("infDPS ausente"))?;
        let fim_inf = xml[inicio..]
            .find(FECHA_INF)
            .map(|i| inicio + i + FECHA_INF.len())
            .ok_or_else(|| invalida("infDPS sem fechamento"))?;
        if xml[fim_inf..].contains(ABRE_INF) {
            return Err(invalida("mais de um infDPS"));
        }
        let id_ini = inicio + ABRE_INF.len();
        let id_fim = xml[id_ini..]
            .find("\">")
            .map(|i| id_ini + i)
            .ok_or_else(|| invalida("Id do infDPS"))?;
        let id = &xml[id_ini..id_fim];
        if id.len() != 45
            || !id.starts_with("DPS")
            || !id[3..].bytes().all(|c| c.is_ascii_alphanumeric())
        {
            return Err(invalida("Id do infDPS"));
        }
        Ok(Self {
            inf: &xml[inicio..fim_inf],
            id,
            fim_inf,
        })
    }
}

fn valor_da_tag<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let abre = format!("<{tag}>");
    let ini = xml.find(&abre)? + abre.len();
    let fim = xml[ini..].find(&format!("</{tag}>"))? + ini;
    Some(&xml[ini..fim])
}

fn invalida(motivo: &str) -> DfeError {
    DfeError::Assinatura(format!("DPS: {motivo}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forma_canonica_declara_o_namespace_herdado_antes_do_id() {
        let id = format!("DPS{}", "1".repeat(42));
        let inf = format!("<infDPS Id=\"{id}\"><tpAmb>2</tpAmb></infDPS>");
        assert_eq!(
            canonico(&inf, &id),
            format!("<infDPS xmlns=\"{NS_NFSE}\" Id=\"{id}\"><tpAmb>2</tpAmb></infDPS>")
        );
    }
}
