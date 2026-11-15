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

        // reqwest usa native-tls: no Windows é o SChannel/CAPI, que lê RC2/3DES dos certificados
        // ICP-Brasil; no Linux é o OpenSSL, que só lê com o provider `legacy` já carregado.
        #[cfg(not(target_os = "windows"))]
        super::cert_openssl::garantir_provider_legacy();
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

/// Metadados legíveis de um certificado digital A1 (PKCS#12 / `.pfx`).
///
/// Refere-se ao certificado **folha** (o da empresa), nunca à CA da cadeia.
/// Não expõe a chave privada. Datas no formato `AAAA/MM/DD`.
#[derive(Debug, Clone)]
pub struct CertInfo {
    pub subject: String,
    pub issuer: String,
    pub valid_from: String,
    pub valid_to: String,
}

impl CertInfo {
    /// Lê os metadados do certificado folha de um `.pfx` em disco.
    ///
    /// Importa o PFX via Windows CAPI **persistindo** a chave (apagada logo após a
    /// leitura). Isso é proposital: o import efêmero (`PKCS12_NO_PERSIST_KEY`) falha
    /// para PFX ICP-Brasil (RC2/3DES) em máquinas onde o certificado não foi instalado
    /// manualmente — justamente o cenário de um PDV recém-instalado. O caminho
    /// persistido funciona sem o cert instalado e não deixa keyset órfão em disco.
    pub fn from_pfx(path: &str, password: &str) -> Result<CertInfo> {
        let mut buf = Vec::new();
        let mut pfx = File::open(path)?;
        pfx.read_to_end(&mut buf)?;
        pfx_capi_read_info(&buf, password).map_err(DfeError::Certificado)
    }
}

/// Identifica o certificado folha (da empresa) num store importado do PFX.
///
/// O PFX contém a empresa + a cadeia de CAs; `CertEnumCertificatesInStore` não garante
/// ordem e pode retornar a CA primeiro (ex.: "AC SyngularID"). O folha é o único que tem
/// material de chave privada associado — detectado pelas propriedades que o
/// `PFXImportCertStore` grava nele: `CERT_KEY_PROV_INFO_PROP_ID` (chave persistida) ou
/// `CERT_KEY_CONTEXT_PROP_ID` (chave efêmera/NO_PERSIST). Não depende de a chave estar
/// de fato acessível, então funciona mesmo sem o certificado instalado no Windows.
#[cfg(target_os = "windows")]
unsafe fn cert_has_private_key(
    cert_ctx: *const windows_sys::Win32::Security::Cryptography::CERT_CONTEXT,
) -> bool {
    use windows_sys::Win32::Security::Cryptography::CertGetCertificateContextProperty;
    const CERT_KEY_PROV_INFO_PROP_ID: u32 = 2;
    const CERT_KEY_CONTEXT_PROP_ID: u32 = 5;

    let mut cb: u32 = 0;
    if CertGetCertificateContextProperty(cert_ctx, CERT_KEY_PROV_INFO_PROP_ID, std::ptr::null_mut(), &mut cb) != 0
        && cb > 0
    {
        return true;
    }
    let mut cb2: u32 = 0;
    CertGetCertificateContextProperty(cert_ctx, CERT_KEY_CONTEXT_PROP_ID, std::ptr::null_mut(), &mut cb2) != 0
        && cb2 > 0
}

/// Percorre o store e devolve o contexto do certificado folha (com chave privada).
/// Retorna ponteiro nulo se nenhum for encontrado. O contexto retornado deve ser
/// liberado pelo chamador com `CertFreeCertificateContext`.
#[cfg(target_os = "windows")]
unsafe fn find_leaf_cert(
    store: *mut std::ffi::c_void,
) -> *const windows_sys::Win32::Security::Cryptography::CERT_CONTEXT {
    use windows_sys::Win32::Security::Cryptography::CertEnumCertificatesInStore;
    let mut cert_ctx = std::ptr::null();
    loop {
        cert_ctx = CertEnumCertificatesInStore(store, cert_ctx);
        if cert_ctx.is_null() {
            return std::ptr::null();
        }
        if cert_has_private_key(cert_ctx) {
            return cert_ctx;
        }
        // Sem chave (CA): o contexto atual é liberado automaticamente ao ser passado
        // como pPrev na próxima iteração de CertEnumCertificatesInStore.
    }
}

