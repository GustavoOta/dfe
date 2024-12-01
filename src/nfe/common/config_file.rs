/// Configuration file for the NFE
///
/// This file is used to store the configuration of the NFE, such as the password for the PFX file.
///
use anyhow::*;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use openssl::hash::MessageDigest;
//use openssl::pkcs12::Pkcs12;
use openssl::pkcs5::pbkdf2_hmac;
use openssl::symm::{decrypt, encrypt, Cipher};
use rand::Rng;
use std::fs::File;
use std::io::{BufRead, Write};

/// encrypt_password_to_file encrypts the password using the `aes_256_cbc` cipher and saves it to a file.
pub fn encrypt_password_to_file(password: &str) -> Result<(), anyhow::Error> {
    let mut key = [0u8; 32];
    let salt = b"some_salt"; // Ideally, this should be random and stored securely
    pbkdf2_hmac(
        password.as_bytes(),
        salt,
        10000,
        MessageDigest::sha256(),
        &mut key,
    )?;

    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut iv);

    let cipher = Cipher::aes_256_cbc();
    let encrypted = encrypt(cipher, &key, Some(&iv), password.as_bytes())?;

    let encrypted_password = STANDARD.encode(&encrypted);
    let key = STANDARD.encode(&key);
    let iv = STANDARD.encode(&iv);

    save_to_file(&encrypted_password, &key, &iv, "cert_pass.txt");

    Ok(())

    /* let mut file = File::open(file_path)?;
    let mut pfx_data = Vec::new();
    file.read_to_end(&mut pfx_data)?;
    let pfx_password = password;

    let pkcs12 = Pkcs12::from_der(&pfx_data).expect("Failed to parse PFX data");
    let parsed = pkcs12
        .parse2(password)
        .expect("Failed to parse PFX password");

    if parsed.cert.is_none() {
        return Err(anyhow::anyhow!("Failed to parse PFX certificate"));
    }

    let mut key = [0u8; 32];
    let salt = b"some_salt"; // Ideally, this should be random and stored securely
    pbkdf2_hmac(
        password.as_bytes(),
        salt,
        10000,
        MessageDigest::sha256(),
        &mut key,
    )
    .unwrap();

    let mut iv = [0u8; 16];
    rand::thread_rng().fill(&mut iv);

    let cipher = Cipher::aes_256_cbc();
    let encrypted = encrypt(cipher, &key, Some(&iv), password.as_bytes()).unwrap();

    let encrypted_password = STANDARD.encode(&encrypted);
    let key = STANDARD.encode(&key);
    let iv = STANDARD.encode(&iv);

    Ok(ConfigFile {
        encrypted_password,
        key,
        iv,
    }) */
}

pub fn decrypt_password_from_file() -> Result<String, anyhow::Error> {
    let file = File::open("cert_pass.txt")?;
    let mut lines = std::io::BufReader::new(file).lines();
    let encrypted_password = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing encrypted password line"))??;
    let key = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing key line"))??;
    let iv = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing iv line"))??;
    let cipher = Cipher::aes_256_cbc();

    let encrypted_password = STANDARD.decode(encrypted_password)?;
    let key = STANDARD.decode(key)?;
    let iv = STANDARD.decode(iv)?;

    let decrypted = decrypt(cipher, &key, Some(&iv), &encrypted_password)?;

    let decrypted_password = String::from_utf8(decrypted)?;

    Ok(decrypted_password)
}

fn save_to_file(encrypted_password: &str, key: &str, iv: &str, file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    writeln!(file, "{}", encrypted_password).unwrap();
    writeln!(file, "{}", key).unwrap();
    writeln!(file, "{}", iv).unwrap();
}
