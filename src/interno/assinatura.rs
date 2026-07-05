//! Montagem do XML da assinatura XML-DSig (RSA-SHA1): `<SignedInfo>` e `<Signature>`.
//!
//! Unifica (fase A3b) a montagem antes **duplicada** em emissão, cancelamento e manifestação.
//! A saída é **byte-a-byte idêntica** à das implementações anteriores — travada por golden em
//! `cancelar::tests` e em `tests` abaixo. **Qualquer alteração aqui pode invalidar assinaturas
//! na SEFAZ** (o nó `SignedInfo` é exatamente o que é assinado e reverificado pela SEFAZ).
//!
//! A limpeza que simula a canonicalização C14N vem de [`crate::interno::cleaner`].

use crate::interno::cleaner::Strings;

/// Monta o `<SignedInfo>` (o nó assinado): `CanonicalizationMethod` + `SignatureMethod` RSA-SHA1
/// + `Reference` com `Transforms` (enveloped-signature + C14N), `DigestMethod` SHA1 e o `digest`
/// (base64) informado. Retorna já limpo (C14N simulada).
///
/// `reference_uri` é o alvo da assinatura:
/// - emissão de NF-e/NFC-e: `"#NFe{chave}"`;
/// - eventos (cancelamento, manifestação): `"#ID{tpEvento}{chave}{nSeqEvento:02}"`.
pub fn signed_info_xml(reference_uri: &str, digest: &str) -> String {
    let xml = String::new()
        + "<SignedInfo xmlns=\"http://www.w3.org/2000/09/xmldsig#\">"
        + "<CanonicalizationMethod Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></CanonicalizationMethod>"
        + "<SignatureMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#rsa-sha1\"></SignatureMethod>"
        + "<Reference URI=\"" + reference_uri + "\">"
        + "<Transforms>"
        + "<Transform Algorithm=\"http://www.w3.org/2000/09/xmldsig#enveloped-signature\"></Transform>"
        + "<Transform Algorithm=\"http://www.w3.org/TR/2001/REC-xml-c14n-20010315\"></Transform>"
        + "</Transforms>"
        + "<DigestMethod Algorithm=\"http://www.w3.org/2000/09/xmldsig#sha1\"></DigestMethod>"
        + "<DigestValue>" + digest + "</DigestValue>"
        + "</Reference></SignedInfo>";
    Strings::clear_xml_string(&xml)
}

/// Monta o `<Signature>` completo: o `signed_info` já pronto + `<SignatureValue>` + `<KeyInfo>`
/// com o `<X509Certificate>` (base64 do DER do certificado folha). Não limpa de novo — o
/// `signed_info` já vem limpo de [`signed_info_xml`], e o restante não tem espaços a colapsar.
pub fn signature_xml(signed_info: &str, signature_value: &str, x509_cert: &str) -> String {
    String::new()
        + "<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">"
        + signed_info
        + "<SignatureValue>" + signature_value + "</SignatureValue>"
        + "<KeyInfo><X509Data><X509Certificate>" + x509_cert
        + "</X509Certificate></X509Data></KeyInfo></Signature>"
}

#[cfg(test)]
mod tests {
    use super::*;

    // Golden das duas formas de Reference URI usadas na crate — trava a montagem unificada.
    #[test]
    fn golden_signed_info_nfe() {
        let out = signed_info_xml("#NFe35000000000000000000550010000000001000000001", "RGlnZXN0Rml4bw==");
        insta::assert_snapshot!(out);
    }

    #[test]
    fn golden_signed_info_evento() {
        let out = signed_info_xml("#ID1101113500000000000000000055001000000000100000000101", "RGlnZXN0Rml4bw==");
        insta::assert_snapshot!(out);
    }

    #[test]
    fn golden_signature() {
        let out = signature_xml("<SignedInfo>FIXO</SignedInfo>", "U2lnRml4bw==", "Q2VydEZpeG8=");
        insta::assert_snapshot!(out);
    }
}