/// Apaga o keyset persistido pelo `PFXImportCertStore` para o certificado folha.
/// Necessário porque assinamos importando com a chave persistida (única forma de
/// `CryptAcquireCertificatePrivateKey` achar a chave sem o certificado instalado no
/// Windows); sem apagar, cada emissão deixaria um keyset órfão em disco.
#[cfg(target_os = "windows")]
unsafe fn delete_key_container(
    cert_ctx: *const windows_sys::Win32::Security::Cryptography::CERT_CONTEXT,
) {
    use windows_sys::Win32::Security::Cryptography::{
        CertGetCertificateContextProperty, CryptAcquireContextW, NCryptDeleteKey,
        NCryptFreeObject, NCryptOpenKey, NCryptOpenStorageProvider, CRYPT_KEY_PROV_INFO,
    };
    const CERT_KEY_PROV_INFO_PROP_ID: u32 = 2;
    const CRYPT_DELETEKEYSET: u32 = 0x00000010;
    const NCRYPT_SILENT_FLAG: u32 = 0x00000040;

    let mut cb: u32 = 0;
    if CertGetCertificateContextProperty(cert_ctx, CERT_KEY_PROV_INFO_PROP_ID, std::ptr::null_mut(), &mut cb) == 0
        || cb == 0
    {
        // Sem prov info persistida (ex.: import NO_PERSIST) — nada a apagar.
        return;
    }
    // Buffer alinhado a 8 bytes para conter CRYPT_KEY_PROV_INFO + as strings.
    let mut aligned: Vec<u64> = vec![0u64; ((cb as usize) + 7) / 8];
    let buf = aligned.as_mut_ptr() as *mut std::ffi::c_void;
    if CertGetCertificateContextProperty(cert_ctx, CERT_KEY_PROV_INFO_PROP_ID, buf, &mut cb) == 0 {
        return;
    }
    let info = &*(buf as *const CRYPT_KEY_PROV_INFO);

    if info.dwProvType == 0 {
        // CNG / KSP
        let mut h_prov: usize = 0;
        if NCryptOpenStorageProvider(&mut h_prov, info.pwszProvName, 0) == 0 {
            let mut h_key: usize = 0;
            if NCryptOpenKey(h_prov, &mut h_key, info.pwszContainerName, 0, NCRYPT_SILENT_FLAG) == 0 {
                NCryptDeleteKey(h_key, NCRYPT_SILENT_FLAG); // também libera h_key
            }
            NCryptFreeObject(h_prov);
        }
    } else {
        // CSP legado: CryptAcquireContext com CRYPT_DELETEKEYSET remove o container.
        let mut h_prov: usize = 0;
        CryptAcquireContextW(
            &mut h_prov,
            info.pwszContainerName,
            info.pwszProvName,
            info.dwProvType,
            CRYPT_DELETEKEYSET,
        );
    }
}

