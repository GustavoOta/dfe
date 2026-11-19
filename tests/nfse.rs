#![cfg(feature = "nfse")]

use chrono::{DateTime, NaiveDate};
use dfe::nfse::{
    Ambiente, Documento, IdentificacaoDps, NfseBuilder, Prestador, RegimeTributario, Servico,
    Tomador, TotalTributos, Tributacao,
};
use rust_decimal::Decimal;

fn prestador(documento: Documento) -> Prestador {
    Prestador {
        documento,
        inscricao_municipal: Some("0012".into()),
        regime: RegimeTributario::SimplesNacional,
    }
}

fn servico(municipio: &str) -> Servico {
    Servico {
        municipio_prestacao: municipio.into(),
        codigo_tributacao_nacional: "010701".into(),
        codigo_tributacao_municipal: Some("007".into()),
        codigo_nbs: Some("115032000".into()),
        descricao: "Suporte & manutenção <teste>".into(),
        valor: Decimal::new(12345, 2),
    }
}

fn tomador() -> Tomador {
    Tomador {
        documento: Documento::Cpf("52998224725".into()),
        nome: "Tomador & Cia".into(),
    }
}

fn identificacao(municipio: &str) -> IdentificacaoDps {
    IdentificacaoDps {
        municipio_emissao: municipio.into(),
        serie: 42,
        numero: 7,
        emitida_em: DateTime::parse_from_rfc3339("2026-11-18T09:30:00-03:00").unwrap(),
        competencia: NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
        versao_aplicativo: "teste-1".into(),
    }
}

fn builder(ambiente: Ambiente, municipio: &str) -> NfseBuilder {
    // A DPS é montada em memória nos dois leiautes, sem cliente de rede.
    NfseBuilder::ambiente(NfseBuilder::new(), ambiente)
        .identificacao(identificacao(municipio))
        .prestador(prestador(Documento::Cnpj("11222333000181".into())))
        .servico(servico(municipio))
        .tributacao(Tributacao {
            total: TotalTributos::PercentualSimplesNacional(Decimal::new(675, 2)),
        })
}

#[test]
fn gera_id_e_preserva_os_dados_no_xml() {
    let dps = builder(Ambiente::Producao, "3526100")
        .tomador(tomador())
        .gerar_dps()
        .unwrap();
    assert_eq!(dfe::nfse::DpsGerada::ambiente(&dps), Ambiente::Producao);
    assert_eq!(dps.id(), "DPS352610021122233300018100042000000000000007");
    assert!(dps.xml().contains(&format!("<infDPS Id=\"{}\">", dps.id())));
    assert!(dps
        .xml()
        .contains("<dhEmi>2026-11-18T09:30:00-03:00</dhEmi>"));
    assert!(dps.xml().contains("<vServ>123.45</vServ>"));
    assert!(dps.xml().contains("Suporte &amp; manutenção &lt;teste&gt;"));
    assert!(dps.xml().contains("<xNome>Tomador &amp; Cia</xNome>"));
    assert!(!dps.xml().contains("<finNFSe>"));
}

#[test]
fn cpf_no_id_recebe_tres_zeros() {
    let dps = builder(Ambiente::ProducaoRestrita, "3529906")
        .prestador(prestador(Documento::Cpf("52998224725".into())))
        .gerar_dps()
        .unwrap();
    assert_eq!(dps.id(), "DPS352990610005299822472500042000000000000007");
}

#[test]
fn rejeita_cnpj_alfanumerico_em_producao() {
    let erro = builder(Ambiente::Producao, "3526100")
        .prestador(prestador(Documento::Cnpj("12ABC34501DE35".into())))
        .gerar_dps()
        .unwrap_err();
    assert!(matches!(erro, dfe::DfeError::Validacao(msg) if msg.contains("CNPJ")));
}

#[test]
fn rejeita_valor_que_exigiria_arredondamento() {
    let mut servico = servico("3526100");
    servico.valor = Decimal::new(12345, 3);
    let erro = builder(Ambiente::Producao, "3526100")
        .servico(servico)
        .gerar_dps()
        .unwrap_err();
    assert!(matches!(erro, dfe::DfeError::Validacao(msg) if msg.contains("vServ")));
}

#[test]
fn rejeita_competencia_posterior_ao_dia_local_de_emissao() {
    let mut id = identificacao("3529906");
    id.emitida_em = DateTime::parse_from_rfc3339("2026-11-18T23:59:59-03:00").unwrap();
    id.competencia = NaiveDate::from_ymd_opt(2026, 11, 19).unwrap();
    let erro = builder(Ambiente::ProducaoRestrita, "3529906")
        .identificacao(id.clone())
        .gerar_dps()
        .unwrap_err();
    assert!(matches!(erro, dfe::DfeError::Validacao(msg) if msg.contains("E0015")));

    id.competencia = NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
    let dps = builder(Ambiente::ProducaoRestrita, "3529906")
        .identificacao(id)
        .gerar_dps()
        .unwrap();
    assert!(dps.xml().contains("<dCompet>2026-11-17</dCompet>"));
}

