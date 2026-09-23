//! Inutilização de faixa de numeração (`inutNFe`, serviço `NFeInutilizacao4`).
//!
//! Declara à SEFAZ que uma faixa de números de NF-e/NFC-e **não foi e não será usada**. É o
//! desfecho para um número que ficou "pulado" — o caso que motivou este módulo é a numeração
//! queimada por uma emissão que falhou na comunicação e teve a venda concluída em contingência
//! com o número seguinte: se a consulta de situação mostrar que aquela chave nunca chegou à
//! SEFAZ (`ConsultaSituacaoBuilder`), o número fica inutilizado aqui, fechando o intervalo.
//!
//! Só `cStat 102` ("Inutilização de número homologado") é sucesso. A recusa mais importante é a
//! faixa já ter sido usada por uma nota autorizada: nesse caso o caminho não é inutilizar, é
//! cancelar a nota (ou cancelá-la por substituição).

mod parser;
mod xml;

use crate::error::{DfeError, Result};
use crate::interno::cert::{DigestValue, RawPubKey, Sign};
use crate::interno::cleaner::Strings;
use crate::interno::cnpj_cpf::sanitize_cnpj;
use crate::interno::dates::get_current_year;
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::interno::ws::nfe_inutilizacao;
use crate::tipos::inutilizacao::Response;

/// Builder fluente da inutilização de numeração.
///
/// # Exemplo
/// ```no_run
/// use dfe::InutilizacaoBuilder;
/// # async fn ex() -> Result<(), dfe::DfeError> {
/// let resp = InutilizacaoBuilder::new()
///     .cert("./cert.pfx", "senha")
///     .tp_amb(2)
///     .uf("SP")
///     .cnpj("11.222.333/0001-81")
///     .mod_(65)
///     .serie(1)
///     .faixa(325, 325)
///     .justificativa("Numeracao pulada por falha de comunicacao na emissao")
///     .send()
///     .await?;
/// assert_eq!(resp.response.c_stat, "102");
/// # Ok(()) }
/// ```
pub struct InutilizacaoBuilder {
    cert_path: Option<String>,
    cert_pass: Option<String>,
    tp_amb: Option<u8>,
    uf: Option<String>,
    cnpj: Option<String>,
    mod_: Option<u32>,
    serie: Option<u32>,
    n_ini: Option<u64>,
    n_fin: Option<u64>,
    ano: Option<String>,
    justificativa: Option<String>,
}

impl InutilizacaoBuilder {
    pub fn new() -> Self {
        Self {
            cert_path: None,
            cert_pass: None,
            tp_amb: None,
            uf: None,
            cnpj: None,
            mod_: None,
            serie: None,
            n_ini: None,
            n_fin: None,
            ano: None,
            justificativa: None,
        }
    }

    pub fn cert(mut self, path: &str, pass: &str) -> Self {
        self.cert_path = Some(path.to_string());
        self.cert_pass = Some(pass.to_string());
        self
    }

    /// 1 = Produção | 2 = Homologação
    pub fn tp_amb(mut self, v: u8) -> Self {
        self.tp_amb = Some(v);
        self
    }

    /// Sigla da UF do emitente (ex.: `"SP"`) — decide o webservice.
    pub fn uf(mut self, v: &str) -> Self {
        self.uf = Some(v.to_string());
        self
    }

    /// CNPJ do emitente (com ou sem máscara; aceita CNPJ alfanumérico).
    pub fn cnpj(mut self, v: &str) -> Self {
        self.cnpj = Some(v.to_string());
        self
    }

    /// Modelo: 55 = NF-e | 65 = NFC-e
    pub fn mod_(mut self, v: u32) -> Self {
        self.mod_ = Some(v);
        self
    }

    pub fn serie(mut self, v: u32) -> Self {
        self.serie = Some(v);
        self
    }

    /// Faixa a inutilizar. Um número só = `faixa(n, n)`.
    pub fn faixa(mut self, n_ini: u64, n_fin: u64) -> Self {
        self.n_ini = Some(n_ini);
        self.n_fin = Some(n_fin);
        self
    }