/// Abre o PFX via Windows CAPI e retorna os bytes DER do certificado folha.
/// Suporta nativamente RC2-40-CBC e 3DES presentes em certificados ICP-Brasil.
///
/// IMPORTANTE: este DER é embutido como `<X509Certificate>` na assinatura. Precisa ser
/// o MESMO certificado folha usado para assinar (o da empresa), e não a CA da cadeia —
/// senão a SEFAZ rejeita com cStat 290 "Certificado Assinatura inválido", pois a
/// assinatura não verifica contra o certificado embutido.
#[cfg(target_os = "windows")]
fn pfx_capi_extract_cert_der(pfx_bytes: &[u8], password: &str) -> std::result::Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CertCloseStore, CertFreeCertificateContext, PFXImportCertStore, CRYPT_INTEGER_BLOB,
    };

    const PKCS12_NO_PERSIST_KEY: u32 = 0x00008000;

    let pw_wide: Vec<u16> = password.encode_utf16().chain(std::iter::once(0)).collect();
    let pfx_blob = CRYPT_INTEGER_BLOB {
        cbData: pfx_bytes.len() as u32,
        pbData: pfx_bytes.as_ptr() as *mut u8,
    };

    unsafe {
        // NO_PERSIST: só precisamos dos bytes do certificado, não da chave em si.
        let store = PFXImportCertStore(&pfx_blob, pw_wide.as_ptr(), PKCS12_NO_PERSIST_KEY);
        if store.is_null() {
            return Err("Senha inválida ou PFX corrompido".to_string());
        }

        let cert_ctx = find_leaf_cert(store);
        let result = if cert_ctx.is_null() {
            Err("Nenhum certificado com chave privada encontrado no PFX".to_string())
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

/// Assina `data` com RSA-SHA1 usando a chave privada do PFX via Windows.
/// Suporta nativamente RC2/3DES dos certificados ICP-Brasil.
///
/// Importa o PFX **persistindo** a chave (CRYPT_USER_KEYSET): essa é a única forma de
/// `CryptAcquireCertificatePrivateKey` localizar a chave sem o certificado já estar
/// instalado no Windows. Com PKCS12_NO_PERSIST_KEY a chave fica só em memória e, em
/// várias máquinas, o acquire falha ("Chave privada não acessível") a menos que o
/// usuário tenha instalado o certificado manualmente. Após assinar, apagamos o keyset
/// (`delete_key_container`) para não acumular containers órfãos em disco.
///
/// Chaves A1 ICP-Brasil podem ser CNG (KSP/NCrypt) ou CAPI legado (CSP). Por isso o
/// acquire usa `CRYPT_ACQUIRE_PREFER_NCRYPT_KEY_FLAG` e ramificamos:
/// - chave CNG  → `NCryptSignHash` (retorna assinatura big-endian PKCS#1, sem reverter)
/// - chave CAPI → `CryptSignHashW` (retorna little-endian, revertemos para big-endian)
#[cfg(target_os = "windows")]
fn pfx_capi_sign_rsa_sha1(data: &[u8], pfx_bytes: &[u8], password: &str) -> std::result::Result<Vec<u8>, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CertCloseStore, CertFreeCertificateContext, CryptAcquireCertificatePrivateKey,
        CryptCreateHash, CryptDestroyHash, CryptHashData, CryptReleaseContext, CryptSignHashW,
        NCryptFreeObject, NCryptSignHash, PFXImportCertStore, BCRYPT_PKCS1_PADDING_INFO,
        CRYPT_INTEGER_BLOB,
    };

    const CRYPT_EXPORTABLE: u32 = 0x00000001;
    const CRYPT_USER_KEYSET: u32 = 0x00001000;
    const PKCS12_ALLOW_OVERWRITE_KEY: u32 = 0x00004000;
    const CALG_SHA1: u32 = 0x00008004;
    // Flags de CryptAcquireCertificatePrivateKey
    const CRYPT_ACQUIRE_SILENT_FLAG: u32 = 0x00000040;
    const CRYPT_ACQUIRE_PREFER_NCRYPT_KEY_FLAG: u32 = 0x00020000;
    // Marca que a chave retornada é um handle CNG (NCRYPT_KEY_HANDLE)
    const CERT_NCRYPT_KEY_SPEC: u32 = 0xFFFFFFFF;
    const BCRYPT_PAD_PKCS1: u32 = 0x00000002;

    let pw_wide: Vec<u16> = password.encode_utf16().chain(std::iter::once(0)).collect();
    let pfx_blob = CRYPT_INTEGER_BLOB {
        cbData: pfx_bytes.len() as u32,
        pbData: pfx_bytes.as_ptr() as *mut u8,
    };

    unsafe {
        // Persiste a chave no keyset do usuário para que o acquire a encontre sem o
        // certificado precisar estar instalado no Windows. Apagada ao final.
        let store = PFXImportCertStore(
            &pfx_blob,
            pw_wide.as_ptr(),
            CRYPT_EXPORTABLE | CRYPT_USER_KEYSET | PKCS12_ALLOW_OVERWRITE_KEY,
        );
        if store.is_null() {
            return Err("Senha inválida ou PFX corrompido".to_string());
        }

        // Seleciona o certificado folha (da empresa), não a CA da cadeia.
        let cert_ctx = find_leaf_cert(store);
        if cert_ctx.is_null() {
            CertCloseStore(store, 0);
            return Err("Nenhum certificado com chave privada encontrado no PFX".to_string());
        }

        let mut h_key: usize = 0;
        let mut dw_key_spec: u32 = 0;
        let mut b_free_key: i32 = 0;
        let ok = CryptAcquireCertificatePrivateKey(
            cert_ctx,
            CRYPT_ACQUIRE_SILENT_FLAG | CRYPT_ACQUIRE_PREFER_NCRYPT_KEY_FLAG,
            std::ptr::null(),
            &mut h_key,
            &mut dw_key_spec,
            &mut b_free_key,
        );
        if ok == 0 {
            delete_key_container(cert_ctx);
            CertFreeCertificateContext(cert_ctx);
            CertCloseStore(store, 0);
            return Err("Chave privada não acessível".to_string());
        }

        let result = if dw_key_spec == CERT_NCRYPT_KEY_SPEC {
            // ── Caminho CNG (NCrypt) ──────────────────────────────────────────
            // NCryptSignHash assina o digest SHA1 já calculado, com padding PKCS#1.
            let mut hasher = Sha1::new();
            hasher.update(data);
            let hash = hasher.finalize();

            // BCRYPT_SHA1_ALGORITHM = "SHA1"
            let sha1_alg: Vec<u16> = "SHA1".encode_utf16().chain(std::iter::once(0)).collect();
            let padding = BCRYPT_PKCS1_PADDING_INFO {
                pszAlgId: sha1_alg.as_ptr(),
            };
            let padding_ptr = &padding as *const _ as *const std::ffi::c_void;

            // Primeira chamada: obtém o tamanho da assinatura
            let mut sig_len: u32 = 0;
            let st = NCryptSignHash(
                h_key, padding_ptr, hash.as_ptr(), hash.len() as u32,
                std::ptr::null_mut(), 0, &mut sig_len, BCRYPT_PAD_PKCS1,
            );
            if st != 0 {
                Err("Erro ao calcular tamanho da assinatura (NCrypt)".to_string())
            } else {
                let mut sig = vec![0u8; sig_len as usize];
                let st = NCryptSignHash(
                    h_key, padding_ptr, hash.as_ptr(), hash.len() as u32,
                    sig.as_mut_ptr(), sig_len, &mut sig_len, BCRYPT_PAD_PKCS1,
                );
                if st != 0 {
                    Err("Erro ao assinar dados com a chave privada (NCrypt)".to_string())
                } else {
                    sig.truncate(sig_len as usize);
                    // NCrypt já retorna PKCS#1 big-endian padrão — não reverter
                    Ok(sig)
                }
            }
        } else {
            // ── Caminho CAPI legado (CSP) ─────────────────────────────────────
            let mut h_hash: usize = 0;
            if CryptCreateHash(h_key, CALG_SHA1, 0, 0, &mut h_hash) == 0 {
                Err("Erro ao criar contexto de hash SHA1".to_string())
            } else {
                CryptHashData(h_hash, data.as_ptr(), data.len() as u32, 0);

                // Primeira chamada: obtém o tamanho da assinatura
                let mut sig_len: u32 = 0;
                CryptSignHashW(h_hash, dw_key_spec, std::ptr::null(), 0, std::ptr::null_mut(), &mut sig_len);

                // Segunda chamada: assina de fato
                let mut sig = vec![0u8; sig_len as usize];
                let ok = CryptSignHashW(h_hash, dw_key_spec, std::ptr::null(), 0, sig.as_mut_ptr(), &mut sig_len);
                sig.truncate(sig_len as usize);
                CryptDestroyHash(h_hash);

                if ok == 0 {
                    Err("Erro ao assinar dados com a chave privada".to_string())
                } else {
                    // CAPI retorna RSA em little-endian; PKCS#1 padrão é big-endian
                    sig.reverse();
                    Ok(sig)
                }
            }
        };

        // Liberação dos handles
        if b_free_key != 0 {
            if dw_key_spec == CERT_NCRYPT_KEY_SPEC {
                NCryptFreeObject(h_key);
            } else {
                CryptReleaseContext(h_key, 0);
            }
        }
        // Apaga o keyset persistido na importação (evita acúmulo de containers em disco).
        delete_key_container(cert_ctx);
        CertFreeCertificateContext(cert_ctx);
        CertCloseStore(store, 0);

        result
    }
}

