//! Ciclo real da NFS-e na **produção restrita** da SEFIN Nacional (opt-in).
//!
//! Precisa de um prestador habilitado no Emissor Nacional, com o A1 de `DFE_TEST_CERT_PATH` e os
//! `DFE_TEST_NFSE_*` do `.env` (ver `env.example`). Sem eles, pula. O ambiente é
//! `common::HOMOLOGACAO_NFSE`, fixo no código.
//!
//! `cargo test --features "sefaz-live nfse" --test nfse_live -- --nocapture --test-threads=1`
#![cfg(all(feature = "sefaz-live", feature = "nfse"))]

mod common;

use chrono::{FixedOffset, Utc};
use dfe::nfse::{
    ConsultaDps, Documento, ErroEnvio, IdentificacaoDps, NfseBuilder, NfseClient, Prestador,
    RegimeTributario, ResultadoEmissao, Servico, TotalTributos, Tributacao,
};
use rust_decimal::Decimal;

#[tokio::test]
async fn live_nfse_emite_e_consulta_na_producao_restrita() {
    let (Some(env), Some(nfse)) = (common::live_env(), common::live_nfse()) else {
        eprintln!("SKIP: DFE_TEST_CERT_PATH/DFE_TEST_NFSE_* não configurados");
        return;
    };
    let documento = if env.cnpj.len() == 11 {
        Documento::Cpf(env.cnpj.clone())
    } else {
        Documento::Cnpj(env.cnpj.clone())
    };
    let agora = Utc::now().with_timezone(&FixedOffset::west_opt(3 * 3600).unwrap());
    let agora = agora - chrono::Duration::nanoseconds(agora.timestamp_subsec_nanos().into());
    let numero = (Utc::now().timestamp() % 900_000_000 + 1) as u64;

    let dps = NfseBuilder::new()
        .ambiente(common::HOMOLOGACAO_NFSE)
        .identificacao(IdentificacaoDps {
            municipio_emissao: nfse.municipio.clone(),
            serie: nfse.serie,
            numero,
            emitida_em: agora,
            competencia: agora.date_naive(),
            versao_aplicativo: "dfe-live".into(),
        })
        .prestador(Prestador {
            documento,
            inscricao_municipal: nfse.inscricao_municipal.clone(),
            regime: if nfse.mei {
                RegimeTributario::Mei
            } else {
                RegimeTributario::SimplesNacional
            },
        })
        .servico(Servico {
            municipio_prestacao: nfse.municipio.clone(),
            codigo_tributacao_nacional: nfse.codigo_tributacao_nacional.clone(),
            codigo_tributacao_municipal: nfse.codigo_tributacao_municipal.clone(),
            codigo_nbs: None,
            descricao: "Servico de teste \"dfe\" em producao restrita".into(),
            valor: Decimal::new(1000, 2),
        })
        .tributacao(Tributacao {
            total: TotalTributos::NaoInformado,
        })
        .gerar_dps()
        .expect("DPS");

    let cliente = NfseClient::builder()
        .ambiente(common::HOMOLOGACAO_NFSE)
        .cert(env.cert_path.clone(), env.cert_pass.clone())
        .build()
        .unwrap();
    let assinada = cliente.assinar(&dps).await.expect("assinatura");
    eprintln!("DPS {}", assinada.id());

    match cliente.transmitir(&assinada).await {
        Ok(ResultadoEmissao::Autorizada(n)) => {
            eprintln!(
                "AUTORIZADA chave {} alertas {:?}",
                n.chave_acesso, n.alertas
            );
            assert!(n.xml.contains(&n.chave_acesso));
            assert_eq!(
                cliente.consultar_dps(assinada.id()).await.unwrap(),
                ConsultaDps::Encontrada {
                    chave_acesso: n.chave_acesso.clone()
                }
            );
            let consultada = cliente.consultar_nfse(&n.chave_acesso).await.unwrap();
            assert!(consultada.xml.contains(assinada.id()));
        }
        Ok(ResultadoEmissao::Rejeitada(r)) => {
            panic!(
                "REJEITADA: {}",
                r.erros
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        }
        Err(ErroEnvio::Indeterminado { .. }) => {
            let consulta = cliente.consultar_dps(assinada.id()).await;
            panic!("indeterminado; consulta da DPS: {consulta:?}");
        }
        Err(e) => panic!("{e}"),
    }
}
