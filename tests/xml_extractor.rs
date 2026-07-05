//! Camada 2 (hermética) — `XmlExtractor` / `XmlExtractorSignature`.
//!
//! Golden `insta` (Fase 0.3): snapshot `Debug` da struct extraída das fixtures curadas.
//! Determinístico (fixture estática, sem relógio) → sem redaction. Aceitar mudanças com
//! `cargo insta review` (ou `INSTA_UPDATE=always cargo test`) após inspecionar o diff.

use dfe::{XmlExtractor, XmlExtractorSignature};

mod common;

#[test]
fn golden_extrai_nfe_proc_da_fixture_55() {
    let xml = common::read_xml_fixture("nfe55_autorizada.xml");
    let proc = XmlExtractor::new()
        .nfe_proc_from_string(&xml)
        .expect("extrair nfeProc da fixture 55");
    insta::assert_debug_snapshot!(proc);
}

#[test]
fn golden_extrai_nfe_proc_da_fixture_65() {
    let xml = common::read_xml_fixture("nfce65_autorizada.xml");
    let proc = XmlExtractor::new()
        .nfe_proc_from_string(&xml)
        .expect("extrair nfeProc da fixture 65");
    insta::assert_debug_snapshot!(proc);
}

/// Preserva o caso movido de `src/bin/tests/test_xml_extractor.rs` (Fase 0.2): o extractor
/// deve lidar com um `NFe/infNFe` mínimo (só `cUF`) sem panicar.
#[test]
fn smoke_extrai_de_xml_minimo_inline() {
    let xml = r#"<NFe><infNFe><ide><cUF>35</cUF></ide></infNFe></NFe>"#;
    let _ = XmlExtractor::new().nfe_proc_from_string(xml);
    let _ = XmlExtractor::new().nfe_from_string(xml);
}
