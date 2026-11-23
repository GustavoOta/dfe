//! Assinatura e cliente REST da NFS-e, offline: PFX sintético de `tests/fixtures/cert/` e
//! transporte simulado (`cliente::testes::Falso`).

use super::cliente::testes::{falha, http};
use super::cliente::{compactar, descompactar, Falha};
use super::*;
use crate::error::DfeError;
use crate::interno::cert::DigestValue;
use chrono::{DateTime, NaiveDate};
use rust_decimal::Decimal;

const SENHA: &str = "senha-teste-dfe";

fn pfx() -> String {
    format!(
        "{}/tests/fixtures/cert/sintetico_aes.pfx",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn dps(ambiente: Ambiente, descricao: &str) -> DpsGerada {
    NfseBuilder::new()
        .ambiente(ambiente)
        .identificacao(IdentificacaoDps {
            municipio_emissao: "3529906".into(),
            serie: 42,
            numero: 7,
            emitida_em: DateTime::parse_from_rfc3339("2026-11-23T09:30:00-03:00").unwrap(),
            competencia: NaiveDate::from_ymd_opt(2026, 11, 23).unwrap(),
            versao_aplicativo: "dfe-teste".into(),
        })
        .prestador(Prestador {
            documento: Documento::Cnpj("12345678000195".into()),
            inscricao_municipal: None,
            regime: RegimeTributario::SimplesNacional,
        })
        .tomador(Tomador {
            documento: Documento::Cpf("52998224725".into()),
            nome: "Tomador & Cia".into(),
        })
        .servico(Servico {
            municipio_prestacao: "3529906".into(),
            codigo_tributacao_nacional: "010701".into(),
            codigo_tributacao_municipal: None,
            codigo_nbs: Some("115032000".into()),
            descricao: descricao.into(),
            valor: Decimal::new(12345, 2),
        })
        .tributacao(Tributacao {
            total: TotalTributos::PercentualSimplesNacional(Decimal::new(675, 2)),
        })
        .gerar_dps()
        .unwrap()
}

fn cliente(
    ambiente: Ambiente,
    respostas: Vec<std::result::Result<cliente::RespostaHttp, Falha>>,
) -> (NfseClient, std::sync::Arc<cliente::testes::Falso>) {
    NfseClient::com_respostas(ambiente, &pfx(), SENHA, respostas)
}

async fn assinada(ambiente: Ambiente) -> DpsAssinada {
    let (c, _) = cliente(ambiente, vec![]);
    c.assinar(&dps(ambiente, "Suporte técnico")).await.unwrap()
}

fn chave() -> String {
    format!("3529906{}", "1".repeat(43))
}

/// `<infDPS>` com o namespace herdado — o que o C14N da SEFIN digere.
fn inf_canonico(xml: &str) -> String {
    let ini = xml.find("<infDPS ").unwrap();
    let fim = xml.find("</infDPS>").unwrap() + "</infDPS>".len();
    xml[ini..fim].replacen(
        "<infDPS ",
        "<infDPS xmlns=\"http://www.sped.fazenda.gov.br/nfse\" ",
        1,
    )
}

fn tag<'a>(xml: &'a str, nome: &str) -> &'a str {
    let ini = xml.find(&format!("<{nome}>")).unwrap() + nome.len() + 2;
    let fim = xml[ini..].find(&format!("</{nome}>")).unwrap() + ini;
    &xml[ini..fim]
}

