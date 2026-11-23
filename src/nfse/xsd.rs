//! Validação da DPS contra o XSD oficial do ambiente (`schemas/prod` e `schemas/rest`).
//!
//! Os pacotes são extraídos para o temp do SO na primeira validação, uma pasta por pacote, no
//! mesmo esquema de `interno::validation` (NF-e). O XSD de produção tem **uma** linha corrigida
//! (`TSSerieDPS` sem `^`/`$` literais); origem, hashes e o motivo estão em `schemas/README.md`.

use super::Ambiente;
use crate::error::{DfeError, Result};
use libxml::parser::Parser;
use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

type Pacote = [(&'static str, &'static [u8]); 4];

/// Produção v1.01-20260209, com a série corrigida.
static PRODUCAO: Pacote = [
    (
        "DPS_v1.01.xsd",
        include_bytes!("schemas/prod/DPS_v1.01.xsd"),
    ),
    (
        "tiposComplexos_v1.01.xsd",
        include_bytes!("schemas/prod/tiposComplexos_v1.01.xsd"),
    ),
    (
        "tiposSimples_v1.01.xsd",
        include_bytes!("schemas/prod/tiposSimples_v1.01.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("schemas/prod/xmldsig-core-schema.xsd"),
    ),
];

/// Produção restrita v1.01-20261020, sem alteração.
static PRODUCAO_RESTRITA: Pacote = [
    (
        "DPS_v1.01.xsd",
        include_bytes!("schemas/rest/DPS_v1.01.xsd"),
    ),
    (
        "tiposComplexos_v1.01.xsd",
        include_bytes!("schemas/rest/tiposComplexos_v1.01.xsd"),
    ),
    (
        "tiposSimples_v1.01.xsd",
        include_bytes!("schemas/rest/tiposSimples_v1.01.xsd"),
    ),
    (
        "xmldsig-core-schema.xsd",
        include_bytes!("schemas/rest/xmldsig-core-schema.xsd"),
    ),
];

static PASTA_PRODUCAO: OnceLock<std::result::Result<PathBuf, String>> = OnceLock::new();
static PASTA_RESTRITA: OnceLock<std::result::Result<PathBuf, String>> = OnceLock::new();

fn pasta(ambiente: Ambiente) -> Result<&'static Path> {
    let (celula, pacote, nome) = match ambiente {
        Ambiente::Producao => (&PASTA_PRODUCAO, &PRODUCAO, "dfe_schemas_dps_v1.01_20260209"),
        Ambiente::ProducaoRestrita => (
            &PASTA_RESTRITA,
            &PRODUCAO_RESTRITA,
            "dfe_schemas_dps_v1.01_20261020",
        ),
    };
    celula
        .get_or_init(|| extrair(pacote, nome).map_err(|e| e.to_string()))
        .as_ref()
        .map(|p| p.as_path())
        .map_err(|e| DfeError::Validacao(e.clone()))
}

fn extrair(pacote: &Pacote, nome: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(nome);
    std::fs::create_dir_all(&dir)?;
    for (arquivo, bytes) in pacote {
        std::fs::write(dir.join(arquivo), bytes)?;
    }
    Ok(dir)
}

/// Valida o XML da DPS no XSD do ambiente. O erro traz todas as mensagens do libxml.
pub(super) fn validar_dps(ambiente: Ambiente, xml: &str) -> Result<()> {
    let xsd = pasta(ambiente)?.join("DPS_v1.01.xsd");
    let doc = Parser::default()
        .parse_string(xml)
        .map_err(|e| DfeError::Xml(format!("DPS: XML malformado: {e:?}")))?;
    let mut parser = SchemaParserContext::from_file(&xsd.to_string_lossy());
    let mut contexto = SchemaValidationContext::from_parser(&mut parser)
        .map_err(|e| DfeError::Validacao(format!("DPS: falha ao carregar o XSD: {e:?}")))?;
    contexto.validate_document(&doc).map_err(|erros| {
        let mensagens: Vec<String> = erros
            .iter()
            .filter_map(|e| e.message.as_deref())
            .map(|m| m.trim().to_string())
            .collect();
        DfeError::Validacao(format!("DPS fora do XSD: {}", mensagens.join(" | ")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trava a correção documentada: só a série muda em relação ao pacote oficial.
    #[test]
    fn xsd_de_producao_tem_a_serie_sem_ancoras_literais() {
        let simples = String::from_utf8_lossy(PRODUCAO[2].1);
        assert!(simples.contains(r#"<xs:pattern value="0{0,4}\d{1,5}"/>"#));
        assert!(!simples.contains(r"^0{0,4}"));
    }
}