/// Lê subject/issuer/validade do certificado folha de um PFX via Windows CAPI.
///
/// Importa **persistindo** a chave (`CRYPT_USER_KEYSET`) e apaga o keyset ao final
/// (`delete_key_container`). Diferente de `pfx_capi_extract_cert_der`, que usa
/// `PKCS12_NO_PERSIST_KEY`: o import efêmero retorna NULL para PFX ICP-Brasil com
/// cifragem legada (RC2/3DES) em máquinas onde o certificado não está instalado —
/// o que fazia esta leitura falhar com "PFX corrompido" num PDV recém-instalado.
#[cfg(target_os = "windows")]
fn pfx_capi_read_info(pfx_bytes: &[u8], password: &str) -> std::result::Result<CertInfo, String> {
    use windows_sys::Win32::Security::Cryptography::{
        CertCloseStore, CertFreeCertificateContext, CertGetNameStringW, PFXImportCertStore,
        CERT_NAME_ISSUER_FLAG, CERT_X500_NAME_STR, CRYPT_INTEGER_BLOB,
    };
    use windows_sys::Win32::Foundation::SYSTEMTIME;
    use windows_sys::Win32::System::Time::FileTimeToSystemTime;

    const CRYPT_USER_KEYSET: u32 = 0x00001000;
    const PKCS12_ALLOW_OVERWRITE_KEY: u32 = 0x00004000;

    let pw_wide: Vec<u16> = password.encode_utf16().chain(std::iter::once(0)).collect();
    let pfx_blob = CRYPT_INTEGER_BLOB {
        cbData: pfx_bytes.len() as u32,
        pbData: pfx_bytes.as_ptr() as *mut u8,
    };

    unsafe {
        // Persiste a chave (apagada ao final) — ver doc da função para o porquê.
        let store = PFXImportCertStore(
            &pfx_blob,
            pw_wide.as_ptr(),
            CRYPT_USER_KEYSET | PKCS12_ALLOW_OVERWRITE_KEY,
        );
        if store.is_null() {
            return Err("Senha inválida ou PFX corrompido".to_string());
        }

        let cert_ctx = find_leaf_cert(store);
        if cert_ctx.is_null() {
            CertCloseStore(store, 0);
            return Err("Nenhum certificado com chave privada encontrado no PFX".to_string());
        }

        // Subject (titular)
        let mut subj_buf = vec![0u16; 512];
        CertGetNameStringW(
            cert_ctx, CERT_X500_NAME_STR, 0, std::ptr::null_mut(),
            subj_buf.as_mut_ptr(), subj_buf.len() as u32,
        );
        let subject = String::from_utf16_lossy(&subj_buf).trim_matches('\0').to_string();

        // Issuer (autoridade certificadora)
        let mut iss_buf = vec![0u16; 512];
        CertGetNameStringW(
            cert_ctx, CERT_X500_NAME_STR, CERT_NAME_ISSUER_FLAG, std::ptr::null_mut(),
            iss_buf.as_mut_ptr(), iss_buf.len() as u32,
        );
        let issuer = String::from_utf16_lossy(&iss_buf).trim_matches('\0').to_string();

        // Validade: CERT_INFO → FILETIME → SYSTEMTIME
        let cert_info = &*(*cert_ctx).pCertInfo;
        let mut st_from: SYSTEMTIME = std::mem::zeroed();
        FileTimeToSystemTime(&cert_info.NotBefore, &mut st_from);
        let mut st_to: SYSTEMTIME = std::mem::zeroed();
        FileTimeToSystemTime(&cert_info.NotAfter, &mut st_to);
        let valid_from = format!("{:04}/{:02}/{:02}", st_from.wYear, st_from.wMonth, st_from.wDay);
        let valid_to = format!("{:04}/{:02}/{:02}", st_to.wYear, st_to.wMonth, st_to.wDay);

        // Apaga o keyset persistido na importação e libera os handles.
        delete_key_container(cert_ctx);
        CertFreeCertificateContext(cert_ctx);
        CertCloseStore(store, 0);

        Ok(CertInfo { subject, issuer, valid_from, valid_to })
    }
}

