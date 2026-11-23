//! Trava: **teste nunca fala com a SEFAZ de produção.**
//!
//! Roda no `cargo test` padrão (sem a feature `sefaz-live`) e lê o fonte dos testes de
//! integração: todo `.ambiente(…)`, `.tp_amb(…)`, `.environment(…)` e `tp_amb: …` tem de usar
//! `common::HOMOLOGACAO`, e essa constante tem de valer 2. Antes desta trava, seis testes de
//! distribuição/manifestação mandavam `ambiente(1)` — com um `.env` de verdade, enviariam
//! eventos conclusivos em produção sobre uma NF-e real.

use std::fs;
use std::path::Path;

const CHAMADAS: [&str; 4] = [".ambiente(", ".tp_amb(", ".environment(", "tp_amb:"];

#[test]
fn constante_de_homologacao_vale_2() {
    let common = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/common/mod.rs")).unwrap();
    assert!(
        common.contains("pub const HOMOLOGACAO: u8 = 2;"),
        "tests/common/mod.rs: HOMOLOGACAO tem de ser 2 (homologação)"
    );
}

/// NFS-e: `HOMOLOGACAO_NFSE` (que também começa com `common::HOMOLOGACAO`, e por isso passa na
/// varredura abaixo) tem de ser a produção restrita.
#[test]
fn constante_de_homologacao_da_nfse_e_producao_restrita() {
    let common = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/common/mod.rs")).unwrap();
    assert!(
        common.contains(
            "pub const HOMOLOGACAO_NFSE: dfe::nfse::Ambiente = dfe::nfse::Ambiente::ProducaoRestrita;"
        ),
        "tests/common/mod.rs: HOMOLOGACAO_NFSE tem de ser Ambiente::ProducaoRestrita"
    );
    let constantes = common.matches("pub const HOMOLOGACAO").count();
    assert_eq!(constantes, 2, "constante HOMOLOGACAO* nova precisa de trava própria aqui");
}

#[test]
fn nenhum_teste_escolhe_o_ambiente_por_conta_propria() {
    let pasta = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut fora = Vec::new();
    for entrada in fs::read_dir(&pasta).unwrap() {
        let caminho = entrada.unwrap().path();
        let nome = caminho.file_name().unwrap().to_string_lossy().to_string();
        if !nome.ends_with(".rs") || nome == "homologacao_travada.rs" {
            continue;
        }
        let fonte = fs::read_to_string(&caminho).unwrap();
        for (n, linha) in fonte.lines().enumerate() {
            let codigo = linha.split("//").next().unwrap_or("");
            for chamada in CHAMADAS {
                if let Some(pos) = codigo.find(chamada) {
                    let resto = codigo[pos + chamada.len()..].trim_start();
                    if !resto.starts_with("common::HOMOLOGACAO") {
                        fora.push(format!("tests/{nome}:{}: {}", n + 1, linha.trim()));
                    }
                }
            }
        }
    }
    assert!(fora.is_empty(), "ambiente fora de common::HOMOLOGACAO:\n{}", fora.join("\n"));
}
