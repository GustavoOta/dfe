//! Carta de Correção Eletrônica (CC-e) — evento **tpEvento 110110** (MOC / `CCe_v1.00.xsd`).
//!
//! Corrige dados de uma NF-e/NFC-e **não** relacionados a valores, impostos ou destinatário
//! (regras do Convênio S/N de 15/12/1970, art. 7º, §1º-A). Vale para NF-e **e** NFC-e.
//!
//! Builder fino (§2.1) sobre a espinha unificada de eventos: `<infEvento>` via
//! [`crate::interno::evento`], assinatura via [`crate::interno::assinatura`], envelope via
//! [`crate::interno::evento::env_evento_xml`] e envio via [`crate::interno::transporte`] —
//! molde igual ao `substituicao`, diferindo no `detEvento`, no `tpEvento` e por aceitar
//! `nSeqEvento` **acumulativo**.
//!
//! # Campos do `detEvento` (ordem do schema `CCe_v1.00.xsd`)
//! `descEvento` = "Carta de Correcao" · `xCorrecao` (o texto da correção, 15–1000) · `xCondUso`
//! (texto **fixo** da legislação, embutido — nunca informado pelo usuário).
//!
//! # `nSeqEvento` acumulativo
//! Cada CC-e da mesma chave incrementa o `nSeqEvento` (≥ 1). A legislação exige que **cada CC-e
//! contenha todas as correções anteriores** (a última substitui as demais) — cabe ao chamador
//! montar o texto acumulado e informar o `nSeqEvento` correto via [`CartaCorrecaoBuilder::n_seq_evento`].
//!
//! > **Roteamento (A6):** endpoint resolvido pela UF real da chave (`interno::uf` +
//! > `webservices.json`), cobrindo as 27 UFs (autorizadores próprios + SVAN/SVRS).

use crate::error::{DfeError, Result};
use crate::interno::cert::{DigestValue, RawPubKey, Sign};
use crate::interno::chave_acesso::ChaveAcesso;
use crate::interno::chave_acesso_props::ExtractComposition;
use crate::interno::cleaner::Strings;
use crate::interno::transporte::{MtlsTransport, SoapTransport};
use crate::interno::ws::nfe_recepcao_evento;
use crate::tipos::cancelar::{InfEvento, Response};

const TP_EVENTO: &str = "110110";
const DESC_EVENTO: &str = "Carta de Correcao"; // sem acento, conforme a NT
const VER_EVENTO: &str = "1.00";

// Texto fixo da condição de uso (Convênio S/N de 15/12/1970, art. 7º, §1º-A). Sem acentos,
// conforme convenção da SEFAZ. É obrigatório e imutável — o usuário nunca o informa.
const X_COND_USO: &str = "A Carta de Correcao e disciplinada pelo paragrafo 1o-A do art. 7o do Convenio S/N, de 15 de dezembro de 1970 e pode ser utilizada para regularizacao de erro ocorrido na emissao de documento fiscal, desde que o erro nao esteja relacionado com: I - as variaveis que determinam o valor do imposto tais como: base de calculo, aliquota, diferenca de preco, quantidade, valor da operacao ou da prestacao; II - a correcao de dados cadastrais que implique mudanca do remetente ou do destinatario; III - a data de emissao ou de saida.";

// ─── Builder público ──────────────────────────────────────────────────────────

/// Carta de Correção Eletrônica (evento 110110). Retorna
/// [`crate::tipos::cancelar::Response`] — o `retEvento` tem a mesma forma do cancelamento.
///
/// # Exemplo
/// ```no_run
/// use dfe::CartaCorrecaoBuilder;
/// # async fn ex() -> Result<(), dfe::DfeError> {
/// let resp = CartaCorrecaoBuilder::new()
///     .cert("./cert.pfx", "senha")
///     .tp_amb(2)
///     .chave("35...")                 // NF-e/NFC-e a corrigir (44 dígitos)
///     .correcao("Onde se le X, leia-se Y no campo de observacoes")
///     .n_seq_evento(1)                // acumulativo; 1 na primeira CC-e
///     .send()
///     .await?;
/// println!("cStat: {}", resp.response.c_stat);
/// # Ok(()) }
/// ```
pub struct CartaCorrecaoBuilder {
    cert_path:     Option<String>,
    cert_pass:     Option<String>,
    tp_amb:        Option<u8>,
    chave:         Option<String>,
    correcao:      Option<String>,
    n_seq_evento:  Option<u32>,
}

