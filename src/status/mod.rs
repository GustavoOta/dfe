//! Status do webservice SEFAZ (`consStatServ`, `NFeStatusServico4`).
//!
//! [`NFeService`] consulta se a SEFAZ da UF está no ar. NF-e (55) e NFC-e (65) têm webservices
//! próprios — em várias UFs, hosts diferentes — e um pode cair sem o outro: use
//! [`NFeService::modelo`] para escolher (padrão 55). Só `c_stat == "107"` é "em operação".
//!
//! A SEFAZ controla consumo indevido também aqui: no mínimo 3 min entre consultas, senão
//! `cStat 656` e bloqueio do CNPJ por 1 h. A crate não limita — o controle é do consumidor.
//! Guia: `docs/status-webservice.md`.

mod endpoint;
mod parser;
mod service;
mod validation;
mod xml;

use serde::Serialize;

/// Resposta da consulta de status do webservice SEFAZ.
#[derive(Debug, Serialize)]
pub struct NFeServiceResponse {
    /// Código de status (`"107"` = serviço em operação).
    pub c_stat: String,
    /// Descrição do status retornado pela SEFAZ.
    pub x_motivo: String,
    /// XML enviado na requisição SOAP.
    pub sent_xml: String,
    /// XML recebido da SEFAZ.
    pub received_xml: String,
    /// URL do endpoint consultado.
    pub url: String,
}

/// Builder fluente para consulta de status do webservice SEFAZ.
///
/// # Exemplo
///
/// ```no_run
/// use dfe::NFeService;
///
/// # async fn example() -> Result<(), String> {
/// let r = NFeService::new()
///     .cert_path("./cert.pfx")
///     .cert_pass("senha")
///     .uf("SP")
///     .environment(2)
///     .modelo(65) // NFC-e; sem isto consulta o da NF-e (55)
///     .send()
///     .await?;
///
/// println!("{} — {}", r.c_stat, r.x_motivo);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct NFeService {
    /// Caminho do certificado A1 (`.pfx`).
    pub cert_path: String,
    /// Senha do certificado.
    pub cert_pass: String,
    /// Sigla da UF (ex.: `"SP"`, `"RJ"`).
    pub uf: String,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub environment: u8,
    /// Modelo do documento: `55` = NF-e (padrão) · `65` = NFC-e. Escolhe o webservice.
    pub modelo: u32,
}

impl NFeService {
    /// Cria um builder vazio.
    pub fn new() -> Self {
        Self {
            cert_path: String::new(),
            cert_pass: String::new(),
            uf: String::new(),
            environment: 0,
            modelo: 55,
        }
    }

    /// Caminho do certificado A1 (`.pfx`).
    pub fn cert_path(mut self, cert_path: &str) -> Self {
        self.cert_path = cert_path.to_string();
        self
    }

    /// Senha do certificado.
    pub fn cert_pass(mut self, cert_pass: &str) -> Self {
        self.cert_pass = cert_pass.to_string();
        self
    }

    /// Sigla da UF (ex.: `"SP"`, `"RJ"`).
    pub fn uf(mut self, uf: &str) -> Self {
        self.uf = uf.to_string();
        self
    }

    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub fn environment(mut self, environment: u8) -> Self {
        self.environment = environment;
        self
    }

    /// Modelo: `55` = NF-e (padrão) · `65` = NFC-e. Cada um tem o seu webservice de status.
    pub fn modelo(mut self, modelo: u32) -> Self {
        self.modelo = modelo;
        self
    }

    /// Valida os campos sem enviar a requisição. Retorna `Err` se algum campo obrigatório estiver vazio.
    pub fn build(self) -> Result<Self, String> {
        validation::validate_nfe_service(&self)?;

        Ok(self)
    }

    /// Consulta o status do webservice SEFAZ e retorna [`NFeServiceResponse`].
    ///
    /// # Erros
    ///
    /// Retorna `Err(String)` se a requisição SOAP falhar ou a UF não for suportada.
    pub async fn send(self) -> Result<NFeServiceResponse, String> {
        validation::validate_nfe_service(&self)?;

        let url = endpoint::status_url(self.environment, &self.uf, self.modelo)?;
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
