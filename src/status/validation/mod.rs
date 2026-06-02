use std::fs;
use std::path::Path;

use super::NFeService;

pub fn validate_nfe_service(service: &NFeService) -> Result<(), String> {
    if service.cert_path.trim().is_empty() {
        return Err("O path do certificado e obrigatorio".to_string());
    }

    let path = Path::new(&service.cert_path);
    let exists = path
        .try_exists()
        .map_err(|e| format!("Erro ao verificar o path do certificado: {}", e))?;

    if !exists {
        return Err(format!("Arquivo não encontrado: {}", service.cert_path));
    }

    let metadata =
        fs::metadata(path).map_err(|e| format!("Erro ao ler metadata do certificado: {}", e))?;

    if !metadata.is_file() {
        return Err(format!(
            "O path do certificado nao aponta para um arquivo: {}",
            service.cert_path
        ));
    }

    if service.cert_pass.trim().is_empty() {
        return Err("A senha do certificado e obrigatoria".to_string());
    }

    if service.uf.trim().is_empty() {
        return Err("A UF e obrigatoria".to_string());
    }

    if service.environment != 1 && service.environment != 2 {
        return Err("O ambiente deve ser 1 (producao) ou 2 (homologacao)".to_string());
    }

    Ok(())
}
