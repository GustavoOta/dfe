//! # dfe
//!
//! Crate Rust para integração com os webservices da **SEFAZ** brasileira e da **SEFIN Nacional**.
//! Emite, cancela e distribui NF-e e NFC-e, emite NFS-e Nacional (feature `nfse`), gera DANFE em
//! PDF e imprime via ESC/POS.
//!
//! ## Início rápido
//!
//! ```toml
//! [dependencies]
//! dfe = "0.8"
//! ```
//!
//! ## Funcionalidades
//!
//! | Módulo | Responsabilidade |
//! |---|---|
//! | `emissao` | Emissão de NF-e e NFC-e via `NFeBuilder` |
//! | [`cancelar`] | Cancelamento via [`CancelarBuilder`] |
//! | [`substituicao`] | Cancelamento por substituição de NFC-e (110112) via [`SubstituicaoBuilder`] |
//! | [`carta_correcao`] | Carta de Correção Eletrônica (110110) via [`CartaCorrecaoBuilder`] |
//! | [`inutilizacao`] | Inutilização de faixa de numeração via [`InutilizacaoBuilder`] |
//! | [`consulta_situacao`] | Situação da NF-e/NFC-e pela chave via [`ConsultaSituacaoBuilder`] |
//! | [`danfe`] | Geração de DANFE em PDF via [`DanfeBuilder`] |
//! | [`escpos`] | Impressão ESC/POS via [`EscPosBuilder`] e [`EscPosNFCeBuilder`] |
//! | [`distribuicao`] | Distribuição de DF-e (Ambiente Nacional) |
//! | [`status`] | Status do webservice SEFAZ (NF-e 55 / NFC-e 65) via [`NFeService`] |
//! | [`manifestacao`] | Manifestação do destinatário |
//! | [`xml_extractor`] | Extração de campos de XML autorizado |
//! | [`tipos`] | Structs e enums de domínio (`Icms`, `Det`, `Ide`, …) |
//! | `nfse` | NFS-e Nacional (feature `nfse`): DPS, assinatura, envio e consulta na SEFIN |
//!
//! ## Plataformas
//!
//! | Sistema | Certificado e assinatura | TLS com a SEFAZ |
//! |---|---|---|
//! | Windows | CryptoAPI do sistema (sem OpenSSL) | SChannel |
//! | Linux | OpenSSL do sistema + provider `legacy` (PFX ICP-Brasil em RC2/3DES) | OpenSSL + cadeia ICP-Brasil embutida |
//! | macOS | não testado | não testado |
//!
//! A API é a mesma nos dois sistemas suportados, e o resultado também (a assinatura sai idêntica
//! byte a byte). No Linux, compile com `pkg-config` e `libssl-dev`. A feature `validacao-xsd`
//! também requer `libxml2-dev`. Como cada parte
//! funciona e por quê:
//! [Certificado digital e plataformas](https://github.com/GustavoOta/dfe/blob/main/docs/certificado-e-plataformas.md).
//!
//! ## Exemplo — Emissão de NF-e
//!
//! ```no_run
//! # #[cfg(feature = "emissao")]
//! # mod exemplo {
//! use dfe::{NFeBuilder, DfeError};
//! use dfe::tipos::{Det, Emit, Icms, Ide, Pag, Pis, Cofins, Total, Transp};
//!
//! # async fn example() -> Result<(), DfeError> {
//! let resp = NFeBuilder::new()
//!     .cert("./cert.pfx", "senha")
//!     .ide(Ide { c_uf: 35, mod_: 55, serie: 1, n_nf: 1, tp_amb: 2, ..Default::default() })
//!     .emitente(Emit { cnpj: Some("11111111111111".into()), ..Default::default() })
//!     .itens(vec![Det {
//!         c_prod: "001".into(), x_prod: "PRODUTO".into(), ncm: "22030000".into(),
//!         cfop: 5102, q_com: 1.0, v_un_com: 10.0, v_prod: 10.0,
//!         icms: Icms::sn102(0, "400"),
//!         pis: Pis::Nt { cst: "07".into() },
//!         cofins: Cofins::Nt { cst: "07".into() },
//!         ..Default::default()
//!     }])
//!     .total(Total::default())
//!     .transporte(Transp::default())
//!     .pagamento(Pag::default())
//!     .emitir()
//!     .await?;
//!
//! println!("cStat: {}", resp.protocolo.inf_prot.c_stat);
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Tratamento de erros
//!
//! Todas as funções públicas retornam `Result<T, `[`DfeError`]`>`.
//!
//! ```no_run
//! use dfe::DfeError;
//!
//! # fn example(result: Result<(), DfeError>) {
//! match result {
//!     Ok(_)                         => {}
//!     Err(DfeError::Certificado(m)) => eprintln!("Problema no .pfx: {m}"),
//!     Err(DfeError::Validacao(m))   => eprintln!("Dado inválido: {m}"),
//!     Err(DfeError::Webservice(m))  => eprintln!("Falha SEFAZ: {m}"),
//!     Err(e)                        => eprintln!("Erro: {e}"),
//! }
//! # }
//! ```
pub mod arredondamento;
pub mod autorizacao;
pub mod cancelar;
pub mod carta_correcao;
pub mod consulta_situacao;
#[cfg(feature = "danfe")]
pub mod danfe;
#[cfg(feature = "distribuicao")]
pub mod distribuicao;
#[cfg(feature = "emissao")]
pub mod emissao;
pub mod error;
pub mod inutilizacao;
#[cfg(feature = "escpos-base")]
pub mod escpos;
#[cfg(feature = "nfse")]
pub mod nfse;
#[cfg(feature = "distribuicao")]
pub mod manifestacao;
pub mod status;
pub mod substituicao;
pub mod tipos;
pub mod xml_extractor;

