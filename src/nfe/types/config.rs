use serde::{Deserialize, Serialize};

pub enum Use {
    ManualConfig(Fields),
    FileConfig,
}

pub struct Fields {
    pub cert_path: String,
    pub cert_pass: Password,
    pub federative_unit: String,
    pub environment: Environment,
}

// Environment options to use in the service
#[derive(Serialize, Deserialize, Debug)]
pub enum Environment {
    Production = 1,
    Homologation = 2,
}

// Password options to use the encrypted password, key, and iv in a file
// or the password in a string
#[derive(Serialize, Deserialize, Debug)]
pub enum Password {
    File(PassFile),
    Phrase(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PassFile {
    pub path: String,
}