// ── Assinatura ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn assina_no_perfil_do_manual_e_valida_no_xsd_dos_dois_ambientes() {
    for ambiente in [Ambiente::Producao, Ambiente::ProducaoRestrita] {
        let (c, _) = cliente(ambiente, vec![]);
        let gerada = dps(ambiente, "Suporte técnico");
        let a = c
            .assinar(&gerada)
            .await
            .unwrap_or_else(|e| panic!("{ambiente:?}: {e}"));
        assert_eq!(a.id(), gerada.id());
        assert_eq!(a.ambiente(), ambiente);

        let xml = a.xml();
        let sig = xml
            .find("<Signature xmlns=\"http://www.w3.org/2000/09/xmldsig#\">")
            .unwrap();
        assert_eq!(
            sig,
            xml.find("</infDPS>").unwrap() + "</infDPS>".len(),
            "Signature logo após o infDPS"
        );
        assert!(xml.ends_with("</Signature></DPS>"));
        for trecho in [
            format!("<Reference URI=\"#{}\">", a.id()),
            "xmldsig#rsa-sha1".into(),
            "xmldsig#sha1".into(),
            "xmldsig#enveloped-signature".into(),
            "REC-xml-c14n-20010315".into(),
        ] {
            assert!(xml.contains(&trecho), "{ambiente:?}: falta {trecho}");
        }
        assert_eq!(
            tag(xml, "DigestValue"),
            DigestValue::sha1(&inf_canonico(xml)).unwrap()
        );
        assert!(
            !xml.contains('\n') && !xml.contains("> <"),
            "sem formatação entre tags"
        );
    }
}

#[tokio::test]
async fn dps_assinada_e_deterministica() {
    // RSA-SHA1 PKCS#1 v1.5 é determinístico: mesmo PFX + mesma DPS = mesmo XML, no CAPI e no
    // OpenSSL. O snapshot trava a montagem; foi conferido à parte com lxml (C14N + SHA-1) e
    // `openssl dgst -sha1 -verify` (ver CHANGELOG).
    let a = assinada(Ambiente::ProducaoRestrita).await;
    insta::assert_snapshot!(a.xml());
}

#[tokio::test]
async fn texto_com_aspas_e_crlf_sai_canonico() {
    let (c, _) = cliente(Ambiente::ProducaoRestrita, vec![]);
    let a = c
        .assinar(&dps(
            Ambiente::ProducaoRestrita,
            "Troca \"rápida\" d'água & <filtro>\r\nlinha 2\rlinha 3",
        ))
        .await
        .unwrap();
    assert!(a.xml().contains(
        "<xDescServ>Troca \"rápida\" d'água &amp; &lt;filtro&gt;\nlinha 2\nlinha 3</xDescServ>"
    ));
    assert!(!a.xml().contains("&quot;") && !a.xml().contains("&apos;") && !a.xml().contains('\r'));
    assert_eq!(
        tag(a.xml(), "DigestValue"),
        DigestValue::sha1(&inf_canonico(a.xml())).unwrap()
    );
}

#[tokio::test]
async fn from_xml_reabre_a_dps_guardada() {
    for ambiente in [Ambiente::Producao, Ambiente::ProducaoRestrita] {
        let a = assinada(ambiente).await;
        let b = DpsAssinada::from_xml(a.xml()).unwrap();
        assert_eq!(
            (b.ambiente(), b.id(), b.xml()),
            (a.ambiente(), a.id(), a.xml())
        );
    }
}

#[tokio::test]
async fn from_xml_recusa_dps_alterada_sem_assinatura_ou_fora_do_padrao() {
    let a = assinada(Ambiente::ProducaoRestrita).await;
    let alterada = a
        .xml()
        .replace("<vServ>123.45</vServ>", "<vServ>123.46</vServ>");
    assert!(
        matches!(DpsAssinada::from_xml(&alterada), Err(DfeError::Assinatura(m)) if m.contains("alterado"))
    );

    let gerada = dps(Ambiente::ProducaoRestrita, "Suporte");
    assert!(
        matches!(DpsAssinada::from_xml(gerada.xml()), Err(DfeError::Assinatura(m)) if m.contains("assinatura ausente"))
    );

    let outra_uri = a
        .xml()
        .replace(&format!("URI=\"#{}\"", a.id()), "URI=\"#outro\"");
    assert!(matches!(
        DpsAssinada::from_xml(&outra_uri),
        Err(DfeError::Assinatura(_))
    ));

    let prefixada = a.xml().replace("<DPS xmlns=", "<n:DPS xmlns:n=");
    assert!(matches!(
        DpsAssinada::from_xml(&prefixada),
        Err(DfeError::Assinatura(_))
    ));
}

