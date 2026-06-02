mod endpoint;
mod parser;
mod service;
mod validation;
mod xml;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NFeServiceResponse {
    pub c_stat: String,
    pub x_motivo: String,
    pub sent_xml: String,
    pub received_xml: String,
    pub url: String,
}

#[derive(Debug)]
pub struct NFeService {
    pub cert_path: String,
    pub cert_pass: String,
    pub uf: String,
    pub environment: u8,
}

impl NFeService {
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            uf: String::new(),
            environment: 0,
        }
    }

    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    pub fn uf(mut self, uf: &str) -> Self {
        self.uf = uf.to_string();
        self
    }

    pub fn environment(mut self, environment: u8) -> Self {
        self.environment = environment;
        self
    }

    pub fn build(self) -> Result<Self, String> {
        validation::validate_nfe_service(&self)?;

        Ok(self)
    }

    pub async fn send(self) -> Result<NFeServiceResponse, String> {
        validation::validate_nfe_service(&self)?;

        let url = endpoint::status_url(self.environment, &self.uf)?;
        let xml = xml::status_request_xml(self.environment, &self.uf)?;
        let body =
            service::send_status_request(&self.cert_path, &self.cert_pass, &url, &xml).await?;
        let status = parser::parse_status_response(&body)?;

        Ok(NFeServiceResponse {
            c_stat: status.c_stat,
            x_motivo: status.x_motivo,
            sent_xml: xml,
            received_xml: body,
            url,
        })
    }
}
