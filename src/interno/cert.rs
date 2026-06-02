use base64::{engine::general_purpose::STANDARD, Engine};
use crate::error::{DfeError, Result};
use openssl::pkcs12::Pkcs12;
use openssl::sign::Signer;
use openssl::x509::X509;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Read;

pub struct Cert {
    pub identity: reqwest::Identity,
}

/// Certificado digital em formato PKCS12 (PFX)
/// Uso: Cert::from_pfx("caminho/para/o/certificado.pfx", "senha")
impl Cert {
    pub fn from_pfx(path: &str, password: &str) -> Result<Cert> {
        let mut buf = Vec::new();
        let mut pfx = File::open(path)?;
        pfx.read_to_end(&mut buf)?;

        Pkcs12::from_der(&buf)
            .map_err(|e| DfeError::Certificado(format!(
                "Arquivo de certificado inválido (não é PKCS12/PFX válido): {}", e
            )))?
            .parse2(password)
            .map_err(|e| DfeError::Certificado(format!(
                "Falha ao abrir o PFX. Verifique a senha do certificado: {}", e
            )))?;

        let pkcs2 = reqwest::Identity::from_pkcs12_der(&buf, password).map_err(|e| {
            DfeError::Certificado(format!(
                "Falha ao criar identidade TLS a partir do PFX (backend reqwest/native-tls): {}", e
            ))
        })?;

        Ok(Cert { identity: pkcs2 })
    }
}


pub struct DigestValue;

impl DigestValue {
    pub fn sha1(xml: &str) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(xml.as_bytes());
        let result = hasher.finalize();
        Ok(STANDARD.encode(&result))
    }

}

pub struct RawPubKey;

impl RawPubKey {
    pub async fn get_from_file(pfx_path: &str, password: &str) -> Result<String> {
        let mut file = File::open(pfx_path)?;
        let mut der = vec![];
        file.read_to_end(&mut der)?;

        let pkcs12 = Pkcs12::from_der(&der)?;
        let x509: openssl::pkcs12::ParsedPkcs12_2 = pkcs12.parse2(password)?;

        let cert = x509.cert
            .ok_or_else(|| DfeError::Certificado("Certificado não encontrado".to_string()))?;
        let cert = X509::from_der(&cert.to_der()?)?;
        let cert = cert.to_pem()?;
        let cert = String::from_utf8(cert)?;
        let cert = cert.replace("-----BEGIN CERTIFICATE-----", "");
        let cert = cert.replace("-----END CERTIFICATE-----", "");
        let cert = cert.replace("\n", "");
        Ok(cert)
    }
}

pub struct Sign;

impl Sign {
    pub async fn xml_string(data: &str, pfx_path: &str, password: &str) -> Result<String> {
        let mut buf = Vec::new();
        let mut pfx = File::open(pfx_path)?;
        pfx.read_to_end(&mut buf)?;

        let pkcs12 = Pkcs12::from_der(&buf)?;
        let pkey = pkcs12.parse2(password)?;
        let pkey = pkey.pkey
            .ok_or_else(|| DfeError::Assinatura("Chave privada não encontrada".to_string()))?;

        let mut signer = Signer::new(openssl::hash::MessageDigest::sha1(), &pkey)?;
        signer.update(data.as_bytes())?;

        let signature = signer.sign_to_vec()?;
        Ok(STANDARD.encode(&signature))
    }
}

