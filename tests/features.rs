#![cfg(any(feature = "emissao", all(feature = "danfe", feature = "escpos")))]

use std::process::Command;

fn inclui_libxml(features: &str) -> bool {
    let output = Command::new(env!("CARGO"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "tree",
            "--offline",
            "--no-default-features",
            "--features",
            features,
            "--edges",
            "normal,build",
            "--prefix",
            "none",
        ])
        .output()
        .expect("executar cargo tree");

    assert!(
        output.status.success(),
        "cargo tree falhou para {features}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.starts_with("libxml v"))
}

#[test]
#[cfg(all(feature = "danfe", feature = "escpos"))]
fn renderizacao_dispensa_libxml() {
    assert!(!inclui_libxml("danfe,escpos"));
}

#[test]
#[cfg(feature = "emissao")]
fn emissao_inclui_libxml_para_validar_o_xsd() {
    assert!(inclui_libxml("emissao"));
}
