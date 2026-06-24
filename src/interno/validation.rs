use crate::error::{DfeError, Result};
use libxml::parser::Parser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// XSDs do leiaute NT2026.004 (PL_010d) — CNPJ e chave de acesso ALFANUMÉRICOS.
// Os patterns novos são retrocompatíveis: [0-9A-Z]{12}[0-9]{2} aceita CNPJ numérico
// e a chave [0-9]{6}[0-9A-Z]{12}[0-9]{26} aceita chave numérica.
// A pasta anterior (../schemas, leiaute numérico) é mantida intacta para rollback.
static XSD_FILES: &[(&str, &[u8])] = &[
    ("nfe_v4.00.xsd",                 include_bytes!("../schemas_nt2026_004/nfe_v4.00.xsd")),
    ("tiposBasico_v4.00.xsd",         include_bytes!("../schemas_nt2026_004/tiposBasico_v4.00.xsd")),
    ("leiauteNFe_v4.00.xsd",          include_bytes!("../schemas_nt2026_004/leiauteNFe_v4.00.xsd")),
    ("xmldsig-core-schema_v1.01.xsd", include_bytes!("../schemas_nt2026_004/xmldsig-core-schema_v1.01.xsd")),
    ("DFeTiposBasicos_v1.00.xsd",     include_bytes!("../schemas_nt2026_004/DFeTiposBasicos_v1.00.xsd")),
];

static SCHEMA_DIR: OnceLock<std::result::Result<PathBuf, String>> = OnceLock::new();

fn schema_dir() -> Result<&'static Path> {
    SCHEMA_DIR
        .get_or_init(|| extract_schemas().map_err(|e| e.to_string()))
        .as_ref()
        .map(|p| p.as_path())
        .map_err(|e| DfeError::Validacao(e.clone()))
}

fn extract_schemas() -> Result<PathBuf> {
    let dir = std::env::temp_dir().join("dfe_schemas_nfe");
    std::fs::create_dir_all(&dir)?;
    for (name, bytes) in XSD_FILES {
        std::fs::write(dir.join(name), bytes)?;
    }
    Ok(dir)
}

pub fn is_xml_valid(xml: &str) -> Result<String> {
    let dir = schema_dir()?;
    let nfe_xsd = dir.join("nfe_v4.00.xsd");
    let nfe_xsd_str = nfe_xsd.to_string_lossy();

    let doc = Parser::default()
        .parse_string(xml)
        .map_err(|_| DfeError::Xml("Erro ao parsear o XML".to_string()))?;

    let mut schema_parser = SchemaParserContext::from_file(&nfe_xsd_str);
    let mut xsd = SchemaValidationContext::from_parser(&mut schema_parser)
        .map_err(|e| DfeError::Validacao(format!("Erro ao criar contexto de validação XSD: {:?}", e)))?;

    if let Err(errors) = xsd.validate_document(&doc) {
        let msg = errors
            .first()
            .and_then(|e| e.message.as_deref())
            .unwrap_or("Erro de validação do XML");
        return Err(DfeError::Validacao(msg.to_string()));
    }

    Ok(xml.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xml_invalid() {
        let xml = std::fs::read_to_string("D:/Projetos/dfe-api/nfe_request.xml")
            .expect("Arquivo XML de teste não encontrado ou não pôde ser lido");
        let result = is_xml_valid(&xml);
        if let Err(e) = result {
            println!("Erro esperado: {:?}", e);
        } else {
            println!("XML válido: {:?}", result.unwrap());
        }
    }
}
