//! Certificado A1 (PKCS#12 / `.pfx`) fora do Windows, pelo OpenSSL do sistema.
//!
//! No Windows quem faz isso é o CAPI (`cert.rs`); aqui o resultado tem de sair **igual** ao de
//! lá, porque os consumidores leem o texto: o `subject` é só o valor do CN
//! (`EMPRESA:CNPJ`, de onde o CNPJ do titular é tirado), o `issuer` é o CN da AC e as datas são
//! `AAAA/MM/DD` em UTC. Os testes de `cert.rs` comparam os dois lados com os mesmos PFX
//! sintéticos de `tests/fixtures/cert/`.

use std::sync::OnceLock;

use openssl::asn1::{Asn1Time, Asn1TimeRef};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkcs12::Pkcs12;
use openssl::pkey::{PKey, Private};
use openssl::provider::Provider;
use openssl::rsa::Padding;
use openssl::sign::Signer;
use openssl::x509::{X509NameRef, X509};

use super::cert::CertInfo;

/// Carrega o provider `legacy` do OpenSSL 3 uma vez por processo e o mantém carregado.
///
/// Os A1 ICP-Brasil mais antigos vêm cifrados em RC2-40/3DES, que o OpenSSL 3 só abre com esse
/// provider. `retain_fallbacks = true` mantém o `default` (que carregar um provider à mão
/// desligaria). Vale para o processo inteiro, então cobre também o mTLS do `native-tls`.
/// Devolve `false` quando o sistema não tem o módulo (`ossl-modules/legacy.so`): PFX moderno
/// (AES) continua abrindo; o legado falha com aviso.
pub fn garantir_provider_legacy() -> bool {
    static LEGACY: OnceLock<Option<Provider>> = OnceLock::new();
    LEGACY
        .get_or_init(|| Provider::try_load(None, "legacy", true).ok())
        .is_some()
}

/// Abre o PFX com a senha e devolve a chave privada e o certificado **ligado a ela** (o da
/// empresa — a AC da cadeia vem à parte e é descartada).
pub fn abrir_pfx(pfx_bytes: &[u8], password: &str) -> Result<(PKey<Private>, X509), String> {
    let legacy = garantir_provider_legacy();
    let parsed = Pkcs12::from_der(pfx_bytes)
        .and_then(|p| p.parse2(password))
        .map_err(|_| {
            if legacy {
                "Senha inválida ou PFX corrompido".to_string()
            } else {
                "Senha inválida ou PFX corrompido (o OpenSSL desta máquina não tem o provider \
                 legacy, necessário para PFX em RC2/3DES)"
                    .to_string()
            }
        })?;
    match (parsed.pkey, parsed.cert) {
        (Some(pkey), Some(cert)) => Ok((pkey, cert)),
        _ => Err("Nenhum certificado com chave privada encontrado no PFX".to_string()),
    }
}

/// Metadados do certificado da empresa, no mesmo formato do CAPI.
pub fn ler_info(pfx_bytes: &[u8], password: &str) -> Result<CertInfo, String> {
    let (_, cert) = abrir_pfx(pfx_bytes, password)?;
    Ok(CertInfo {
        subject: cn(cert.subject_name()),
        issuer: cn(cert.issuer_name()),
        valid_from: data(cert.not_before())?,
        valid_to: data(cert.not_after())?,
    })
}

/// DER do certificado da empresa — vai em base64 no `<X509Certificate>` do `KeyInfo`.
pub fn extrair_cert_der(pfx_bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let (_, cert) = abrir_pfx(pfx_bytes, password)?;
    cert.to_der().map_err(|e| format!("Falha ao codificar o certificado: {e}"))
}

/// RSA-SHA1 com padding PKCS#1 v1.5 (o `SignatureMethod` exigido pela SEFAZ). Determinístico:
/// mesmos bytes + mesma chave = mesma assinatura do CAPI.
pub fn assinar_rsa_sha1(data: &[u8], pfx_bytes: &[u8], password: &str) -> Result<Vec<u8>, String> {
    let (pkey, _) = abrir_pfx(pfx_bytes, password)?;
    let mut signer = Signer::new(MessageDigest::sha1(), &pkey)
        .map_err(|e| format!("Falha ao preparar a assinatura: {e}"))?;
    signer
        .set_rsa_padding(Padding::PKCS1)
        .map_err(|e| format!("Falha ao preparar a assinatura: {e}"))?;
    signer
        .update(data)
        .and_then(|_| signer.sign_to_vec())
        .map_err(|e| format!("Falha ao assinar: {e}"))
}

/// Primeiro CN do nome — o que o `CertGetNameStringW` do Windows devolve nesta crate.
fn cn(nome: &X509NameRef) -> String {
    nome.entries_by_nid(Nid::COMMONNAME)
        .next()
        .and_then(|e| e.data().to_string().ok())
        .unwrap_or_default()
}

/// `AAAA/MM/DD` em UTC, como o `FileTimeToSystemTime` do lado Windows.
fn data(t: &Asn1TimeRef) -> Result<String, String> {
    let epoch = Asn1Time::from_unix(0).map_err(|e| format!("Data do certificado: {e}"))?;
    let d = epoch.diff(t).map_err(|e| format!("Data do certificado: {e}"))?;
    let segundos = i64::from(d.days) * 86_400 + i64::from(d.secs);
    chrono::DateTime::from_timestamp(segundos, 0)
        .map(|dt| dt.format("%Y/%m/%d").to_string())
        .ok_or_else(|| "Data do certificado fora do intervalo".to_string())
}