/// Compila os exemplos em Rust do `README.md` (vitrine da crate) a cada `cargo test --features nfse`.
#[cfg(all(doctest, feature = "emissao", feature = "danfe", feature = "nfse"))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

mod interno;

pub use arredondamento::{arred, arred2, arred_decimal, fmt_dec, fmt_dec_ate};
pub use autorizacao::{cstat_autorizado, cstat_autorizado_str, CSTAT_AUTORIZADO};
#[cfg(feature = "emissao")]
pub use emissao::Alerta as AlertaAutorizacao;
pub use cancelar::CancelarBuilder;
pub use consulta_situacao::ConsultaSituacaoBuilder;
#[cfg(feature = "danfe")]
pub use danfe::DanfeBuilder;
#[cfg(feature = "escpos-base")]
pub use escpos::EscPosBuilder;
#[cfg(feature = "escpos")]
pub use escpos::EscPosNFCeBuilder;
#[cfg(feature = "escpos")]
pub use escpos::EscPosDanfeNFeBuilder;
pub use interno::cert::CertInfo;
pub use interno::cnpj_cpf::{format_cnpj, sanitize_cnpj, validate_cnpj, validate_cpf};
#[cfg(feature = "validacao-xsd")]
pub use interno::validation::is_xml_valid;
#[cfg(feature = "distribuicao")]
pub use manifestacao::ManifestacaoBuilder;
pub use substituicao::SubstituicaoBuilder;
pub use carta_correcao::CartaCorrecaoBuilder;
#[cfg(feature = "emissao")]
pub use emissao::NFeBuilder;
#[cfg(feature = "emissao")]
pub use emissao::referencia_cupom::{
    chave_e_de_cupom, finalidade_sujeita_a_vedacao, vedacao_referencia_cupom_ativa,
    MSG_REFERENCIA_CUPOM_VEDADA, VEDACAO_REFERENCIA_CUPOM_PRODUCAO,
};
pub use inutilizacao::InutilizacaoBuilder;
#[cfg(feature = "emissao")]
pub use emissao::{transmitir_xml_assinado, NFeAssinada, Response as EmissaoResponse};
pub use error::DfeError;
pub use status::NFeService;
pub use status::NFeServiceResponse;
pub use xml_extractor::{XmlExtractor, XmlExtractorSignature};
