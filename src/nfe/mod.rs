pub mod autorizacao;
pub mod cancelar;
pub mod common;
pub mod connection;
pub mod mod_status_service;
pub mod types;
pub mod xml_extractor;

use anyhow::{Error, Result};

use common::config_file::decrypt_password_from_file;
use common::config_file::encrypt_password_to_file;
use mod_status_service::*;
use types::config::*;
use types::service_status::Status;

/// Returns the status of the NFe service
///
/// # Arguments
/// `use` - Specifies how to use the service, either `Use::FileConfig` or `Use::ManualConfig`.
/// Where `Use::FileConfig` reads the configurations from a file at the root of the project named `nfe_config.json`
/// and `Use::ManualConfig`
/// receives the configurations as parameters:
///
/// Fields {
///     cert_path: String,
///     cert_pass: Password,
///     federative_unit: String,
///     environment: Environment,
///     svc: bool,
/// }
///
/// Where `Password` is an enum that can be:
///
/// `Password::File(PassFile)` or `Password::Phrase(String)`
///
/// And `PassFile` is a struct with the fields:
///
/// PassFile {
///     encrypted_password: String,
///     key: String,
///     iv: String,
/// }
///  To generate the encrypted password file use the dfe::nfe::crypt() function. (not secure enough)
///
/// # Example
///
/// ```rust
/// use dfe::nfe::service_status;
/// use dfe::nfe::types::service_status::Use;
/// use dfe::nfe::types::service_status::Status;
///
/// let status = service_status(Use::FileConfig);
/// ```
///
/// # Returns
/// The status of the NFe service.
pub async fn service_status(params: Use) -> Result<Status> {
    let result = get(params).await?;

    Ok(result)
}

/// Encrypt the password, key, and iv to a file using the certificate.pfx file and password
/// specified in the function.
/// The encrypted password, key, and iv are saved to a file named `cert_pass.txt` at the root of the project.
/// The password is encrypted using the `aes_256_cbc` cipher.
/// The key and iv are encoded using the `base64` encoding.
/// The encrypted password is encoded using the `base64` encoding.
/// The password, key, and iv are saved in the following format:
/// ```
/// encrypted_password
/// key
/// iv
/// ```
/// # Example
/// ```rust
/// use dfe::nfe::crypt;
/// crypt("password");
/// ```
/// # Returns
///
/// The encrypted password, key, and iv are saved to a file named `cert_pass.txt` at the root of the project.

pub fn crypt(password: &str) -> Result<(), Error> {
    let result = encrypt_password_to_file(password)?;
    Ok(result)
}
/// Decrypt the password from the file `cert_pass.txt` at the
/// root of the project.
/// The password is decrypted using the `aes_256_cbc` cipher.
/// The key and iv are decoded using the `base64` encoding.
/// The encrypted password is decoded using the `base64` encoding.
/// The password, key, and iv are read from the file in the following format:
/// ```
/// encrypted_password
/// key
/// iv
/// ```
/// # Example
/// ```rust
/// use dfe::nfe::decrypt;
/// let password = decrypt()?;
/// ```
/// # Returns
/// The decrypted password as a `String`.
/// # Errors
/// Returns an error if the file `cert_pass.txt` does not exist or if there is an error reading the file.

pub fn decrypt() -> Result<String, Error> {
    let result = decrypt_password_from_file()?;
    Ok(result)
}
