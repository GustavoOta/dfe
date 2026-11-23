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

/// Ambiente de **todo** teste que fala com a SEFAZ: 2 = homologação. Fixo no código, nunca
/// no `.env` — teste não emite nem manifesta em produção. Travado por
/// `tests/homologacao_travada.rs` (nenhum `ambiente(1)`/`tp_amb: 1` passa).
pub const HOMOLOGACAO: u8 = 2;

/// O mesmo para a NFS-e: a SEFIN Nacional chama a homologação de **produção restrita**.
/// Travado junto com o [`HOMOLOGACAO`] por `tests/homologacao_travada.rs`.
#[cfg(feature = "nfse")]
pub const HOMOLOGACAO_NFSE: dfe::nfse::Ambiente = dfe::nfse::Ambiente::ProducaoRestrita;

/// Prestador habilitado no Emissor Nacional para o ciclo de NFS-e em produção restrita
/// (`DFE_TEST_NFSE_*` no `.env`, ver `env.example`). `None` = teste pulado.
#[cfg(feature = "nfse")]
pub struct LiveNfse {
    pub municipio: String,
    pub inscricao_municipal: Option<String>,
    pub mei: bool,
    pub codigo_tributacao_nacional: String,
    pub codigo_tributacao_municipal: Option<String>,
    pub serie: u32,
}

#[cfg(feature = "nfse")]
pub fn live_nfse() -> Option<LiveNfse> {
    live_env()?;
    let v = |k: &str| {
        std::env::var(k)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    Some(LiveNfse {
        municipio: v("DFE_TEST_NFSE_MUNICIPIO")?,
        inscricao_municipal: v("DFE_TEST_NFSE_IM"),
        mei: v("DFE_TEST_NFSE_MEI").is_some_and(|s| s == "1"),
        codigo_tributacao_nacional: v("DFE_TEST_NFSE_CTRIBNAC")?,
        codigo_tributacao_municipal: v("DFE_TEST_NFSE_CTRIBMUN"),
        serie: v("DFE_TEST_NFSE_SERIE")
            .and_then(|s| s.parse().ok())
            .unwrap_or(900),
    })
}

/// Credenciais/parâmetros para a **camada `sefaz-live`** (Camada 3), lidos do `.env`
/// (ver `env.example`). Retorna `None` quando `DFE_TEST_CERT_PATH` não está definido →
/// o teste live deve ser **ignorado (skip)**, nunca falhar por falta de segredo.
/// O ambiente não vem daqui: é sempre [`HOMOLOGACAO`].
pub struct LiveEnv {
    pub cert_path: String,
    pub cert_pass: String,
    pub cnpj: String,
    pub uf: String,
}

/// Lê o ambiente de teste live do `.env`. `None` se não configurado.
pub fn live_env() -> Option<LiveEnv> {
    // `.env` ausente = skip; `.env` que existe e não lê (ex.: `D:\pasta` sem aspas — o `\` é
    // escape) derruba o teste, em vez de pular calado como se não houvesse certificado.
    if let Err(e) = dotenvy::dotenv() {
        assert!(e.not_found(), ".env inválido: {e} — use `/` no caminho ou aspas simples");
    }
    let non_empty = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());

    let cert_path = caminho_da_plataforma(&non_empty("DFE_TEST_CERT_PATH")?);
    Some(LiveEnv {
        cert_path,
        cert_pass: std::env::var("DFE_TEST_CERT_PASS").unwrap_or_default(),
        // Aceita com ou sem pontuação ("47.810.175/0001-28"); a SEFAZ quer só os caracteres.
        cnpj: non_empty("DFE_TEST_CNPJ")
            .unwrap_or_default()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_uppercase())
            .collect(),
        uf: non_empty("DFE_TEST_UF").unwrap_or_else(|| "SP".to_string()),
    })
}

