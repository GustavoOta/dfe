use libxml::parser::Parser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};

pub async fn validate_xml(xml: String, xsd: String) -> Result<String, String> {
    // spawn new async tokio thread
    let result = tokio::task::spawn_blocking(move || is_xml_valid(&xml, &xsd)).await;
    if let Err(e) = result {
        return Err(format!("{}", e));
    }
    Ok(result.unwrap().unwrap())
}
pub fn is_xml_valid(xml: &str, xsd: &str) -> Result<String, String> {
    let raw_incoming_xml = xml.to_string();
    // Parse do XML
    let doc = Parser::default()
        .parse_string(xml)
        .expect("Erro ao parsear XML");
    // Criação do contexto do XSD
    let mut schema_parser = SchemaParserContext::from_file(xsd);
    let xsd = SchemaValidationContext::from_parser(&mut schema_parser);

    if let Err(_) = xsd {
        /* for err in &errors {
            println!("{}", err.message.as_ref().unwrap());
        } */

        return Err("Erro ao criar contexto de validação XSD PL_010b_NT2025_002_v1.21".to_string());
    }
    // Validação
    let mut xsd = xsd.unwrap();

    if let Err(errors) = xsd.validate_document(&doc) {
        for err in &errors {
            //println!("{}", err.message.as_ref().unwrap());
            return Err(err.message.as_ref().unwrap().to_string());
        }

        return Err("Erro de validação do XML".to_string());
    } else {
        //println!("XML válido de acordo com o XSD.");
        return Ok(raw_incoming_xml);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xml_invalid() {
        let xml = std::fs::read_to_string("D:/Projetos/dfe-api/nfe_request.xml")
            .expect("Arquivo XML de teste não encontrado ou não pôde ser lido");

        let xsd = "./dfe/shema/PL_010b_NT2025_002_v1.21/nfe_v4.00.xsd";
        let result = is_xml_valid(&xml, xsd);

        if let Err(e) = result {
            println!("Erro esperado: {:?}", e);
        } else {
            // show validation result
            println!("XML válido: {:?}", result.unwrap());
        }
    }
}
