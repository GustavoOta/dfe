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

/// Valida um fragmento (ex.: `<ICMS>…</ICMS>`) contra a definição do elemento no
/// `leiauteNFe` embutido. Só para teste: a NF-e inteira precisa de assinatura, e o que se
/// quer travar aqui é a ordem e o formato dos campos de um grupo.
///
/// Recorta do leiaute o `xs:element` pedido (e o `Torig`, único tipo que ele não tira do
/// `tiposBasico`) e monta um schema mínimo ao lado dos XSDs extraídos.
#[cfg(test)]
pub(crate) fn validar_fragmento(elemento: &str, fragmento: &str) -> Result<()> {
    let dir = schema_dir()?;
    let leiaute = String::from_utf8_lossy(
        XSD_FILES.iter().find(|(n, _)| *n == "leiauteNFe_v4.00.xsd").map(|(_, b)| *b).unwrap_or_default(),
    )
    .into_owned();

    let recorte = |abertura: &str, tag: &str| -> Option<String> {
        let inicio = leiaute.find(abertura)?;
        let mut prof = 0i32;
        let mut pos = inicio;
        loop {
            let prox_abre = leiaute[pos..].find(&format!("<xs:{tag}")).map(|i| pos + i);
            let prox_fecha = leiaute[pos..].find(&format!("</xs:{tag}>")).map(|i| pos + i)?;
            match prox_abre {
                Some(a) if a < prox_fecha => {
                    let fim_tag = leiaute[a..].find('>')? + a;
                    if leiaute.as_bytes()[fim_tag - 1] != b'/' {
                        prof += 1;
                    }
                    pos = fim_tag + 1;
                }
                _ => {
                    prof -= 1;
                    pos = prox_fecha + format!("</xs:{tag}>").len();
                    if prof == 0 {
                        return Some(leiaute[inicio..pos].to_string());
                    }
                }
            }
        }
    };

    // Elemento com tipo nomeado (ex.: `IBSCBS type="TTribNFe" minOccurs="0"`, que vem do
    // `DFeTiposBasicos`): no schema mínimo ele vira global, onde `minOccurs` não é permitido.
    let el = recorte(&format!(r#"<xs:element name="{elemento}">"#), "element")
        .or_else(|| {
            recorte(&format!(r#"<xs:element name="{elemento}" "#), "element")
                .map(|e| e.replacen(r#" minOccurs="0""#, "", 1))
        })
        .ok_or_else(|| DfeError::Validacao(format!("elemento {elemento} não achado no leiaute")))?;
    // Todos os tipos simples globais do leiaute (Torig, TGuid…): o elemento recortado pode
    // usar qualquer um deles (o <prod> usa TGuid no nFCI).
    let mut torig = String::new();
    let mut pos = 0;
    while let Some(i) = leiaute[pos..].find(r#"<xs:simpleType name=""#) {
        let ini = pos + i;
        let nome_ini = ini + r#"<xs:simpleType name=""#.len();
        let nome_fim = leiaute[nome_ini..].find('"').map(|j| nome_ini + j).unwrap_or(nome_ini);
        let nome = &leiaute[nome_ini..nome_fim];
        if let Some(t) = recorte(&format!(r#"<xs:simpleType name="{nome}">"#), "simpleType") {
            pos = ini + t.len();
            torig.push_str(&t);
            torig.push('\n');
        } else {
            pos = nome_fim;
        }
    }

    let xsd = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xs:schema xmlns="http://www.portalfiscal.inf.br/nfe" xmlns:xs="http://www.w3.org/2001/XMLSchema" targetNamespace="http://www.portalfiscal.inf.br/nfe" elementFormDefault="qualified" attributeFormDefault="unqualified">
<xs:include schemaLocation="tiposBasico_v4.00.xsd"/>
<xs:include schemaLocation="DFeTiposBasicos_v1.00.xsd"/>
{torig}
{el}
</xs:schema>"#
    );
    // Um arquivo por chamada: os testes rodam em paralelo e um reescreveria o schema do outro.
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let caminho = dir.join(format!("fragmento_{elemento}_{}_{n}.xsd", std::process::id()));
    std::fs::write(&caminho, xsd).map_err(|e| DfeError::Validacao(e.to_string()))?;

    let xml = fragmento.replacen(
        &format!("<{elemento}>"),
        &format!(r#"<{elemento} xmlns="http://www.portalfiscal.inf.br/nfe">"#),
        1,
    );
    let doc = Parser::default()
        .parse_string(&xml)
        .map_err(|_| DfeError::Xml(format!("fragmento mal formado: {xml}")))?;
    let mut parser = SchemaParserContext::from_file(&caminho.to_string_lossy());
    let mut ctx = SchemaValidationContext::from_parser(&mut parser)
        .map_err(|e| DfeError::Validacao(format!("schema do fragmento: {:?}", e)))?;
    ctx.validate_document(&doc).map_err(|erros| {
        DfeError::Validacao(format!(
            "{} → {}",
            erros.first().and_then(|e| e.message.as_deref()).unwrap_or("inválido").trim(),
            xml
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validar_fragmento_aceita_icms00_certo_e_recusa_fora_de_ordem() {
        let certo = "<ICMS><ICMS00><orig>0</orig><CST>00</CST><modBC>3</modBC><vBC>10.00</vBC><pICMS>18.0000</pICMS><vICMS>1.80</vICMS></ICMS00></ICMS>";
        assert!(validar_fragmento("ICMS", certo).is_ok(), "{:?}", validar_fragmento("ICMS", certo));
        let fora = "<ICMS><ICMS00><orig>0</orig><CST>00</CST><vBC>10.00</vBC><modBC>3</modBC><pICMS>18.0000</pICMS><vICMS>1.80</vICMS></ICMS00></ICMS>";
        assert!(validar_fragmento("ICMS", fora).is_err());
    }

    #[test]
    fn test_is_xml_invalid() {
        // XML bem-formado, porém inválido contra o XSD nfe_v4.00: o `infNFe` não tem os
        // elementos obrigatórios (ide/emit/det/total/transp/pag). `is_xml_valid` deve
        // rejeitar com `DfeError::Validacao`. (Auto-contido — sem arquivo externo.)
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe versao="4.00" Id="NFe35000000000000000000550010000000011000000001"><ide/></infNFe></NFe>"#;
        let result = is_xml_valid(xml);
        assert!(
            result.is_err(),
            "esperava Err de validação XSD para XML inválido, veio: {:?}",
            result
        );
    }
}