#[test]
fn rejeita_valor_de_servico_zero() {
    let mut servico = servico("3529906");
    servico.valor = Decimal::ZERO;
    let erro = builder(Ambiente::ProducaoRestrita, "3529906")
        .servico(servico)
        .gerar_dps()
        .unwrap_err();
    assert!(matches!(erro, dfe::DfeError::Validacao(msg) if msg.contains("vServ")));
}

#[test]
fn aceita_somente_series_da_faixa_da_api() {
    for (serie, esperada) in [(0, false), (1, true), (49_999, true), (50_000, false)] {
        let mut id = identificacao("3529906");
        id.serie = serie;
        let resultado = builder(Ambiente::ProducaoRestrita, "3529906")
            .identificacao(id)
            .gerar_dps();
        if esperada {
            assert!(resultado.is_ok(), "{resultado:?}");
        } else {
            assert!(matches!(
                resultado, Err(dfe::DfeError::Validacao(msg)) if msg.contains("E0010")
            ));
        }
    }
}

#[test]
fn desconto_positivo_preserva_valor_bruto_e_zero_omite_o_grupo() {
    let dps = builder(Ambiente::ProducaoRestrita, "3529906")
        .desconto_incondicionado(Decimal::new(1005, 2))
        .gerar_dps()
        .unwrap();
    assert!(dps.xml().contains(
        "<vServPrest><vServ>123.45</vServ></vServPrest><vDescCondIncond><vDescIncond>10.05</vDescIncond></vDescCondIncond><trib>"
    ));

    let sem_desconto = builder(Ambiente::ProducaoRestrita, "3529906")
        .desconto_incondicionado(Decimal::ZERO)
        .gerar_dps()
        .unwrap();
    assert!(!sem_desconto.xml().contains("<vDescCondIncond>"));
}

#[test]
fn rejeita_desconto_negativo_com_fracao_de_centavo_ou_maior_ou_igual_ao_bruto() {
    for desconto in [
        Decimal::new(-1, 2),
        Decimal::new(5, 3),
        Decimal::new(12345, 2),
        Decimal::new(12346, 2),
    ] {
        let erro = builder(Ambiente::ProducaoRestrita, "3529906")
            .desconto_incondicionado(desconto)
            .gerar_dps()
            .unwrap_err();
        assert!(matches!(erro, dfe::DfeError::Validacao(msg) if msg.contains("vDescIncond")));
    }
}

#[test]
fn exige_ambiente_explicito() {
    let erro = NfseBuilder::new().gerar_dps().unwrap_err();
    assert!(matches!(erro, dfe::DfeError::Validacao(msg) if msg.contains("ambiente")));
}

#[test]
fn mei_omite_grupos_opcionais() {
    let dps = builder(Ambiente::ProducaoRestrita, "3529906")
        .prestador(Prestador {
            documento: Documento::Cnpj("11222333000181".into()),
            inscricao_municipal: None,
            regime: RegimeTributario::Mei,
        })
        .tributacao(Tributacao {
            total: TotalTributos::NaoInformado,
        })
        .gerar_dps()
        .unwrap();
    assert!(dps.xml().contains("<finNFSe>0</finNFSe>"));
    assert!(dps.xml().contains("<opSimpNac>2</opSimpNac>"));
    assert!(dps.xml().contains("<indTotTrib>0</indTotTrib>"));
    for grupo in ["<toma>", "<regApTribSN>", "<IM>"] {
        assert!(!dps.xml().contains(grupo));
    }
}

#[cfg(feature = "validacao-xsd")]
#[test]
fn dps_de_cada_ambiente_confere_com_o_xsd_oficial() {
    use libxml::parser::Parser;
    use libxml::schemas::{SchemaParserContext, SchemaValidationContext};
    use std::path::Path;

    for (ambiente, municipio, pacote, documento) in [
        (Ambiente::Producao, "3526100", "prod", "11222333000181"),
        (
            Ambiente::ProducaoRestrita,
            "3529906",
            "rest",
            "12ABC34501DE35",
        ),
    ] {
        let dps = builder(ambiente, municipio)
            .prestador(prestador(Documento::Cnpj(documento.into())))
            .tomador(tomador())
            .desconto_incondicionado(Decimal::new(1005, 2))
            .gerar_dps()
            .unwrap();
        let arquivo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/nfse")
            .join(pacote)
            .join("DPS_v1.01.xsd");
        let mut parser = SchemaParserContext::from_file(arquivo.to_str().unwrap());
        let mut contexto = SchemaValidationContext::from_parser(&mut parser).unwrap();
        let doc = Parser::default().parse_string(dps.xml()).unwrap();
        let resultado = contexto.validate_document(&doc);
        match ambiente {
            Ambiente::ProducaoRestrita => {
                resultado.expect("DPS conforme o XSD de produção restrita")
            }
            Ambiente::Producao => {
                // O XSD de produção usa ^ e $ literais em TSSerieDPS.
                // O schema original deve apontar apenas esse erro.
                let erros = resultado.expect_err("revisar o teste se o XSD corrigir TSSerieDPS");
                assert_eq!(erros.len(), 1, "erros adicionais no XSD: {erros:?}");
                let mensagem = erros[0].message.as_deref().unwrap_or_default();
                assert!(
                    mensagem.contains("}serie'") && mensagem.contains(r"^0{0,4}\d{1,5}$"),
                    "erro inesperado no XSD: {mensagem}"
                );
            }
        }
    }
}
