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
pub mod danfe;
pub mod distribuicao;
pub mod emissao;
pub mod error;
pub mod escpos;
pub mod manifestacao;
pub mod status;
pub mod tipos;
pub mod xml_extractor;

mod interno;

pub use cancelar::CancelarBuilder;
pub use danfe::DanfeBuilder;
pub use escpos::EscPosBuilder;
pub use escpos::EscPosNFCeBuilder;
pub use interno::cnpj_cpf::{validate_cnpj, validate_cpf};
pub use interno::validation::is_xml_valid;
pub use emissao::NFeBuilder;
pub use emissao::Response as EmissaoResponse;
pub use error::DfeError;
pub use status::NFeService;
pub use status::NFeServiceResponse;
pub use xml_extractor::{XmlExtractor, XmlExtractorSignature};