#[cfg(not(target_os = "windows"))]
fn pfx_capi_read_info(pfx_bytes: &[u8], password: &str) -> std::result::Result<CertInfo, String> {
    super::cert_openssl::ler_info(pfx_bytes, password)
}

// Fora do Windows os três pontos de entrada caem no OpenSSL (`cert_openssl.rs`), com o mesmo
// resultado do CAPI — conferido pelos testes abaixo com os PFX sintéticos.
#[cfg(not(target_os = "windows"))]
fn pfx_capi_extract_cert_der(pfx_bytes: &[u8], password: &str) -> std::result::Result<Vec<u8>, String> {
    super::cert_openssl::extrair_cert_der(pfx_bytes, password)
}

#[cfg(not(target_os = "windows"))]
fn pfx_capi_sign_rsa_sha1(data: &[u8], pfx_bytes: &[u8], password: &str) -> std::result::Result<Vec<u8>, String> {
    super::cert_openssl::assinar_rsa_sha1(data, pfx_bytes, password)
}

/// Mesmos PFX sintéticos (`tests/fixtures/cert/gerar.sh`) nos dois sistemas: o Windows (CAPI) é a
/// referência e o Linux (OpenSSL) tem de devolver exatamente o mesmo. Os valores esperados foram
/// medidos no Windows em 15-11-2026.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DfeError;

    const SENHA: &str = "senha-teste-dfe";
    const PFX: [&str; 2] = ["sintetico_aes.pfx", "sintetico_legado.pfx"];

    fn fixture(nome: &str) -> String {
        format!("{}/tests/fixtures/cert/{nome}", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn cert_info_igual_ao_capi_no_pfx_moderno_e_no_legado() {
        for nome in PFX {
            let i = CertInfo::from_pfx(&fixture(nome), SENHA).unwrap_or_else(|e| panic!("{nome}: {e:?}"));
            assert_eq!(i.subject, "EMPRESA TESTE DFE LTDA:12345678000195", "{nome}");
            assert_eq!(i.issuer, "AC SINTETICA DFE", "{nome}");
            assert_eq!((i.valid_from.as_str(), i.valid_to.as_str()), ("2026/11/15", "2126/10/22"), "{nome}");
        }
    }

    #[test]
    fn cert_info_senha_errada_diz_senha() {
        for nome in PFX {
            match CertInfo::from_pfx(&fixture(nome), "senha-errada") {
                Err(DfeError::Certificado(m)) => assert!(m.starts_with("Senha inválida ou PFX corrompido"), "{nome}: {m}"),
                outro => panic!("{nome}: esperado erro de senha, veio {outro:?}"),
            }
        }
    }

    #[test]
    fn cert_info_arquivo_inexistente_e_erro_de_arquivo_nao_de_senha() {
        let r = CertInfo::from_pfx(&fixture("nao_existe.pfx"), SENHA);
        assert!(matches!(r, Err(DfeError::Io(_))), "{r:?}");
    }

    #[test]
    fn cert_info_pfx_corrompido_e_recusado() {
        let bytes = std::fs::read(fixture("sintetico_aes.pfx")).unwrap();
        let caminho = std::env::temp_dir().join(format!("dfe_pfx_corrompido_{}.pfx", std::process::id()));
        std::fs::write(&caminho, &bytes[..bytes.len() / 2]).unwrap();
        let r = CertInfo::from_pfx(caminho.to_str().unwrap(), SENHA);
        let _ = std::fs::remove_file(&caminho);
        assert!(matches!(r, Err(DfeError::Certificado(_))), "{r:?}");
    }

    /// Começo do DER (base64) do certificado da empresa — o que vai no `<X509Certificate>`.
    const DER_INICIO: &str = "MIIDuDCCAqCgAwIBAgIUVFhcNsPRZomovpobVpVt";
    /// SHA-1 desse base64 inteiro, calculado à parte pela linha de comando
    /// (`openssl pkcs12 -clcerts -nokeys | openssl x509 -outform DER | base64 -w0 | sha1sum`).
    const DER_SHA1: &str = "344768496703f86f51deac0bc8509e04749c8560";
    /// RSA-SHA1 PKCS#1 v1.5 é determinístico: mesmo PFX + mesmo texto = mesma assinatura.
    const ASSINATURA: &str = "N3hu7sky2GXYaeMbbTZpa+VCpwptrF7Xr6t/+Zg5J9FHFkCztVNknjcxIEu35KzAasPCwFmemUYCpZvHTzerPFJUdbSWraB+gyQqA3l6vPRcmophGu91375XPiegiFGYXJi2VRDfDrtJ85rF/OOIjp8IJnPP1Qf8Y0KSAlObxTaZklYX3VaeAJWi1OcPaGv4vFJRivTTA7q0GOT4JWd7M4eJDDYvfeF+M/MdtTOOR4RBiXOJhzZK9YGHgq63aaQ4Kqe4H9KwStwAHiuGIQ+fBWHRZv2jcvvQRocCLH4/dvnNEVphs32DL3V1OLW06Pvlz2yXouBOKJQc6+XMbvWktg==";

    #[tokio::test]
    async fn certificado_extraido_e_o_da_empresa_inteiro() {
        for nome in PFX {
            let der = RawPubKey::get_from_file(&fixture(nome), SENHA).await.unwrap_or_else(|e| panic!("{nome}: {e:?}"));
            assert_eq!(der.len(), 1276, "{nome}");
            assert!(der.starts_with(DER_INICIO), "{nome}");
            assert_eq!(hex::encode(Sha1::digest(der.as_bytes())), DER_SHA1, "{nome}: não é o certificado da empresa");
        }
    }

    #[tokio::test]
    async fn extracao_e_assinatura_com_senha_errada_falham() {
        let p = fixture("sintetico_aes.pfx");
        assert!(matches!(RawPubKey::get_from_file(&p, "x").await, Err(DfeError::Certificado(_))));
        assert!(matches!(Sign::xml_string("a", &p, "x").await, Err(DfeError::Assinatura(_))));
    }

    #[tokio::test]
    async fn assinatura_igual_a_do_capi() {
        for nome in PFX {
            let sig = Sign::xml_string("<SignedInfo>teste dfe</SignedInfo>", &fixture(nome), SENHA)
                .await
                .unwrap_or_else(|e| panic!("{nome}: {e:?}"));
            assert_eq!(sig, ASSINATURA, "{nome}");
        }
    }
}

#[cfg(test)]
mod tests_mtls {
    use super::*;

    /// A identidade do mTLS (`reqwest::Identity`) abre os dois PFX. No Linux ela passa pelo
    /// OpenSSL do `native-tls`, que só lê o legado (RC2/3DES) com o provider `legacy` já
    /// carregado — este teste fica num módulo próprio e roda isolado na verificação.
    #[test]
    fn identidade_mtls_abre_pfx_moderno_e_legado() {
        for nome in ["sintetico_aes.pfx", "sintetico_legado.pfx"] {
            let p = format!("{}/tests/fixtures/cert/{nome}", env!("CARGO_MANIFEST_DIR"));
            if let Err(e) = Cert::from_pfx(&p, "senha-teste-dfe") {
                panic!("{nome}: {e:?}");
            }
        }
    }
}

/// Camada `sefaz-live` com o **A1 real** do `.env` (sem enviar nada à SEFAZ): imprime o que
/// precisa sair igual no Windows (CAPI) e no Linux (OpenSSL) — dados do certificado, SHA-1 do
/// DER e SHA-256 de uma assinatura sobre texto fixo. Rodar nos dois e comparar a linha `REF`.
/// Não imprime chave nem senha; a assinatura vai só como hash.
#[cfg(all(test, feature = "sefaz-live"))]
mod tests_live_cert_real {
    use super::*;
    use sha2::Sha256;

    fn env_cert() -> Option<(String, String)> {
        if let Err(e) = dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/.env")) {
            assert!(e.not_found(), ".env inválido: {e} — use `/` no caminho ou aspas simples");
        }
        let caminho = std::env::var("DFE_TEST_CERT_PATH").ok().filter(|v| !v.is_empty())?;
        let senha = std::env::var("DFE_TEST_CERT_PASS").unwrap_or_default();
        let b = caminho.as_bytes();
        let eh_windows = b.len() > 2 && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
        let caminho = if cfg!(target_os = "windows") || !eh_windows {
            caminho.clone()
        } else {
            format!("/mnt/{}/{}", (b[0] as char).to_ascii_lowercase(), caminho[3..].replace('\\', "/"))
        };
        Some((caminho, senha))
    }

    #[tokio::test]
    async fn live_certificado_real_igual_nos_dois_sistemas() {
        let Some((caminho, senha)) = env_cert() else {
            eprintln!("skip: .env sem DFE_TEST_CERT_PATH");
            return;
        };
        let info = CertInfo::from_pfx(&caminho, &senha).unwrap_or_else(|e| panic!("CertInfo: {e:?}"));
        let der = RawPubKey::get_from_file(&caminho, &senha).await.unwrap_or_else(|e| panic!("DER: {e:?}"));
        let sig = Sign::xml_string("<SignedInfo>dfe equivalencia</SignedInfo>", &caminho, &senha)
            .await
            .unwrap_or_else(|e| panic!("assinatura: {e:?}"));
        Cert::from_pfx(&caminho, &senha).unwrap_or_else(|e| panic!("identidade mTLS: {e:?}"));
        println!(
            "REF subject={} | issuer={} | {}..{} | der_sha1={} | sig_sha256={}",
            info.subject,
            info.issuer,
            info.valid_from,
            info.valid_to,
            hex::encode(Sha1::digest(der.as_bytes())),
            hex::encode(Sha256::digest(sig.as_bytes())),
        );
    }
}
