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

// ── Impressão da NFC-e em contingência off-line (sem nfeProc) ──────────────────────────
// Bug de campo (2026-09-21): a nota de contingência é um `<NFe>` sem `nfeProc` e a leitura
// estrita falhava com "missing field `@versao`" — o PDV não conseguia imprimir o cupom.

#[test]
fn leitura_estrita_continua_recusando_nfe_sem_protocolo() {
    let xml = common::nfce65_contingencia_sem_protocolo();
    assert!(XmlExtractor::new().nfe_proc_from_string(&xml).is_err());
}

#[test]
fn impressao_aceita_nfce_em_contingencia_sem_protocolo() {
    let xml = common::nfce65_contingencia_sem_protocolo();
    let proc_ = XmlExtractor::new()
        .nfe_proc_para_impressao(&xml)
        .expect("NFC-e em contingência deve ser lida para impressão");
    assert_eq!(proc_.versao, "4.00");
    assert_eq!(proc_.nfe.inf_nfe.ide.tp_emis.as_deref(), Some("9"));
    assert!(proc_.prot_nfe.inf_prot.is_none(), "contingência não tem protocolo");
}

#[test]
fn impressao_recusa_nfe_normal_sem_protocolo() {
    // tpEmis=1 sem nfeProc = nota não autorizada: não pode virar DANFE.
    let xml = common::nfe_sem_protocolo_com_tp_emis("nfce65_autorizada.xml", "1");
    let err = XmlExtractor::new().nfe_proc_para_impressao(&xml).unwrap_err();
    assert!(err.to_string().contains("tpEmis=9"), "erro inesperado: {err}");
}

#[test]
fn impressao_recusa_nfe_55_sem_protocolo_mesmo_com_tp_emis_9() {
    // tpEmis=9 só existe para NFC-e (mod 65).
    let xml = common::nfe_sem_protocolo_com_tp_emis("nfe55_autorizada.xml", "9");
    assert!(XmlExtractor::new().nfe_proc_para_impressao(&xml).is_err());
}

#[test]
fn impressao_le_nfe_proc_autorizado_como_antes() {
    let xml = common::read_xml_fixture("nfce65_autorizada.xml");
    let proc_ = XmlExtractor::new().nfe_proc_para_impressao(&xml).unwrap();
    assert!(proc_.prot_nfe.inf_prot.is_some());
}
