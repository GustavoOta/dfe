//! # dfe
//!
//! Crate Rust para integração com os webservices da **SEFAZ** brasileira.
//! Emite, cancela e distribui NF-e e NFC-e; gera DANFE em PDF e imprime via ESC/POS.
//!
//! ## Início rápido
//!
//! ```toml
//! [dependencies]
//! dfe = "0.5.7"
//! ```
//!
//! ## Funcionalidades
//!
//! | Módulo | Responsabilidade |
//! |---|---|
//! | [`emissao`] | Emissão de NF-e e NFC-e via [`NFeBuilder`] |
//! | [`cancelar`] | Cancelamento via [`CancelarBuilder`] |
//! | [`substituicao`] | Cancelamento por substituição de NFC-e (110112) via [`SubstituicaoBuilder`] |
//! | [`carta_correcao`] | Carta de Correção Eletrônica (110110) via [`CartaCorrecaoBuilder`] |
//! | [`danfe`] | Geração de DANFE em PDF via [`DanfeBuilder`] |
//! | [`escpos`] | Impressão ESC/POS via [`EscPosBuilder`] e [`EscPosNFCeBuilder`] |
//! | [`distribuicao`] | Distribuição de DF-e (Ambiente Nacional) |
//! | [`status`] | Status do webservice SEFAZ via [`NFeService`] |
//! | [`manifestacao`] | Manifestação do destinatário |
//! | [`xml_extractor`] | Extração de campos de XML autorizado |
//! | [`tipos`] | Structs e enums de domínio (`Icms`, `Det`, `Ide`, …) |
//!
//! ## Exemplo — Emissão de NF-e
//!
//! ```no_run
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
pub mod cancelar;
pub mod carta_correcao;
pub mod consulta_situacao;
#[cfg(feature = "danfe")]
pub mod danfe;
#[cfg(feature = "distribuicao")]
pub mod distribuicao;
pub mod emissao;
pub mod error;
#[cfg(feature = "escpos")]
pub mod escpos;
#[cfg(feature = "distribuicao")]
pub mod manifestacao;
pub mod status;
pub mod substituicao;
pub mod tipos;
pub mod xml_extractor;

mod interno;

pub use cancelar::CancelarBuilder;
pub use consulta_situacao::ConsultaSituacaoBuilder;
#[cfg(feature = "danfe")]
pub use danfe::DanfeBuilder;
#[cfg(feature = "escpos")]
pub use escpos::EscPosBuilder;
#[cfg(feature = "escpos")]
pub use escpos::EscPosNFCeBuilder;
pub use interno::cert::CertInfo;
pub use interno::cnpj_cpf::{format_cnpj, sanitize_cnpj, validate_cnpj, validate_cpf};
pub use interno::validation::is_xml_valid;
#[cfg(feature = "distribuicao")]
pub use manifestacao::ManifestacaoBuilder;
pub use substituicao::SubstituicaoBuilder;
pub use carta_correcao::CartaCorrecaoBuilder;
pub use emissao::NFeBuilder;
pub use emissao::Response as EmissaoResponse;
pub use emissao::transmitir_xml_assinado;
pub use error::DfeError;
pub use status::NFeService;
pub use status::NFeServiceResponse;
pub use xml_extractor::{XmlExtractor, XmlExtractorSignature};
