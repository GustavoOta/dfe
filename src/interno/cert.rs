use base64::{engine::general_purpose::STANDARD, Engine};
use crate::error::{DfeError, Result};
use openssl::sign::Signer;
use openssl::x509::X509;
use p12::PFX;
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

        // reqwest usa native-tls (Windows CAPI) que suporta RC2-40-CBC dos certificados ICP-Brasil
        // O parse via openssl::pkcs12::Pkcs12::parse2() foi removido pois falha com OpenSSL 3.x
        // vendorizado (que não inclui o provider legacy) e o resultado era descartado de qualquer forma
        let pkcs2 = reqwest::Identity::from_pkcs12_der(&buf, password).map_err(|e| {
            DfeError::Certificado(format!(
                "Erro ao ler certificado: Senha inválida ou PFX corrompido: {}", e
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

        // p12 crate: RC2-40-CBC puro-Rust — não depende do provider legacy do OpenSSL
        let pfx = PFX::parse(&der)
            .map_err(|e| DfeError::Certificado(format!("PFX inválido: {:?}", e)))?;
        let certs = pfx
            .cert_x509_bags(password)
            .map_err(|e| DfeError::Certificado(format!("Erro ao extrair certificado: {:?}", e)))?;
        let cert_der = certs
            .into_iter()
            .next()
            .ok_or_else(|| DfeError::Certificado("Certificado não encontrado no PFX".to_string()))?;

        let cert = X509::from_der(&cert_der)?;
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
        let mut pfx_file = File::open(pfx_path)?;
        pfx_file.read_to_end(&mut buf)?;

        // p12 crate: extrai a chave privada descriptografando RC2-40-CBC em Rust puro
        let pfx = PFX::parse(&buf)
            .map_err(|e| DfeError::Assinatura(format!("PFX inválido: {:?}", e)))?;
        let keys = pfx
            .key_bags(password)
            .map_err(|e| DfeError::Assinatura(format!("Erro ao extrair chave privada: {:?}", e)))?;
        let key_der = keys
            .into_iter()
            .next()
            .ok_or_else(|| DfeError::Assinatura("Chave privada não encontrada no PFX".to_string()))?;

        // Carrega a chave RSA no OpenSSL — não usa algoritmos legados nesta etapa
        let pkey = openssl::pkey::PKey::private_key_from_pkcs8(&key_der)
            .or_else(|_| openssl::pkey::PKey::private_key_from_der(&key_der))
            .map_err(|e| DfeError::Assinatura(format!("Erro ao carregar chave privada: {}", e)))?;

        let mut signer = Signer::new(openssl::hash::MessageDigest::sha1(), &pkey)?;
        signer.update(data.as_bytes())?;

        let signature = signer.sign_to_vec()?;
        Ok(STANDARD.encode(&signature))
    }
}
