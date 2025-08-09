use anyhow::{Error, Ok, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use openssl::pkcs12::Pkcs12;
use openssl::sign::Signer;
use openssl::x509::X509;
use serde::{Deserialize, Serialize};
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

        let pkcs2 = reqwest::Identity::from_pkcs12_der(&buf, password)?;

        Ok(Cert { identity: pkcs2 })
    }
}

pub async fn raw_private_key(pfx_path: &str, password: &str) -> Result<String> {
    let mut file = File::open(pfx_path)?;
    let mut der = vec![];
    file.read_to_end(&mut der)?;

    // generate a X509Certificate from the PFX file
    let pkcs12 = Pkcs12::from_der(&der)?;
    let x509: openssl::pkcs12::ParsedPkcs12_2 = pkcs12.parse2(password)?;

    // print the private key
    let pkey = x509.pkey;
    if pkey.is_none() {
        return Err(anyhow::anyhow!("Chave privada não encontrada"));
    }
    let pkey = pkey.unwrap();
    let pkey = pkey.private_key_to_pem_pkcs8()?;
    let pkey = String::from_utf8(pkey)?;
    // clean the private key
    let pkey = pkey.replace("-----BEGIN PRIVATE KEY-----", "");
    let pkey = pkey.replace("-----END PRIVATE KEY-----", "");
    let pkey = pkey.replace("\n", "");
    Ok(pkey)
}

pub struct DigestValue;

impl DigestValue {
    pub fn sha1(xml: &str) -> Result<String, Error> {
        let mut hasher = Sha1::new();
        hasher.update(xml.as_bytes());
        let result = hasher.finalize();
        let digest_value = STANDARD.encode(&result);
        Ok(digest_value)
    }

    pub fn sha2(xml: &str) -> Result<String, Error> {
        let mut hasher = openssl::sha::Sha256::new();
        hasher.update(xml.as_bytes());
        let result = hasher.finish();
        let digest_value = STANDARD.encode(&result);
        Ok(digest_value)
    }
}

pub struct RawPubKey;

impl RawPubKey {
    pub async fn get_from_file(pfx_path: &str, password: &str) -> Result<String, Error> {
        let mut file = File::open(pfx_path)?;
        let mut der = vec![];
        file.read_to_end(&mut der)?;

        // generate a X509Certificate from the PFX file
        let pkcs12 = Pkcs12::from_der(&der)?;
        let x509: openssl::pkcs12::ParsedPkcs12_2 = pkcs12.parse2(password)?;

        // print the certificate
        let cert = x509.cert;
        if cert.is_none() {
            return Err(anyhow::anyhow!("Certificado não encontrado"));
        }
        let cert = cert.unwrap();
        let cert = X509::from_der(&cert.to_der()?)?;
        let cert = cert.to_pem()?;
        let cert = String::from_utf8(cert)?;
        // clean the certificate
        let cert = cert.replace("-----BEGIN CERTIFICATE-----", "");
        let cert = cert.replace("-----END CERTIFICATE-----", "");
        let cert = cert.replace("\n", "");
        Ok(cert)
    }
}

pub struct Sign;

impl Sign {
    pub async fn xml_string(data: &str, pfx_path: &str, password: &str) -> Result<String, Error> {
        let mut buf = Vec::new();
        let mut pfx = File::open(pfx_path)?;
        pfx.read_to_end(&mut buf)?;

        let pkcs12 = Pkcs12::from_der(&buf)?;
        let pkey = pkcs12.parse2(password)?;
        let pkey = pkey.pkey.expect("Chave privada não encontrada");

        let mut signer = Signer::new(openssl::hash::MessageDigest::sha1(), &pkey)?;
        signer.update(data.as_bytes())?;

        let signature = signer.sign_to_vec()?;
        let signature = STANDARD.encode(&signature);

        Ok(signature)
    }
}

// display certificate information
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub valid_from: String,
    pub valid_to: String,
}
impl CertificateInfo {
    pub fn from_pfx(pfx_path: &str, password: &str) -> Result<CertificateInfo, Error> {
        let mut file = File::open(pfx_path)?;
        let mut der = vec![];
        file.read_to_end(&mut der)?;

        let pkcs12 = Pkcs12::from_der(&der)?;
        let x509: openssl::pkcs12::ParsedPkcs12_2 = pkcs12.parse2(password)?;

        let cert = x509
            .cert
            .ok_or_else(|| anyhow::anyhow!("Certificado não encontrado"))?;
        let cert = X509::from_der(&cert.to_der()?)?;
        let subject = cert
            .subject_name()
            .entries()
            .fold(String::new(), |mut acc, entry| {
                if !acc.is_empty() {
                    acc.push_str(", ");
                }
                acc.push_str(&format!(
                    "{}={}",
                    entry.object().nid().short_name().unwrap_or(""),
                    entry.data().as_utf8().unwrap()
                ));
                acc
            });
        let issuer = cert
            .issuer_name()
            .entries()
            .fold(String::new(), |mut acc, entry| {
                if !acc.is_empty() {
                    acc.push_str(", ");
                }
                acc.push_str(&format!(
                    "{}={}",
                    entry.object().nid().short_name().unwrap_or(""),
                    entry.data().as_utf8().unwrap()
                ));
                acc
            });
        let valid_from = cert.not_before().to_string();
        let valid_to = cert.not_after().to_string();
        Ok(CertificateInfo {
            subject,
            issuer,
            valid_from,
            valid_to,
        })
    }
}