#[tokio::test]
async fn recusa_ambiente_trocado_ao_assinar_e_ao_transmitir() {
    let (c, falso) = cliente(Ambiente::Producao, vec![]);
    let erro = c
        .assinar(&dps(Ambiente::ProducaoRestrita, "x"))
        .await
        .unwrap_err();
    assert!(matches!(erro, DfeError::Validacao(m) if m.contains("ProducaoRestrita")));

    let restrita = assinada(Ambiente::ProducaoRestrita).await;
    assert!(matches!(
        c.transmitir(&restrita).await,
        Err(ErroEnvio::Local(DfeError::Validacao(_)))
    ));
    assert!(
        falso.pedidos.lock().unwrap().is_empty(),
        "nada pode ter sido enviado"
    );
}

#[tokio::test]
async fn senha_errada_nao_assina() {
    let c = NfseClient::builder()
        .ambiente(Ambiente::ProducaoRestrita)
        .cert(pfx(), "errada")
        .build()
        .unwrap();
    let erro = c
        .assinar(&dps(Ambiente::ProducaoRestrita, "x"))
        .await
        .unwrap_err();
    assert!(
        matches!(erro, DfeError::Assinatura(_) | DfeError::Certificado(_)),
        "{erro:?}"
    );
}

#[test]
fn builder_do_cliente_exige_ambiente_e_certificado() {
    assert!(matches!(
        NfseClient::builder().cert("a.pfx", "s").build(),
        Err(DfeError::Configuracao(_))
    ));
    assert!(matches!(
        NfseClient::builder()
            .ambiente(Ambiente::ProducaoRestrita)
            .build(),
        Err(DfeError::Configuracao(_))
    ));
    assert!(matches!(
        NfseClient::builder()
            .ambiente(Ambiente::ProducaoRestrita)
            .cert(" ", "s")
            .build(),
        Err(DfeError::Configuracao(_))
    ));
}

// ── Transmissão ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn autorizada_devolve_chave_xml_e_alertas() {
    let a = assinada(Ambiente::ProducaoRestrita).await;
    let nfse = format!(
        "<NFSe xmlns=\"http://www.sped.fazenda.gov.br/nfse\"><chave>{}</chave></NFSe>",
        chave()
    );
    let corpo = serde_json::json!({
        "tipoAmbiente": 2,
        "versaoAplicativo": "SefinNacional_1.0",
        "dataHoraProcessamento": "2026-11-23T09:31:00-03:00",
        "idDps": a.id(),
        "chaveAcesso": chave(),
        "nfseXmlGZipB64": compactar(&nfse).unwrap(),
        "alertas": [{ "codigo": "A0001", "descricao": "Alerta de teste" }]
    });
    let (c, falso) = cliente(
        Ambiente::ProducaoRestrita,
        vec![http(201, &corpo.to_string())],
    );

    let ResultadoEmissao::Autorizada(r) = c.transmitir(&a).await.unwrap() else {
        panic!("esperava autorizada")
    };
    assert_eq!(
        (r.chave_acesso.as_str(), r.id_dps.as_str(), r.xml.as_str()),
        (chave().as_str(), a.id(), nfse.as_str())
    );
    assert_eq!(r.alertas[0].codigo.as_deref(), Some("A0001"));
    assert_eq!(
        r.data_hora_processamento.as_deref(),
        Some("2026-11-23T09:31:00-03:00")
    );

    let pedidos = falso.pedidos.lock().unwrap();
    assert!(pedidos[0].post);
    assert_eq!(
        pedidos[0].url,
        "https://sefin.producaorestrita.nfse.gov.br/SefinNacional/nfse"
    );
    let corpo: serde_json::Value =
        serde_json::from_str(pedidos[0].corpo.as_deref().unwrap()).unwrap();
    assert_eq!(corpo.as_object().unwrap().len(), 1);
    assert_eq!(
        descompactar(corpo["dpsXmlGZipB64"].as_str().unwrap()).unwrap(),
        a.xml()
    );
}