    /// Ano da numeração, 2 dígitos (`"26"`). Padrão: ano corrente.
    pub fn ano(mut self, v: &str) -> Self {
        self.ano = Some(v.to_string());
        self
    }

    /// Justificativa (15 a 255 caracteres).
    pub fn justificativa(mut self, v: &str) -> Self {
        self.justificativa = Some(v.to_string());
        self
    }

    pub async fn send(self) -> Result<Response> {
        let cert_path = self
            .cert_path
            .ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass = self
            .cert_pass
            .ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let tp_amb = self
            .tp_amb
            .ok_or_else(|| DfeError::Configuracao("tp_amb não informado".to_string()))?;
        let uf = self
            .uf
            .ok_or_else(|| DfeError::Validacao("uf não informada".to_string()))?;
        let cnpj = self
            .cnpj
            .ok_or_else(|| DfeError::Validacao("cnpj não informado".to_string()))?;
        let mod_ = self
            .mod_
            .ok_or_else(|| DfeError::Validacao("mod_ não informado".to_string()))?;
        let serie = self
            .serie
            .ok_or_else(|| DfeError::Validacao("serie não informada".to_string()))?;
        let n_ini = self
            .n_ini
            .ok_or_else(|| DfeError::Validacao("faixa não informada".to_string()))?;
        let n_fin = self
            .n_fin
            .ok_or_else(|| DfeError::Validacao("faixa não informada".to_string()))?;
        let justificativa = self
            .justificativa
            .ok_or_else(|| DfeError::Validacao("justificativa não informada".to_string()))?;

        // Tudo abaixo roda antes de qualquer I/O (certificado/rede) — erro de preenchimento não
        // consome uma ida à SEFAZ (mesmo padrão de `substituicao`/`consulta_situacao`).
        let uf = uf.trim().to_uppercase();
        let c_uf = crate::interno::uf::codigo_por_sigla(&uf)?;

        if tp_amb != 1 && tp_amb != 2 {
            return Err(DfeError::Validacao(
                "tp_amb deve ser 1 (producao) ou 2 (homologacao)".to_string(),
            ));
        }
        if mod_ != 55 && mod_ != 65 {
            return Err(DfeError::Validacao(
                "mod_ deve ser 55 (NF-e) ou 65 (NFC-e)".to_string(),
            ));
        }

        let cnpj = sanitize_cnpj(&cnpj);
        if cnpj.len() != 14 {
            return Err(DfeError::Validacao(
                "cnpj do emitente deve ter 14 posições".to_string(),
            ));
        }
        if serie > 999 {
            return Err(DfeError::Validacao(
                "serie deve estar entre 0 e 999".to_string(),
            ));
        }
        if n_ini == 0 || n_fin == 0 {
            return Err(DfeError::Validacao(
                "faixa de numeração deve começar em 1".to_string(),
            ));
        }
        if n_fin < n_ini {
            return Err(DfeError::Validacao(
                "número final da faixa não pode ser menor que o inicial".to_string(),
            ));
        }
        if n_fin > 999_999_999 {
            return Err(DfeError::Validacao(
                "número da faixa excede 9 dígitos".to_string(),
            ));
        }

        let justificativa = justificativa.trim().to_string();
        let len = justificativa.chars().count();
        if !(15..=255).contains(&len) {
            return Err(DfeError::Validacao(
                "justificativa deve ter entre 15 e 255 caracteres".to_string(),
            ));
        }

        let ano = match self.ano {
            Some(a) => {
                let a = a.trim().to_string();
                if a.len() != 2 || !a.chars().all(|c| c.is_ascii_digit()) {
                    return Err(DfeError::Validacao(
                        "ano deve ter 2 dígitos (ex.: \"26\")".to_string(),
                    ));
                }
                a
            }
            None => get_current_year(2),
        };

        inutilizar(
            cert_path,
            cert_pass,
            xml::Pedido {
                tp_amb,
                c_uf,
                ano: &ano,
                cnpj: &cnpj,
                mod_,
                serie,
                n_ini,
                n_fin,
                justificativa: &justificativa,
            },
            &uf,
        )
        .await
    }
}

