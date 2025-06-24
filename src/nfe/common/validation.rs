use libxml::parser::Parser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};

pub fn is_xml_valid(xml: &str, xsd: &str) -> Result<(), String> {
    // Parse do XML
    let doc = Parser::default()
        .parse_string(xml)
        .expect("Erro ao parsear XML");
    // Criação do contexto do XSD
    let mut schema_parser = SchemaParserContext::from_file(xsd);
    let mut validation_ctx =
        SchemaValidationContext::from_parser(&mut schema_parser).expect("Erro no Parser de XSD");

    // Validação
    match validation_ctx.validate_document(&doc) {
        Ok(_) => Ok(()),
        Err(errors) => {
            let mut xml_error = "Erro de validacao XSD: ".to_string();
            for err in errors {
                let current_error = err.message.unwrap_or("Erro desconhecido".to_string());
                xml_error.push_str(&format!("{}\n", current_error));
            }
            return Err(xml_error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_xml_invalid() {
        let xml = std::fs::read_to_string("D:\\Projetos\\dfe-api\\xml_validation_error.xml")
            .expect("Arquivo XML de teste não encontrado ou não pôde ser lido");
        let xsd = "./dfe/shema/PL_009p_NT2024_003_v1.03/nfe_v4.00.xsd";
        let result = is_xml_valid(&xml, xsd);
        assert!(
            result.is_err(),
            "O XML deveria ser inválido, mas passou na validação"
        );
        if let Err(e) = result {
            println!("Erro esperado: {:?}", e);
        }
    }
}
