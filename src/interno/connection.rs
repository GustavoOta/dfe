use crate::error::Result;
use reqwest::Client;
use reqwest::Identity;

#[derive(Debug)]
pub struct WebService {}

impl WebService {
    /// Constrói um cliente reqwest com o certificado A1 na camada TLS (mTLS exigido pela SEFAZ).
    /// Usado pelo [`crate::interno::transporte::MtlsTransport`].
    pub fn client(identity: Identity) -> Result<Client> {
        let client = Client::builder().identity(identity).build()?;
        Ok(client)
    }
}