impl CartaCorrecaoBuilder {
    pub fn new() -> Self {
        Self {
            cert_path: None, cert_pass: None, tp_amb: None,
            chave: None, correcao: None, n_seq_evento: None,
        }
    }

    /// Certificado A1 (`.pfx`) e senha.
    pub fn cert(mut self, path: &str, pass: &str) -> Self {
        self.cert_path = Some(path.to_string());
        self.cert_pass = Some(pass.to_string());
        self
    }

    /// 1 = Produção | 2 = Homologação
    pub fn tp_amb(mut self, v: u8) -> Self { self.tp_amb = Some(v); self }

    /// Chave de acesso (44 dígitos) da NF-e/NFC-e a corrigir.
    pub fn chave(mut self, v: &str) -> Self { self.chave = Some(v.to_string()); self }

    /// Texto da correção (`xCorrecao`, 15–1000 caracteres após normalização).
    pub fn correcao(mut self, v: &str) -> Self { self.correcao = Some(v.to_string()); self }

    /// Número sequencial do evento (`nSeqEvento`, ≥ 1). Padrão 1. Acumulativo entre CC-e da
    /// mesma chave — informe o próximo número a cada nova carta.
    pub fn n_seq_evento(mut self, v: u32) -> Self { self.n_seq_evento = Some(v); self }

    pub async fn send(self) -> Result<Response> {
        let cert_path = self.cert_path.ok_or_else(|| DfeError::Configuracao("cert_path não informado".to_string()))?;
        let cert_pass = self.cert_pass.ok_or_else(|| DfeError::Configuracao("cert_pass não informado".to_string()))?;
        let tp_amb    = self.tp_amb.ok_or_else(|| DfeError::Configuracao("tp_amb não informado".to_string()))?;
        let chave     = self.chave.ok_or_else(|| DfeError::Validacao("chave não informada".to_string()))?;
        let correcao  = self.correcao.ok_or_else(|| DfeError::Validacao("correcao não informada".to_string()))?;
        let n_seq_evento = self.n_seq_evento.unwrap_or(1);

        // ── Validações (antes de qualquer I/O) ──
        if chave.len() != 44 {
            return Err(DfeError::Validacao("chave deve ter 44 dígitos".to_string()));
        }
        if n_seq_evento < 1 {
            return Err(DfeError::Validacao("n_seq_evento deve ser >= 1".to_string()));
        }
        // Quebras de linha → espaço (o clear_xml_string, como o sped-nfe, não preserva \n; assim
        // evitamos juntar palavras). Single-line, seguro para a assinatura.
        let correcao = normalize_correcao(&correcao);
        let corr_len = correcao.chars().count();
        // MOC: xCorrecao 15–1000.
        if corr_len < 15 {
            return Err(DfeError::Validacao("correcao deve ter no mínimo 15 caracteres".to_string()));
        }
        if corr_len > 1000 {
            return Err(DfeError::Validacao("correcao deve ter no máximo 1000 caracteres".to_string()));
        }

        let comp = ChaveAcesso::extract_composition(&chave)?;

        carta_correcao(cert_path, cert_pass, tp_amb, comp, chave, correcao, n_seq_evento).await
    }
}

impl Default for CartaCorrecaoBuilder {
    fn default() -> Self { Self::new() }
}

// ─── Lógica interna ───────────────────────────────────────────────────────────

