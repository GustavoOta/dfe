use crate::error::Result;
use reqwest::Client;
use reqwest::Identity;
use std::time::Duration;

#[derive(Debug)]
pub struct WebService {}

impl WebService {
    /// Constrói um cliente reqwest com o certificado A1 na camada TLS (mTLS exigido pela SEFAZ).
    /// Timeout de 30 s: cobre respostas lentas sem bloquear indefinidamente.
    /// Usado pelo [`crate::interno::transporte::MtlsTransport`].
    pub fn client(identity: Identity) -> Result<Client> {
        let client = Client::builder()
            .identity(identity)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(client)
    }
}
