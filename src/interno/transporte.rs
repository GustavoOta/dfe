//! Transporte SOAP para os webservices da SEFAZ.
//!
//! Abstrai o envio HTTP por trás do trait [`SoapTransport`] para permitir **injeção de um
//! transporte falso nos testes** (caracterização offline de emissão/cancelamento — Camada 4
//! do plano de testes): o fake captura o envelope gerado e devolve uma resposta *canned*, sem
//! rede nem certificado. O caminho de produção é o [`MtlsTransport`] (mTLS com certificado A1).

use crate::error::{DfeError, Result};
use crate::interno::cert::Cert;
use crate::interno::connection::WebService;

/// Envio de um envelope SOAP 1.2 a um webservice da SEFAZ.
///
/// O retorno é o **corpo da resposta como texto** (o que os módulos parseiam). Um status HTTP
/// fora de 2xx vira [`DfeError::Webservice`]. Modelado com *return-position impl Trait* (em vez
/// de `async fn` no trait) para não disparar o lint `async_fn_in_trait` e manter o `Send`.
pub trait SoapTransport {
    /// Envia `body` para `url` e devolve o corpo da resposta.
    fn send_soap(
        &self,
        url: &str,
        body: String,
    ) -> impl std::future::Future<Output = Result<String>> + Send;
}

/// Transporte real: mTLS com certificado A1 (`.pfx`).
///
/// Lê o certificado do arquivo **a cada envio** — nunca cacheia o objeto `Cert` entre
/// requisições (exigência da crate; ver `CLAUDE.md`).
pub struct MtlsTransport {
    cert_path: String,
    cert_pass: String,
}

impl MtlsTransport {
    pub fn new(cert_path: impl Into<String>, cert_pass: impl Into<String>) -> Self {
        Self {
            cert_path: cert_path.into(),
            cert_pass: cert_pass.into(),
        }
    }
}

impl SoapTransport for MtlsTransport {
    async fn send_soap(&self, url: &str, body: String) -> Result<String> {
        let cert = Cert::from_pfx(&self.cert_path, &self.cert_pass)?;
        let client = WebService::client(cert.identity)?;

        // Content-Type é obrigatório para SOAP 1.2; Content-Length replica o header que os
        // módulos de emissão/cancelamento já enviavam (o reqwest calcularia o mesmo valor).
        let response = client
            .post(url)
            .header("Content-Type", "application/soap+xml; charset=utf-8")
            .header("Content-Length", body.len().to_string())
            .body(body)
            .send()
            .await
            .map_err(|e| DfeError::Webservice(format!("Falha ao enviar requisição: {e}")))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| DfeError::Webservice(format!("Falha ao ler corpo da resposta: {e}")))?;

        if !status.is_success() {
            return Err(DfeError::Webservice(format!(
                "Erro HTTP {status}: {}",
                text.chars().take(300).collect::<String>()
            )));
        }
        Ok(text)
    }
}
