use crate::interno::transporte::{MtlsTransport, SoapTransport};

pub async fn send_status_request(
    cert_path: &str,
    cert_pass: &str,
    url: &str,
    xml: &str,
) -> Result<String, String> {
    MtlsTransport::new(cert_path, cert_pass)
        .send_soap(url, xml.to_string())
        .await
        .map_err(|e| e.to_string())
}
