use crate::interno::ws::nfe_status_servico;

pub fn status_url(environment: u8, uf: &str) -> Result<String, String> {
    nfe_status_servico(environment, uf, 55, false)
        .map(|url| url.to_string())
        .map_err(|e| e.to_string())
}