#[tokio::test]
async fn producao_usa_a_url_de_producao() {
    let a = assinada(Ambiente::Producao).await;
    let (c, falso) = cliente(Ambiente::Producao, vec![http(500, "")]);
    let _ = c.transmitir(&a).await;
    assert_eq!(
        falso.pedidos.lock().unwrap()[0].url,
        "https://sefin.nfse.gov.br/SefinNacional/nfse"
    );
}

#[tokio::test]
async fn rejeitada_le_as_mensagens_nas_duas_grafias() {
    let a = assinada(Ambiente::ProducaoRestrita).await;
    let maiuscula = r#"{"idDPS":"X","erros":[{"Codigo":"E0014","Descricao":"DPS já utilizada","Complemento":null}]}"#;
    let minuscula = r#"{"erros":[{"codigo":"E0714","descricao":"Assinatura inválida","complemento":"digest"}]}"#;
    let singular = r#"{"erro":{"Codigo":1234,"Descricao":"Erro"}}"#;
    let (c, _) = cliente(
        Ambiente::ProducaoRestrita,
        vec![
            http(400, maiuscula),
            http(400, minuscula),
            http(422, singular),
        ],
    );

    let ResultadoEmissao::Rejeitada(r) = c.transmitir(&a).await.unwrap() else {
        panic!()
    };
    assert_eq!(r.id_dps, "X");
    assert_eq!(
        r.erros,
        vec![Mensagem {
            codigo: Some("E0014".into()),
            descricao: Some("DPS já utilizada".into()),
            complemento: None,
        }]
    );
    assert_eq!(r.erros[0].to_string(), "E0014 - DPS já utilizada");

    let ResultadoEmissao::Rejeitada(r) = c.transmitir(&a).await.unwrap() else {
        panic!()
    };
    assert_eq!(r.id_dps, a.id(), "sem idDps na resposta, vale o enviado");
    assert_eq!(r.erros[0].complemento.as_deref(), Some("digest"));

    let ResultadoEmissao::Rejeitada(r) = c.transmitir(&a).await.unwrap() else {
        panic!()
    };
    assert_eq!(r.erros[0].codigo.as_deref(), Some("1234"));
}

#[tokio::test]
async fn sem_resposta_conclusiva_e_indeterminado() {
    let a = assinada(Ambiente::ProducaoRestrita).await;
    let (c, _) = cliente(
        Ambiente::ProducaoRestrita,
        vec![
            http(500, r#"{"erros":[{"codigo":"X"}]}"#),
            http(502, "<html>Bad Gateway</html>"),
            http(201, "{}"),
            http(
                201,
                &serde_json::json!({"chaveAcesso": chave(), "nfseXmlGZipB64": "@@"}).to_string(),
            ),
            falha(Falha::Indeterminada("timeout".into())),
        ],
    );
    let mut status = Vec::new();
    for _ in 0..5 {
        match c.transmitir(&a).await {
            Err(ErroEnvio::Indeterminado {
                id_dps,
                status: s,
                motivo,
            }) => {
                assert_eq!(id_dps, a.id());
                status.push((s, motivo));
            }
            outro => panic!("esperava indeterminado, veio {outro:?}"),
        }
    }
    assert_eq!(
        status.iter().map(|(s, _)| *s).collect::<Vec<_>>(),
        vec![Some(500), Some(502), Some(201), Some(201), None]
    );
    assert!(
        status[3].1.contains(&chave()),
        "a chave gerada aparece no motivo: {}",
        status[3].1
    );
}

#[tokio::test]
async fn falha_antes_do_envio_e_local() {
    let a = assinada(Ambiente::ProducaoRestrita).await;
    let (c, _) = cliente(
        Ambiente::ProducaoRestrita,
        vec![falha(Falha::NaoEnviada(DfeError::Webservice(
            "conexão recusada".into(),
        )))],
    );
    assert!(matches!(
        c.transmitir(&a).await,
        Err(ErroEnvio::Local(DfeError::Webservice(_)))
    ));
}

// ── Consultas ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn consulta_dps_encontrada_ausente_e_negada() {
    let a = assinada(Ambiente::ProducaoRestrita).await;
    let (c, falso) = cliente(
        Ambiente::ProducaoRestrita,
        vec![
            http(
                200,
                &serde_json::json!({"idDps": a.id(), "chaveAcesso": chave()}).to_string(),
            ),
            http(404, ""),
            http(
                403,
                r#"{"erros":[{"Codigo":"E9999","Descricao":"Certificado não é ator da DPS"}]}"#,
            ),
        ],
    );
    assert_eq!(
        c.consultar_dps(a.id()).await.unwrap(),
        ConsultaDps::Encontrada {
            chave_acesso: chave()
        }
    );
    assert_eq!(
        c.consultar_dps(a.id()).await.unwrap(),
        ConsultaDps::NaoEncontrada
    );
    let erro = c.consultar_dps(a.id()).await.unwrap_err();
    assert!(
        matches!(&erro, DfeError::Webservice(m) if m.contains("403") && m.contains("E9999")),
        "{erro:?}"
    );

    let pedidos = falso.pedidos.lock().unwrap();
    assert!(!pedidos[0].post);
    assert_eq!(
        pedidos[0].url,
        format!(
            "https://sefin.producaorestrita.nfse.gov.br/SefinNacional/dps/{}",
            a.id()
        )
    );
}

