use std::fmt;

#[derive(Debug, Clone)]
pub enum DfeError {
    Certificado(String),
    Xml(String),
    Assinatura(String),
    Webservice(String),
    Validacao(String),
    Configuracao(String),
    Io(String),
}

impl fmt::Display for DfeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DfeError::Certificado(msg) => write!(f, "Erro de certificado: {}", msg),
            DfeError::Xml(msg) => write!(f, "Erro de XML: {}", msg),
            DfeError::Assinatura(msg) => write!(f, "Erro de assinatura: {}", msg),
            DfeError::Webservice(msg) => write!(f, "Erro de webservice: {}", msg),
            DfeError::Validacao(msg) => write!(f, "Erro de validação: {}", msg),
            DfeError::Configuracao(msg) => write!(f, "Erro de configuração: {}", msg),
            DfeError::Io(msg) => write!(f, "Erro de I/O: {}", msg),
        }
    }
}

impl std::error::Error for DfeError {}

impl From<std::io::Error> for DfeError {
    fn from(e: std::io::Error) -> Self {
        DfeError::Io(e.to_string())
    }
}

impl From<openssl::error::ErrorStack> for DfeError {
    fn from(e: openssl::error::ErrorStack) -> Self {
        DfeError::Assinatura(e.to_string())
    }
}

impl From<reqwest::Error> for DfeError {
    fn from(e: reqwest::Error) -> Self {
        DfeError::Webservice(e.to_string())
    }
}

impl From<quick_xml::Error> for DfeError {
    fn from(e: quick_xml::Error) -> Self {
        DfeError::Xml(e.to_string())
    }
}

impl From<quick_xml::de::DeError> for DfeError {
    fn from(e: quick_xml::de::DeError) -> Self {
        DfeError::Xml(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for DfeError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        DfeError::Xml(e.to_string())
    }
}

impl From<base64::DecodeError> for DfeError {
    fn from(e: base64::DecodeError) -> Self {
        DfeError::Configuracao(e.to_string())
    }
}

impl From<serde_json::Error> for DfeError {
    fn from(e: serde_json::Error) -> Self {
        DfeError::Configuracao(e.to_string())
    }
}

impl From<tokio::task::JoinError> for DfeError {
    fn from(e: tokio::task::JoinError) -> Self {
        DfeError::Validacao(e.to_string())
    }
}

impl From<regex::Error> for DfeError {
    fn from(e: regex::Error) -> Self {
        DfeError::Xml(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DfeError>;