fn normalize_correcao(s: &str) -> String {
    // Substitui qualquer whitespace (inclui \n, \r, \t) por espaço e colapsa sequências,
    // aparando as pontas. Determinístico e single-line.
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

async fn carta_correcao(
    cert_path: String, cert_pass: String,
    tp_amb: u8, comp: ExtractComposition,
    chave: String, correcao: String, n_seq_evento: u32,
) -> Result<Response> {
    let inf_evento_xml = inf_evento_xml(tp_amb, &comp, &chave, &correcao, n_seq_evento)?;
    let inf_evento_xml = Strings::clear_xml_string(&inf_evento_xml);

    let digest_value = DigestValue::sha1(&inf_evento_xml)?;
    let reference_uri = format!("#ID{}{}{:>02}", TP_EVENTO, chave, n_seq_evento);
    let signed_info  = crate::interno::assinatura::signed_info_xml(&reference_uri, &digest_value);

    let signature_base64 = Sign::xml_string(&signed_info, &cert_path, &cert_pass).await?;
    let x509_cert        = RawPubKey::get_from_file(&cert_path, &cert_pass).await?;
    let signature        = crate::interno::assinatura::signature_xml(&signed_info, &signature_base64, &x509_cert);
    let envelope         = crate::interno::evento::env_evento_xml(&inf_evento_xml, &signature)?;
    let envelope         = Strings::clear_xml_string(&envelope);

    // A6: UF real derivada da chave. CC-e vale p/ NF-e e NFC-e → roteia pelo modelo da chave.
    let modelo: u32 = comp.modelo.parse().unwrap_or(55);
    let uf = crate::interno::uf::sigla_por_codigo(&comp.uf_code)?;
    let url = nfe_recepcao_evento(tp_amb, uf, modelo, false)?;

    let send_envelope = envelope.clone();
    let response = MtlsTransport::new(&cert_path, &cert_pass)
        .send_soap(url, envelope)
        .await?;

    let parsed: InfEvento = crate::interno::evento::parse_ret_evento(&response)?;
    Ok(Response { response: parsed, send_xml: send_envelope, receive_xml: response })
}

fn inf_evento_xml(
    tp_amb: u8,
    comp: &ExtractComposition,
    chave: &str,
    correcao: &str,
    n_seq_evento: u32,
) -> Result<String> {
    // detEvento do 110110 (ordem do schema CCe_v1.00.xsd): descEvento + xCorrecao + xCondUso.
    let det_campos: Vec<(&str, &str)> = vec![
        ("xCorrecao", correcao),
        ("xCondUso", X_COND_USO),
    ];
    crate::interno::evento::inf_evento_xml(&crate::interno::evento::InfEvento {
        c_orgao: &comp.uf_code,
        tp_amb,
        cnpj: &comp.doc,
        chave,
        tp_evento: TP_EVENTO,
        n_seq_evento,
        ver_evento: VER_EVENTO,
        desc_evento: DESC_EVENTO,
        det_campos: &det_campos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHAVE: &str = "35000000000000000000550010000000001000000001";

    fn comp() -> ExtractComposition {
        ChaveAcesso::extract_composition(CHAVE).unwrap()
    }

    // Golden do infEvento (nó assinado) do 110110. dhEvento (volátil) redigido. Trava a ordem do
    // detEvento (descEvento/xCorrecao/xCondUso) e o texto FIXO de xCondUso.
    #[test]
    fn golden_inf_evento_110110() {
        let out = inf_evento_xml(2, &comp(), CHAVE, "Onde se le X, leia-se Y no campo de observacoes", 1).unwrap();
        insta::with_settings!({filters => vec![(r"<dhEvento>.*?</dhEvento>", "<dhEvento>[DH]</dhEvento>")]}, {
            insta::assert_snapshot!(out);
        });
    }

    // nSeqEvento acumulativo: com n=3 o Id termina em "03" e <nSeqEvento>3</nSeqEvento>.
    #[test]
    fn golden_inf_evento_110110_nseq_3() {
        let out = inf_evento_xml(2, &comp(), CHAVE, "Correcao numero tres acumulando as anteriores", 3).unwrap();
        insta::with_settings!({filters => vec![(r"<dhEvento>.*?</dhEvento>", "<dhEvento>[DH]</dhEvento>")]}, {
            insta::assert_snapshot!(out);
        });
    }

    #[test]
    fn normaliza_quebras_de_linha() {
        // \n/\r/\t viram espaço único, sem juntar palavras nem sobrar espaços.
        assert_eq!(normalize_correcao("linha1\nlinha2"), "linha1 linha2");
        assert_eq!(normalize_correcao("  a\t b \r\n c  "), "a b c");
    }

    fn builder_valido() -> CartaCorrecaoBuilder {
        CartaCorrecaoBuilder::new()
            .cert("inexistente.pfx", "x")
            .tp_amb(2)
            .chave(CHAVE)
            .correcao("Onde se le X, leia-se Y no campo de observacoes")
    }

    // Validações fail-fast (antes de I/O).
    #[tokio::test]
    async fn rejeita_correcao_curta() {
        let err = builder_valido().correcao("curta").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_correcao_longa() {
        let longa = "a".repeat(1001);
        let err = builder_valido().correcao(&longa).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_chave_tamanho_invalido() {
        let err = builder_valido().chave("35055").send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }

    #[tokio::test]
    async fn rejeita_n_seq_zero() {
        let err = builder_valido().n_seq_evento(0).send().await.unwrap_err();
        assert!(matches!(err, DfeError::Validacao(_)));
    }
}