/// Emitente real para o ciclo de emissão em homologação (`DFE_TEST_IE`, `DFE_TEST_CRT`,
/// endereço, série — ver `env.example`). Fica no `.env`, nunca no código: a crate é pública.
/// `None` (teste pulado) quando faltar a IE ou o município.
pub struct LiveEmitente {
    pub ie: String,
    pub crt: u8,
    pub razao: String,
    pub logradouro: String,
    pub numero: String,
    pub complemento: Option<String>,
    pub bairro: String,
    pub cod_municipio: String,
    pub municipio: String,
    pub cep: String,
    pub serie: u32,
}

pub fn live_emitente() -> Option<LiveEmitente> {
    live_env()?;
    let v = |k: &str| std::env::var(k).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let digitos = |s: String| s.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
    Some(LiveEmitente {
        ie: digitos(v("DFE_TEST_IE")?),
        crt: v("DFE_TEST_CRT").and_then(|s| s.parse().ok()).unwrap_or(1),
        razao: v("DFE_TEST_RAZAO").unwrap_or_default(),
        logradouro: v("DFE_TEST_LOGRADOURO").unwrap_or_default(),
        numero: v("DFE_TEST_NUMERO").unwrap_or_else(|| "SN".into()),
        complemento: v("DFE_TEST_COMPLEMENTO"),
        bairro: v("DFE_TEST_BAIRRO").unwrap_or_default(),
        cod_municipio: digitos(v("DFE_TEST_COD_MUNICIPIO")?),
        municipio: v("DFE_TEST_MUNICIPIO").unwrap_or_default(),
        cep: digitos(v("DFE_TEST_CEP").unwrap_or_default()),
        serie: v("DFE_TEST_SERIE").and_then(|s| s.parse().ok()).unwrap_or(99),
    })
}

/// CSC de **homologação** da NFC-e (`DFE_TEST_ID_CSC`, `DFE_TEST_CSC`): `(id, código)`.
/// Credencial do contribuinte — só no `.env`. `None` = testes de NFC-e pulados.
pub fn live_csc() -> Option<(String, String)> {
    live_env()?;
    let v = |k: &str| std::env::var(k).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    Some((v("DFE_TEST_ID_CSC")?, v("DFE_TEST_CSC")?))
}

/// O mesmo `.env` serve ao Windows e ao WSL: fora do Windows, `D:\pasta\cert.pfx` vira
/// `/mnt/d/pasta/cert.pfx`. Caminho que já é do Linux passa intacto.
fn caminho_da_plataforma(caminho: &str) -> String {
    let b = caminho.as_bytes();
    let eh_windows = b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
    if cfg!(target_os = "windows") || !eh_windows {
        return caminho.to_string();
    }
    format!("/mnt/{}/{}", (b[0] as char).to_ascii_lowercase(), caminho[3..].replace('\\', "/"))
}

/// NFC-e em contingência off-line **como o PDV imprime**: o `<NFe>` assinado, sem `nfeProc`
/// nem `protNFe` (a SEFAZ ainda não autorizou) e com `tpEmis=9`. Derivada da fixture
/// autorizada — a assinatura fica inválida, o que não importa para ler/imprimir.
#[allow(dead_code)]
pub fn nfce65_contingencia_sem_protocolo() -> String {
    nfe_sem_protocolo_com_tp_emis("nfce65_autorizada.xml", "9")
}

/// `<NFe>` da fixture, sem o envelope `nfeProc`, com o `tpEmis` trocado.
#[allow(dead_code)]
pub fn nfe_sem_protocolo_com_tp_emis(fixture: &str, tp_emis: &str) -> String {
    let xml = read_xml_fixture(fixture);
    let inicio = xml.find("<NFe").expect("fixture sem <NFe>");
    let fim = xml.find("</NFe>").expect("fixture sem </NFe>") + "</NFe>".len();
    let nfe = &xml[inicio..fim];
    assert!(nfe.contains("<tpEmis>1</tpEmis>"), "fixture deveria ter tpEmis=1");
    nfe.replace("<tpEmis>1</tpEmis>", &format!("<tpEmis>{tp_emis}</tpEmis>"))
}
