//! NFS-e Nacional (DPS v1.01), com a feature `nfse`.
//!
//! [`NfseBuilder`] monta a DPS de uma prestação de serviço nacional sem retenção de ISSQN,
//! emitida por MEI ou optante do Simples Nacional. [`NfseClient`] assina a DPS (XMLDSig
//! RSA-SHA1, o mesmo perfil da NF-e), confere no XSD do ambiente, transmite à SEFIN Nacional
//! (`POST /nfse`) e consulta pela DPS (`GET /dps/{id}`) e pela chave (`GET /nfse/{chave}`).

mod assinatura;
mod builder;
mod cliente;
mod xsd;

pub use assinatura::DpsAssinada;
pub use builder::{
    Ambiente, Documento, DpsGerada, IdentificacaoDps, NfseBuilder, Prestador, RegimeTributario,
    Servico, Tomador, TotalTributos, Tributacao,
};
pub use cliente::{
    ConsultaDps, ErroEnvio, Mensagem, NfseAutorizada, NfseClient, NfseClientBuilder,
    NfseConsultada, Rejeicao, ResultadoEmissao,
};

#[cfg(test)]
mod tests;
