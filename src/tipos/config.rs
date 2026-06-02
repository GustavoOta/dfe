use serde::{Deserialize, Serialize};
use std::fmt;

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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Environment {
    Production,
    Homologation,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Environment::Production => write!(f, "1"),
            Environment::Homologation => write!(f, "2"),
        }
    }
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
