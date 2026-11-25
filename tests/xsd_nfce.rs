#![cfg(feature = "emissao")]

use dfe::tipos::{Det, Emit, Icms, Ide, Pag, Total, Transp};
use dfe::{is_xml_valid, DfeError, NFeBuilder};

mod common;

const DATA_CONTINGENCIA: &str = "2026-11-25T09:59:00-03:00";
const JUSTIFICATIVA: &str = "Falha de comunicacao com a SEFAZ";

async fn gerar_nfce_offline() -> String {
    let cert = common::fixtures_dir().join("cert/sintetico_aes.pfx");
    let item = Det {
        c_prod: "001".into(),
        x_prod: "Item de teste".into(),
        ncm: "10063021".into(),
        u_com: "UN".into(),
        q_com: 1.0,
        v_un_com: 10.0,
        v_prod: 10.0,
        q_trib: 1.0,
        v_un_trib: 10.0,
        icms: Icms::sn102(0, "102"),
        ..Default::default()
    };

    NFeBuilder::new()
        .cert(cert.to_str().expect("caminho do PFX"), "senha-teste-dfe")
        .ide(Ide {
            c_nf: Some("12345678".into()),
            dh_emi: Some("2026-11-25T10:00:00-03:00".into()),
            mod_: 65,
            tp_imp: 4,
            tp_emis: 9,
            tp_amb: common::HOMOLOGACAO,
            dh_cont: Some(DATA_CONTINGENCIA.into()),
            x_just: Some(JUSTIFICATIVA.into()),
            ..Default::default()
        })
        .emitente(Emit {
            cnpj: Some("12345678000195".into()),
            ie: Some("110042490114".into()),
            x_lgr: "Rua de Teste".into(),
            nro: "1".into(),
            x_bairro: "Centro".into(),
            c_mun: "3550308".into(),
            x_mun: "Sao Paulo".into(),
            uf: "SP".into(),
            cep: "01001000".into(),
            ..Default::default()
        })
        .itens(vec![item.clone(), item])
        .pagamento(Pag {
            t_pag: "01".into(),
            v_pag: 20.0,
            ..Default::default()
        })
        .transporte(Transp {
            mod_frete: 9,
            ..Default::default()
        })
        .total(Total::default())
        .gerar_xml()
        .await
        .expect("gerar e assinar a NFC-e em contingência com o certificado sintético")
}

fn substituir(xml: &str, original: &str, novo: &str) -> String {
    assert_eq!(
        xml.matches(original).count(),
        1,
        "trecho da NFC-e: {original}"
    );
    assert_ne!(original, novo, "a variante precisa mudar o XML");
    xml.replacen(original, novo, 1)
}

fn conferir_xsd(caso: &str, xml: &str, deve_aceitar: bool) {
    match (deve_aceitar, is_xml_valid(xml)) {
        (true, Ok(validado)) => assert_eq!(validado, xml, "{caso}: XML alterado"),
        (false, Err(DfeError::Validacao(_))) => {}
        (true, Err(erro)) => panic!("{caso}: o XSD deveria aceitar: {erro}"),
        (false, Ok(_)) => panic!("{caso}: o XSD deveria rejeitar"),
        (false, Err(erro)) => panic!("{caso}: esperava erro de leiaute, recebi {erro:?}"),
    }
}

#[tokio::test]
async fn nfce_offline_referencia_xsd() {
    let xml = gerar_nfce_offline().await;
    assert!(xml.contains("<mod>65</mod>") && xml.contains("<tpEmis>9</tpEmis>"));
    assert!(xml.contains("<qrCode><![CDATA[") && xml.contains("|3|2|"));
    assert!(!xml.contains("<protNFe"));
    conferir_xsd("NFC-e offline com QR Code v3", &xml, true);

    // As alterações exercitam o leiaute. Validar XSD não recalcula a assinatura.
    for (tamanho, aceita) in [(14, false), (15, true), (256, true), (257, false)] {
        let variante = substituir(&xml, JUSTIFICATIVA, &"A".repeat(tamanho));
        conferir_xsd(
            &format!("xJust com {tamanho} caracteres"),
            &variante,
            aceita,
        );
    }

    let data = format!("<dhCont>{DATA_CONTINGENCIA}</dhCont>");
    for (caso, original, novo) in [
        (
            "modelo fora da enumeração",
            "<mod>65</mod>",
            "<mod>64</mod>",
        ),
        (
            "série antes do modelo",
            "<mod>65</mod><serie>1</serie>",
            "<serie>1</serie><mod>65</mod>",
        ),
        ("xJust sem dhCont", data.as_str(), ""),
        ("nItem duplicado", r#"nItem="2""#, r#"nItem="1""#),
        (
            "namespace da NFe incorreto",
            r#"<NFe xmlns="http://www.portalfiscal.inf.br/nfe">"#,
            r#"<NFe xmlns="urn:dfe:teste">"#,
        ),
        ("versão de QR Code inválida", "|3|2|", "|0|2|"),
    ] {
        conferir_xsd(caso, &substituir(&xml, original, novo), false);
    }

    let (prefixo, _) = xml
        .split_once("<Signature xmlns=")
        .expect("assinatura da NFC-e");
    conferir_xsd("Signature ausente", &format!("{prefixo}</NFe>"), false);

    // O XSD permite omitir o grupo inteiro. A exigência para tpEmis=9 vive no builder.
    let grupo = format!("{data}<xJust>{JUSTIFICATIVA}</xJust>");
    conferir_xsd(
        "grupo de contingência ausente",
        &substituir(&xml, &grupo, ""),
        true,
    );
}
