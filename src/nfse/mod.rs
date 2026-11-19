//! Construção de DPS v1.01 para a NFS-e Nacional, com a feature `nfse`.
//!
//! O builder gera uma DPS regular de serviço nacional, sem retenção de ISSQN,
//! emitida pelo prestador. O XML gerado está pronto para a etapa de assinatura.

mod builder;

pub use builder::{
    Ambiente, Documento, DpsGerada, IdentificacaoDps, NfseBuilder, Prestador, RegimeTributario,
    Servico, Tomador, TotalTributos, Tributacao,
};
