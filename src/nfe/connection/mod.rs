use anyhow::Result;
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
        let response = client
            .post(url)
            .header("Content-Type", "application/soap+xml; charset=utf-8")
            .body(body)
            .send()
            .await;

        match response {
            Ok(response) => Result::Ok(response),
            Err(e) => Result::Err(anyhow::anyhow!("Failed to send request: {}", e)),
        }
    }
}
