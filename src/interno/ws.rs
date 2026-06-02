use crate::error::{DfeError, Result};
use serde::Deserialize;
use std::sync::OnceLock;

const WEBSERVICES_JSON: &str = include_str!("../data/webservices.json");

#[derive(Deserialize)]
struct Endpoint {
    servico: String,
    ambiente: u8,
    uf: String,
    modelo: u32,
    svn: bool,
    url: String,
}

static ENDPOINTS: OnceLock<Vec<Endpoint>> = OnceLock::new();

fn endpoints() -> &'static Vec<Endpoint> {
    ENDPOINTS.get_or_init(|| {
        serde_json::from_str(WEBSERVICES_JSON)
            .expect("webservices.json embutido é inválido — erro de build")
    })
}

fn lookup(servico: &str, ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    endpoints()
        .iter()
        .find(|e| {
            e.servico == servico
                && e.ambiente == ambiente
                && e.uf == uf
                && e.modelo == modelo
                && e.svn == svn
        })
        .map(|e| e.url.as_str())
        .ok_or_else(|| {
            DfeError::Webservice(format!(
                "Endpoint não encontrado: servico={} ambiente={} uf={} modelo={} svn={}",
                servico, ambiente, uf, modelo, svn
            ))
        })
}

pub fn nfe_status_servico(ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    lookup("NfeStatusServico", ambiente, uf, modelo, svn)
}

pub fn nfe_autorizacao(ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    lookup("NFeAutorizacao", ambiente, uf, modelo, svn)
}

pub fn nfe_recepcao_evento(ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    lookup("RecepcaoEvento", ambiente, uf, modelo, svn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webservices_json_valido() {
        let _ = endpoints();
    }

    #[test]
    fn test_get_ws_url() {
        let url = nfe_status_servico(2, "SP", 55, false);
        assert_eq!(
            url.unwrap(),
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx"
        );
    }

    #[test]
    fn test_endpoint_nao_encontrado() {
        let url = nfe_status_servico(1, "XX", 55, false);
        assert!(url.is_err());
    }
}
