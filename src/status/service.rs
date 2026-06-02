use crate::interno::cert;
use crate::interno::connection;

pub async fn send_status_request(
    cert_path: &str,
    cert_pass: &str,
    url: &str,
    xml: &str,
) -> Result<String, String> {
    let cert = cert::Cert::from_pfx(cert_path, cert_pass).map_err(|e| e.to_string())?;
    let client = connection::WebService::client(cert.identity).map_err(|e| e.to_string())?;

    let response = connection::WebService::send(client, url, xml.to_string())
        .await
        .map_err(|e| e.to_string())?;

    response.text().await.map_err(|e| e.to_string())
}
