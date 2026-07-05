//! Helpers compartilhados pelos testes de integração (Camadas 2 e 3 do plano de testes).
//!
//! A subpasta `tests/common/` **não** é tratada como binário de teste pelo Cargo — é um
//! módulo importado por cada binário via `mod common;`. Cada binário usa só um subconjunto
//! destes helpers, por isso o `allow(dead_code)`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// Diretório raiz das fixtures (`tests/fixtures/`), resolvido a partir de
/// `CARGO_MANIFEST_DIR` — **independente do CWD** do runner de teste.
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Caminho de uma fixture XML em `tests/fixtures/xml/<nome>`.
pub fn xml_fixture_path(nome: &str) -> PathBuf {
    fixtures_dir().join("xml").join(nome)
}

/// Lê uma fixture XML como `String`. Faz `panic!` com o caminho se não existir —
/// numa fixture ausente o teste deve falhar ruidosamente, não silenciar.
pub fn read_xml_fixture(nome: &str) -> String {
    let path = xml_fixture_path(nome);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("falha ao ler a fixture {:?}: {e}", path))
}

/// Credenciais/parâmetros para a **camada `sefaz-live`** (Camada 3), lidos do `.env`
/// (ver `env.example`). Retorna `None` quando `DFE_TEST_CERT_PATH` não está definido →
/// o teste live deve ser **ignorado (skip)**, nunca falhar por falta de segredo.
pub struct LiveEnv {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub uf: String,
    pub tp_amb: u8,
}

/// Lê o ambiente de teste live do `.env`. `None` se não configurado.
pub fn live_env() -> Option<LiveEnv> {
    dotenvy::dotenv().ok();
    let non_empty = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());

    let cert_path = non_empty("DFE_TEST_CERT_PATH")?;
    Some(LiveEnv {
        cert_path,
        cert_pass: std::env::var("DFE_TEST_CERT_PASS").unwrap_or_default(),
        cnpj: non_empty("DFE_TEST_CNPJ").unwrap_or_default(),
        uf: non_empty("DFE_TEST_UF").unwrap_or_else(|| "SP".to_string()),
        tp_amb: non_empty("DFE_TEST_TP_AMB")
            .and_then(|v| v.parse().ok())
            .unwrap_or(2),
    })
}
