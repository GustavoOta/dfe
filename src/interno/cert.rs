use base64::{engine::general_purpose::STANDARD, Engine};
use crate::error::{DfeError, Result};
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

        // reqwest usa native-tls (Windows CAPI) que suporta RC2/3DES dos certificados ICP-Brasil
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

        // Windows CAPI: abre o PFX suportando RC2/3DES dos certificados ICP-Brasil
        // PEM = base64(DER), então encodificamos diretamente sem passar pelo OpenSSL
        let cert_der = pfx_capi_extract_cert_der(&der, password)
            .map_err(|e| DfeError::Certificado(format!("Erro ao extrair certificado: {}", e)))?;

        Ok(STANDARD.encode(&cert_der))
    }
}

pub struct Sign;

impl Sign {
    pub async fn xml_string(data: &str, pfx_path: &str, password: &str) -> Result<String> {
        let mut buf = Vec::new();
        let mut pfx_file = File::open(pfx_path)?;
        pfx_file.read_to_end(&mut buf)?;

        // Windows CAPI: assina RSA-SHA1 diretamente, sem precisar extrair a chave privada
        // O CAPI suporta RC2/3DES dos certificados ICP-Brasil nativamente
        let signature = pfx_capi_sign_rsa_sha1(data.as_bytes(), &buf, password)
            .map_err(|e| DfeError::Assinatura(format!("Erro ao assinar XML: {}", e)))?;

        Ok(STANDARD.encode(&signature))
    }
}

/// Abre o PFX via Windows CAPI e retorna os bytes DER do certificado folha.
/// Suporta nativamente RC2-40-CBC e 3DES presentes em certificados ICP-Brasil.
#[cfg(target_os = "windows")]
fn pfx_capi_extract_cert_der(pfx_bytes: &[u8], password: &str) -> std::result::Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CertCloseStore, CertEnumCertificatesInStore, CertFreeCertificateContext,
        PFXImportCertStore, CRYPT_INTEGER_BLOB,
    };

    const PKCS12_NO_PERSIST_KEY: u32 = 0x00008000;

    let pw_wide: Vec<u16> = password.encode_utf16().chain(std::iter::once(0)).collect();
    let pfx_blob = CRYPT_INTEGER_BLOB {
        cbData: pfx_bytes.len() as u32,
        pbData: pfx_bytes.as_ptr() as *mut u8,
    };

    unsafe {
        let store = PFXImportCertStore(&pfx_blob, pw_wide.as_ptr(), PKCS12_NO_PERSIST_KEY);
        if store.is_null() {
            return Err("Senha inválida ou PFX corrompido".to_string());
        }

        let cert_ctx = CertEnumCertificatesInStore(store, std::ptr::null());
        let result = if cert_ctx.is_null() {
            Err("Nenhum certificado encontrado no PFX".to_string())
        } else {
            let der = std::slice::from_raw_parts(
                (*cert_ctx).pbCertEncoded,
                (*cert_ctx).cbCertEncoded as usize,
            )
            .to_vec();
            CertFreeCertificateContext(cert_ctx);
            Ok(der)
        };

        CertCloseStore(store, 0);
        result
    }
}

/// Assina `data` com RSA-SHA1 usando a chave privada do PFX via Windows CAPI.
/// CAPI suporta nativamente RC2/3DES dos certificados ICP-Brasil.
/// Nota: CAPI retorna assinatura RSA em little-endian; revertemos para big-endian (PKCS#1 padrão).
#[cfg(target_os = "windows")]
fn pfx_capi_sign_rsa_sha1(data: &[u8], pfx_bytes: &[u8], password: &str) -> std::result::Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CertCloseStore, CertEnumCertificatesInStore, CertFreeCertificateContext,
        CryptAcquireCertificatePrivateKey, CryptCreateHash, CryptDestroyHash, CryptHashData,
        CryptReleaseContext, CryptSignHashW, PFXImportCertStore, CRYPT_INTEGER_BLOB,
    };

    const PKCS12_NO_PERSIST_KEY: u32 = 0x00008000;
    const CRYPT_EXPORTABLE: u32 = 0x00000001;
    const CALG_SHA1: u32 = 0x00008004;

    let pw_wide: Vec<u16> = password.encode_utf16().chain(std::iter::once(0)).collect();
    let pfx_blob = CRYPT_INTEGER_BLOB {
        cbData: pfx_bytes.len() as u32,
        pbData: pfx_bytes.as_ptr() as *mut u8,
    };

    unsafe {
        let store = PFXImportCertStore(
            &pfx_blob,
            pw_wide.as_ptr(),
            PKCS12_NO_PERSIST_KEY | CRYPT_EXPORTABLE,
        );
        if store.is_null() {
            return Err("Senha inválida ou PFX corrompido".to_string());
        }

        let cert_ctx = CertEnumCertificatesInStore(store, std::ptr::null());
        if cert_ctx.is_null() {
            CertCloseStore(store, 0);
            return Err("Nenhum certificado encontrado no PFX".to_string());
        }

        let mut h_prov: usize = 0;
        let mut dw_key_spec: u32 = 0;
        let mut b_free_prov: i32 = 0;

        let ok = CryptAcquireCertificatePrivateKey(
            cert_ctx,
            0,
            std::ptr::null(),
            &mut h_prov,
            &mut dw_key_spec,
            &mut b_free_prov,
        );

        if ok == 0 {
            CertFreeCertificateContext(cert_ctx);
            CertCloseStore(store, 0);
            return Err("Chave privada não acessível".to_string());
        }

        let mut h_hash: usize = 0;
        if CryptCreateHash(h_prov, CALG_SHA1, 0, 0, &mut h_hash) == 0 {
            if b_free_prov != 0 { CryptReleaseContext(h_prov, 0); }
            CertFreeCertificateContext(cert_ctx);
            CertCloseStore(store, 0);
            return Err("Erro ao criar contexto de hash SHA1".to_string());
        }

        CryptHashData(h_hash, data.as_ptr(), data.len() as u32, 0);

        // Primeira chamada: obtém o tamanho da assinatura
        let mut sig_len: u32 = 0;
        CryptSignHashW(h_hash, dw_key_spec, std::ptr::null(), 0, std::ptr::null_mut(), &mut sig_len);

        // Segunda chamada: assina de fato
        let mut sig = vec![0u8; sig_len as usize];
        let ok = CryptSignHashW(h_hash, dw_key_spec, std::ptr::null(), 0, sig.as_mut_ptr(), &mut sig_len);
        sig.truncate(sig_len as usize);

        CryptDestroyHash(h_hash);
        if b_free_prov != 0 { CryptReleaseContext(h_prov, 0); }
        CertFreeCertificateContext(cert_ctx);
        CertCloseStore(store, 0);

        if ok == 0 {
            return Err("Erro ao assinar dados com a chave privada".to_string());
        }

        // CAPI retorna assinatura RSA em little-endian; PKCS#1 padrão é big-endian
        sig.reverse();

        Ok(sig)
    }
}

#[cfg(not(target_os = "windows"))]
fn pfx_capi_extract_cert_der(_pfx_bytes: &[u8], _password: &str) -> std::result::Result<Vec<u8>, String> {
    Err("Extração de certificado via CAPI disponível apenas no Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
fn pfx_capi_sign_rsa_sha1(_data: &[u8], _pfx_bytes: &[u8], _password: &str) -> std::result::Result<Vec<u8>, String> {
    Err("Assinatura via CAPI disponível apenas no Windows".to_string())
}
