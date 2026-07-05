//! Camada 2 (hermética) — `EscPosNFCeBuilder`.
//!
//! Golden `insta` (Fase 0.3): snapshot de uma representação textual **estável** dos bytes
//! ESC/POS (printáveis ASCII literais; bytes de controle como `\xNN`). A saída é
//! determinística dado o XML fixo (o gerador não usa relógio de parede) → sem redaction.
//! Aceitar mudanças com `cargo insta review` após inspecionar o diff.

#![cfg(feature = "escpos")]

use dfe::EscPosNFCeBuilder;

mod common;

/// Representação estável e legível de um buffer ESC/POS: mantém ASCII imprimível e `\n`
/// (com quebra real, p/ diff legível), escapando o resto como `\xNN`. Bijetiva o bastante
/// para que qualquer alteração de bytes altere o snapshot.
fn escape_escpos(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        match b {
            b'\n' => s.push_str("\\n\n"),
            0x20..=0x7e => s.push(b as char),
            _ => s.push_str(&format!("\\x{b:02x}")),
        }
    }
    s
}

#[test]
fn golden_escpos_nfce_80mm() {
    let xml = common::read_xml_fixture("nfce65_autorizada.xml");
    let bytes = EscPosNFCeBuilder::new()
        .xml(xml)
        .paper_width(80)
        .build()
        .expect("gerar ESC/POS da NFC-e");
    assert!(!bytes.is_empty(), "ESC/POS deve gerar bytes não vazios");
    insta::assert_snapshot!(escape_escpos(&bytes));
}