impl Default for InutilizacaoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

async fn inutilizar(
    cert_path: String,
    cert_pass: String,
    pedido: xml::Pedido<'_>,
    uf: &str,
) -> Result<Response> {
    let url = nfe_inutilizacao(pedido.tp_amb, uf, pedido.mod_, false)?;

    let id = xml::inf_inut_id(&pedido);
    let inf_inut = Strings::clear_xml_string(&xml::inf_inut_xml(&pedido)?);

    let digest = DigestValue::sha1(&inf_inut)?;
    let signed_info = crate::interno::assinatura::signed_info_xml(&format!("#{id}"), &digest);
    let signature_base64 = Sign::xml_string(&signed_info, &cert_path, &cert_pass).await?;
    let x509_cert = RawPubKey::get_from_file(&cert_path, &cert_pass).await?;
    let signature =
        crate::interno::assinatura::signature_xml(&signed_info, &signature_base64, &x509_cert);

    let envelope = Strings::clear_xml_string(&xml::inut_nfe_envelope(&inf_inut, &signature));
    let send_xml = envelope.clone();

    let receive_xml = MtlsTransport::new(&cert_path, &cert_pass)
        .send_soap(url, envelope)
        .await?;

    let parsed = parser::parse_ret_inut_nfe(&receive_xml)?;
    Ok(Response {
        response: parsed,
        send_xml,
        receive_xml,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builder_valido() -> InutilizacaoBuilder {
        InutilizacaoBuilder::new()
            .cert("inexistente.pfx", "x")
            .tp_amb(2)
            .uf("SP")
            .cnpj("11222333000181")
            .mod_(65)
            .serie(1)
            .faixa(325, 325)
            .justificativa("Numeracao pulada por falha de comunicacao na emissao")
    }

    // Todas as validações abaixo têm de falhar ANTES de tocar no certificado (o .pfx do builder
    // não existe): é o que prova o fail-fast.
    #[tokio::test]
    async fn rejeita_cert_ausente() {
        let err = InutilizacaoBuilder::new()
            .tp_amb(2)
            .uf("SP")
            .cnpj("11222333000181")
            .mod_(65)
            .serie(1)
            .faixa(1, 1)
            .justificativa("Justificativa suficientemente longa")
            .send()
            .await
            .unwrap_err();
        assert!(matches!(err, DfeError::Configuracao(_)));
    }

    #[tokio::test]
    async fn rejeita_uf_invalida() {
        let err = builder_valido().uf("XX").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_modelo_invalido() {
        let err = builder_valido().mod_(57).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_faixa_invertida() {
        let err = builder_valido().faixa(330, 325).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_faixa_comecando_em_zero() {
        let err = builder_valido().faixa(0, 0).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_justificativa_curta() {
        let err = builder_valido()
            .justificativa("Curta demais")
            .send()
            .await
            .unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_cnpj_incompleto() {
        let err = builder_valido().cnpj("1122233300").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_ano_fora_de_2_digitos() {
        let err = builder_valido().ano("2026").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_tp_amb_invalido() {
        let err = builder_valido().tp_amb(9).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    // Endpoint tem de existir para as 27 UFs, nos dois ambientes e nos dois modelos — sem isso a
    // inutilização só funcionaria em parte do parque.
    #[test]
    fn todas_as_ufs_tem_endpoint_de_inutilizacao() {
        for (_, sigla) in crate::interno::uf::UFS {
            for amb in [1u8, 2] {
                for modelo in [55u32, 65] {
                    nfe_inutilizacao(amb, sigla, modelo, false).unwrap_or_else(|e| {
                        panic!("sem endpoint de inutilização p/ {sigla} amb {amb} mod {modelo}: {e:?}")
                    });
                }
            }
        }
    }
}
