//! Structs de emissão: entrada interna de montagem, XML assinado intermediário e os tipos de
//! resposta públicos (`Response`/`InfProt`/`TagInfProt`). Separado do `mod.rs` na fase A7 (§2.1).

use crate::tipos::{Dest, Det, Emit, Entrega, Ide, InfAdic, Pag, Total, Transp};
use rust_decimal::Decimal;

// Struct interna de montagem — não exposta como API pública.
pub(super) struct NFeInterno {
    pub cert_path: String,
    pub cert_pass: String,
    pub id_csc: Option<String>,
    pub csc: Option<String>,
    pub ide: Ide,
    pub emit: Emit,
    pub dest: Option<Dest>,
    pub det: Vec<Det>,
    pub total: Total,
    pub transp: Transp,
    pub pag: Pag,
    pub inf_adic: Option<InfAdic>,
    pub active_ibs_cbs: Option<String>,
    pub desconto_rateio: Option<Decimal>,
    pub frete_rateio: Option<Decimal>,
    pub entrega: Option<Entrega>,
    pub referencias: Vec<String>,
}

/// Resposta da emissão de NF-e ou NFC-e retornada por [`super::NFeBuilder::emitir`].
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Response {
    /// Protocolo de autorização da SEFAZ.
    pub protocolo: TagInfProt,
    /// XML `nfeProc` autorizado — deve ser persistido em disco.
    pub xml: String,
    /// Envelope SOAP efetivamente enviado à SEFAZ (para debug/auditoria pelo consumidor).
    /// Substitui o antigo dump `./nfe_request_envelope.xml` no CWD (ver A1 do refactor).
    #[serde(default)]
    pub send_xml: String,
    /// Corpo cru da resposta da SEFAZ (para debug/auditoria). Substitui `./nfe_response.xml`.
    #[serde(default)]
    pub receive_xml: String,
}

/// Dados do protocolo de autorização (`<infProt>`).
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct InfProt {
    /// Ambiente: `1` = Produção · `2` = Homologação.
    #[serde(rename = "tpAmb")]   pub tp_amb: i32,
    /// Versão do aplicativo da SEFAZ.
    #[serde(rename = "verAplic")] pub ver_aplic: String,
    /// Chave de acesso da NF-e (44 dígitos).
    #[serde(rename = "chNFe")]   pub ch_nfe: String,
    /// Data e hora do recebimento pela SEFAZ (ISO 8601).
    #[serde(rename = "dhRecbto")] pub dh_recbto: String,
    /// Número do protocolo de autorização. Presente somente quando `c_stat == 100`.
    #[serde(rename = "nProt", skip_serializing_if = "Option::is_none")] pub n_prot: Option<String>,
    /// Digest SHA-1 do XML assinado (base64).
    #[serde(rename = "digVal", skip_serializing_if = "Option::is_none")] pub dig_val: Option<String>,
    /// Código de status da SEFAZ. `100` = autorizado.
    #[serde(rename = "cStat")]   pub c_stat: i32,
    /// Descrição do status retornado pela SEFAZ.
    #[serde(rename = "xMotivo")] pub x_motivo: String,
}

/// Wrapper XML em torno de [`InfProt`] (`<protNFe><infProt>…`).
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TagInfProt {
    /// Dados do protocolo.
    #[serde(rename = "infProt")]
    pub inf_prot: InfProt,
}

// XML assinado e validado + metadados necessários para o envio SEFAZ.
pub(super) struct SignedNfe {
    pub nfe_xml: String,
    pub validated_xml: String,
    pub cert_path: String,
    pub cert_pass: String,
    pub ide_mod: u32,
    pub ide_tp_amb: u8,
    pub ide_c_uf: u16,
}
