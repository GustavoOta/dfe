use libxml::parser::Parser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
const XSD_DIR: &str = "./dfe/shema/PL_010b_NT2025_002_v1.21";
const NFE_XSD_PATH: &str = "./dfe/shema/PL_010b_NT2025_002_v1.21/nfe_v4.00.xsd";
const XSD_BASE_URL: &str =
    "https://raw.githubusercontent.com/GustavoOta/dfe/main/dfe/shema/PL_010b_NT2025_002_v1.21";
const XSD_FILES: &[&str] = &[
    "nfe_v4.00.xsd",
    "tiposBasico_v4.00.xsd",
    "leiauteNFe_v4.00.xsd",
    "xmldsig-core-schema_v1.01.xsd",
    "DFeTiposBasicos_v1.00.xsd",
];

pub async fn validate_xml(xml: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || is_xml_valid(&xml))
        .await
        .map_err(|e| format!("Erro na thread de validação: {}", e))?
}

pub fn is_xml_valid(xml: &str) -> Result<String, String> {
    ensure_xsd_files_exist()?;

    let raw_incoming_xml = xml.to_string();

    let doc = Parser::default()
        .parse_string(xml)
        .map_err(|_| "Erro ao parsear o XML".to_string())?;

    let mut schema_parser = SchemaParserContext::from_file(NFE_XSD_PATH);
    let mut xsd = SchemaValidationContext::from_parser(&mut schema_parser)
        .map_err(|e| format!("Erro ao criar contexto de validação XSD: {:?}", e))?;

    if let Err(errors) = xsd.validate_document(&doc) {
        let msg = errors
            .first()
            .and_then(|e| e.message.as_deref())
            .unwrap_or("Erro de validação do XML");
        return Err(msg.to_string());
    }

    Ok(raw_incoming_xml)
}

fn download_xsd_file(filename: &str) -> Result<(), String> {
    let url = format!("{}/{}", XSD_BASE_URL, filename);
    let path = format!("{}/{}", XSD_DIR, filename);

    let response = reqwest::blocking::get(&url).map_err(|e| {
        format!(
            "Arquivo '{}' não existia, tentei baixar e não consegui: {}",
            filename, e
        )
    })?;
    let content = response.text().map_err(|e| {
        format!(
            "Arquivo '{}' não existia, tentei baixar e não consegui: {}",
            filename, e
        )
    })?;
    std::fs::write(&path, content).map_err(|e| {
        format!(
            "Arquivo '{}' não existia, tentei baixar e não consegui: {}",
            filename, e
        )
    })?;
    println!(
        "Arquivo '{}' baixado e salvo com sucesso em '{}'",
        filename, path
    );
    Ok(())
}

fn ensure_xsd_files_exist() -> Result<(), String> {
    if !std::path::Path::new(XSD_DIR).exists() {
        std::fs::create_dir_all(XSD_DIR)
            .map_err(|e| format!("Erro ao criar diretório '{}': {}", XSD_DIR, e))?;
    }
    for filename in XSD_FILES {
        let path = format!("{}/{}", XSD_DIR, filename);
        if !std::path::Path::new(&path).exists() {
            download_xsd_file(filename)?;
        }
    }
    Ok(())
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
            // show validation result
            println!("XML válido: {:?}", result.unwrap());
        }
    }
}