#[tokio::test]
async fn consulta_recusa_id_e_chave_fora_do_formato_sem_enviar() {
    let (c, falso) = cliente(Ambiente::ProducaoRestrita, vec![]);
    for id in ["", "DPS123", "../nfse", &format!("DPS{}/", "1".repeat(41))] {
        assert!(
            matches!(c.consultar_dps(id).await, Err(DfeError::Validacao(_))),
            "{id}"
        );
    }
    for chave in ["", "123", &format!("{}/", "1".repeat(49)), &"a".repeat(50)] {
        assert!(
            matches!(c.consultar_nfse(chave).await, Err(DfeError::Validacao(_))),
            "{chave}"
        );
    }
    assert!(falso.pedidos.lock().unwrap().is_empty());
}

#[tokio::test]
async fn consulta_nfse_devolve_o_xml() {
    let nfse = "<NFSe>ok</NFSe>";
    let (c, falso) = cliente(
        Ambiente::Producao,
        vec![
            http(200, &serde_json::json!({"chaveAcesso": chave(), "nfseXmlGZipB64": compactar(nfse).unwrap()}).to_string()),
            http(404, r#"{"erro":{"codigo":"E404","descricao":"NFS-e não encontrada"}}"#),
        ],
    );
    let r = c.consultar_nfse(&chave()).await.unwrap();
    assert_eq!((r.chave_acesso, r.xml.as_str()), (chave(), nfse));
    assert_eq!(
        falso.pedidos.lock().unwrap()[0].url,
        format!("https://sefin.nfse.gov.br/SefinNacional/nfse/{}", chave())
    );
    assert!(
        matches!(c.consultar_nfse(&chave()).await, Err(DfeError::Webservice(m)) if m.contains("E404"))
    );
}

// ── GZip ─────────────────────────────────────────────────────────────────────

#[test]
fn gzip_ida_e_volta_e_limite_da_descompressao() {
    assert_eq!(
        descompactar(&compactar("<a>ç</a>").unwrap()).unwrap(),
        "<a>ç</a>"
    );
    let grande = "0".repeat(9 * 1024 * 1024);
    assert!(
        matches!(descompactar(&compactar(&grande).unwrap()), Err(DfeError::Xml(m)) if m.contains("limite"))
    );
    assert!(matches!(
        descompactar("não é base64"),
        Err(DfeError::Xml(_))
    ));
    assert!(matches!(descompactar("AAAA"), Err(DfeError::Xml(_))));
}
