use crate::error::{DfeError, Result};
use reqwest::Client;
use reqwest::Identity;

#[derive(Debug)]
pub struct WebService {}

impl WebService {
    pub fn client(identity: Identity) -> Result<Client> {
        let client = Client::builder().identity(identity).build()?;
        Ok(client)
    }

    pub async fn send(client: Client, url: &str, body: String) -> Result<reqwest::Response> {
        client
            .post(url)
            .header("Content-Type", "application/soap+xml; charset=utf-8")
            .body(body)
            .send()
            .await
            .map_err(|e| DfeError::Webservice(format!("Failed to send request: {}", e)))
    }
}
