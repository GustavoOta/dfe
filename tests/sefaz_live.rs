//! Camada 3 (SEFAZ-live, gated) — bate em **homologação/produção** com certificado real.
//!
//! **Fora do `cargo test` padrão.** Rodar com:
//! ```bash
//! cargo test --features sefaz-live
//! ```
//! e um `.env` configurado (ver `env.example`). Sem `.env`, cada teste é ignorado (skip),
//! nunca falha por falta de segredo. O arquivo inteiro é compilado só sob a feature — no
//! build padrão vira um binário de teste vazio.
//!
//! Consolidado na Fase 0.2 a partir de `src/bin/tests/mod.rs` (status/emissão/cancelamento) e
//! `src/distribuicao/test.rs` (distribuição/manifestação).
#![cfg(feature = "sefaz-live")]

#[cfg(feature = "distribuicao")]
use dfe::distribuicao::{
    CienciaOperacao, ConfirmacaoOperacao, Consulta, ConsultaChaveAcesso, ConsultaNSU,
    DesconhecimentoOperacao, DistribuicaoResposta, OperacaoNaoRealizada,
};
use dfe::tipos::emissao::Dest;
use dfe::tipos::{Cofins, Det, Emit, Icms, Ide, Pag, Pis, Total, Transp};
use dfe::{CancelarBuilder, NFeBuilder, NFeService};

mod common;

// ── Status / emissão / cancelamento ──────────────────────────────────────────

#[tokio::test]
async fn live_status_servico() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_status_servico: .env não configurado (ver env.example)");
        return;
    };

    let r = NFeService::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .uf(&env.uf)
        .environment(env.tp_amb)
        .send()
        .await;

    match r {
        Err(e) => println!("Erro live_status_servico: {:?}", e),
        Ok(r) => {
            println!("c_stat: {}", r.c_stat);
            println!("x_motivo: {}", r.x_motivo);
            println!("url: {}", r.url);
        }
    }
}

#[tokio::test]
async fn live_emit_nfe() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_emit_nfe: .env não configurado");
        return;
    };

    let ide = Ide {
        c_uf: 35, // 35 = São Paulo
        mod_: 55,
        serie: 1,
        n_nf: 3,
        c_mun_fg: "3507605".to_string(),
        tp_nf: 1,
        tp_emis: 1,
        tp_amb: 2,
        ind_final: 1,
        ind_pres: 1,
        tp_imp: 1,
        ..Default::default()
    };

    let emitente = Emit {
        cnpj: Some("00000000000000".to_string()),
        ie: Some("000000000000".to_string()),
        crt: 3,
        x_nome: "EMPRESA DE TESTE LTDA".to_string(),
        x_fant: Some("EMPRESA TESTE".to_string()),
        x_lgr: "RUA DAS FLORES".to_string(),
        nro: "123".to_string(),
        x_bairro: "CENTRO".to_string(),
        c_mun: "3507605".to_string(),
        x_mun: "BAURU".to_string(),
        uf: "SP".to_string(),
        cep: "17010000".to_string(),
        ..Default::default()
    };

    let destinatario = Dest {
        cpf: Some("07068093868".to_string()),
        x_nome: Some("NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string()),
        x_lgr: Some("AV PAULISTA".to_string()),
        nro: Some("1000".to_string()),
        x_bairro: Some("BELA VISTA".to_string()),
        c_mun: Some("3550308".to_string()),
        x_mun: Some("SAO PAULO".to_string()),
        uf: Some("SP".to_string()),
        cep: Some("01310100".to_string()),
        ind_ie_dest: Some(9),
        ..Default::default()
    };

    let itens = vec![
        Det {
            c_prod: "001".to_string(),
            x_prod: "PRODUTO TESTE 1".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 1.0,
            v_un_com: 10.0,
            v_prod: 10.0,
            u_trib: "UN".to_string(),
            q_trib: 1.0,
            v_un_trib: 10.0,
            icms: Icms::Icms00 {
                orig: 0,
                mod_bc: 3,
                v_bc: 10.0,
                p_icms: 12.0,
                v_icms: 1.20,
            },
            pis: Pis::Aliq {
                cst: "01".to_string(),
                v_bc: 10.0,
                p_pis: 0.65,
                v_pis: 0.07,
            },
            cofins: Cofins::Aliq {
                cst: "01".to_string(),
                v_bc: 10.0,
                p_cofins: 3.0,
                v_cofins: 0.30,
            },
            ..Default::default()
        },
        Det {
            c_prod: "002".to_string(),
            x_prod: "PRODUTO TESTE 2".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 2.0,
            v_un_com: 10.0,
            v_prod: 20.0,
            u_trib: "UN".to_string(),
            q_trib: 2.0,
            v_un_trib: 10.0,
            icms: Icms::Icms00 {
                orig: 0,
                mod_bc: 3,
                v_bc: 20.0,
                p_icms: 12.0,
                v_icms: 2.40,
            },
            pis: Pis::Aliq {
                cst: "01".to_string(),
                v_bc: 20.0,
                p_pis: 0.65,
                v_pis: 0.13,
            },
            cofins: Cofins::Aliq {
                cst: "01".to_string(),
                v_bc: 20.0,
                p_cofins: 3.0,
                v_cofins: 0.60,
            },
            ..Default::default()
        },
    ];

    let total = Total::default();
    let transporte = Transp {
        mod_frete: 9,
        ..Default::default()
    };
    let pagamento = Pag {
        ind_pag: 0,
        t_pag: "01".to_string(),
        v_pag: 30.0,
        ..Default::default()
    };

    let resultado = NFeBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .ide(ide)
        .emitente(emitente)
        .destinatario(destinatario)
        .itens(itens)
        .total(total)
        .transporte(transporte)
        .pagamento(pagamento)
        .emitir()
        .await;

    match resultado {
        Err(e) => println!("Erro live_emit_nfe: {:?}", e),
        Ok(response) => {
            println!("c_stat: {}", response.protocolo.inf_prot.c_stat);
            println!("x_motivo: {}", response.protocolo.inf_prot.x_motivo);
        }
    }
}

#[tokio::test]
async fn live_emit_nfce() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_emit_nfce: .env não configurado");
        return;
    };

    let ide = Ide {
        c_uf: 35,
        mod_: 65,
        serie: 1,
        n_nf: 1,
        c_mun_fg: "3507605".to_string(),
        tp_nf: 1,
        tp_emis: 1,
        tp_amb: 2,
        ind_final: 1,
        ind_pres: 1,
        tp_imp: 4,
        ..Default::default()
    };

    let emitente = Emit {
        cnpj: Some("00000000000000".to_string()),
        ie: Some("000000000000".to_string()),
        crt: 3,
        x_nome: "EMPRESA DE TESTE LTDA".to_string(),
        x_lgr: "RUA DAS FLORES".to_string(),
        nro: "123".to_string(),
        x_bairro: "CENTRO".to_string(),
        c_mun: "3507605".to_string(),
        x_mun: "BAURU".to_string(),
        uf: "SP".to_string(),
        cep: "17010000".to_string(),
        ..Default::default()
    };

    let itens = vec![
        Det {
            c_prod: "001".to_string(),
            x_prod: "PRODUTO TESTE".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 1.0,
            v_un_com: 15.0,
            v_prod: 15.0,
            u_trib: "UN".to_string(),
            q_trib: 1.0,
            v_un_trib: 15.0,
            icms: Icms::icms00(0, 3, 15.0, 12.0, 1.80),
            pis: Pis::Outr,
            cofins: Cofins::Outr {
                cst: "99".to_string(),
            },
            ..Default::default()
        },
        Det {
            c_prod: "002".to_string(),
            x_prod: "PRODUTO TESTE 2".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 2.0,
            v_un_com: 7.50,
            v_prod: 15.0,
            u_trib: "UN".to_string(),
            q_trib: 2.0,
            v_un_trib: 7.50,
            icms: Icms::icms00(0, 3, 15.0, 12.0, 1.80),
            pis: Pis::Outr,
            cofins: Cofins::Outr {
                cst: "99".to_string(),
            },
            ..Default::default()
        },
    ];

    let pagamento = Pag {
        ind_pag: 0,
        t_pag: "01".to_string(),
        v_pag: 30.0,
        ..Default::default()
    };

    let resultado = NFeBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .ide(ide)
        .emitente(emitente)
        .itens(itens)
        .total(Total::default())
        .transporte(Transp {
            mod_frete: 9,
            ..Default::default()
        })
        .pagamento(pagamento)
        .id_csc("000001")
        .csc("CODIGO_CSC_AQUI")
        .emitir()
        .await;

    match resultado {
        Err(e) => println!("Erro live_emit_nfce: {:?}", e),
        Ok(response) => {
            println!("c_stat: {}", response.protocolo.inf_prot.c_stat);
            println!("x_motivo: {}", response.protocolo.inf_prot.x_motivo);
        }
    }
}

#[tokio::test]
async fn live_cancelar_nfe() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_cancelar_nfe: .env não configurado");
        return;
    };

    let resultado = CancelarBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .tp_amb(2)
        .chave("35000000000000000000550010000000001000000001")
        .protocolo("135000000000001")
        .justificativa("Cancelamento de teste em homologacao")
        .send()
        .await;

    match resultado {
        Err(e) => println!("Erro live_cancelar_nfe: {:?}", e),
        Ok(r) => {
            println!("c_stat: {}", r.response.c_stat);
            println!("x_motivo: {}", r.response.x_motivo);
        }
    }
}

// ── Distribuição de DF-e / manifestação do destinatário ──────────────────────

#[cfg(feature = "distribuicao")]
#[tokio::test]
async fn live_distribuicao_consulta() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_distribuicao_consulta: .env não configurado");
        return;
    };

    let result = Consulta::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .cnpj(&env.cnpj)
        .uf(35)
        .ambiente(1)
        .send()
        .await;

    assert!(result.is_ok(), "Falha ao consultar distribuição: {:?}", result);
    println!("Distribuição:\n{:#?}", result.unwrap());
}

#[cfg(feature = "distribuicao")]
#[tokio::test]
async fn live_distribuicao_consulta_nsu() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_distribuicao_consulta_nsu: .env não configurado");
        return;
    };

    let result = ConsultaNSU::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .cnpj(&env.cnpj)
        .uf(35)
        .ambiente(1)
        .nsu("000000000000083")
        .check_flag()
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao consultar distribuição por NSU: {:?}",
        result
    );
    println!("Distribuição por NSU:\n{:#?}", result.unwrap());
}

#[cfg(feature = "distribuicao")]
#[tokio::test]
async fn live_distribuicao_consulta_chave_acesso() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_distribuicao_consulta_chave_acesso: .env não configurado");
        return;
    };

    let result: Result<DistribuicaoResposta, String> = ConsultaChaveAcesso::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .cnpj(&env.cnpj)
        .uf(35)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393")
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao consultar distribuição por chave de acesso: {:?}",
        result
    );
    println!("Distribuição por Chave de Acesso:\n{:#?}", result.unwrap());
}

#[cfg(feature = "distribuicao")]
#[tokio::test]
async fn live_ciencia_operacao() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_ciencia_operacao: .env não configurado");
        return;
    };

    let result = CienciaOperacao::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .cnpj(&env.cnpj)
        .ambiente(2)
        .chave_acesso("35260402084385000148550010008921991199917393")
        .send()
        .await;

    assert!(result.is_ok(), "Falha ao enviar ciência da operação: {:?}", result);
    println!("Ciência da operação:\n{:#?}", result.unwrap());
}

#[cfg(feature = "distribuicao")]
#[tokio::test]
async fn live_confirmacao_operacao() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_confirmacao_operacao: .env não configurado");
        return;
    };

    let result = ConfirmacaoOperacao::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .cnpj(&env.cnpj)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393")
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao enviar confirmação da operação: {:?}",
        result
    );
    println!("Confirmação da operação:\n{:#?}", result.unwrap());
}

#[cfg(feature = "distribuicao")]
#[tokio::test]
async fn live_desconhecimento_operacao() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_desconhecimento_operacao: .env não configurado");
        return;
    };

    let result = DesconhecimentoOperacao::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .cnpj(&env.cnpj)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393")
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao enviar desconhecimento da operação: {:?}",
        result
    );
    println!("Desconhecimento da operação:\n{:#?}", result.unwrap());
}

#[cfg(feature = "distribuicao")]
#[tokio::test]
async fn live_operacao_nao_realizada() {
    let Some(env) = common::live_env() else {
        eprintln!("skip live_operacao_nao_realizada: .env não configurado");
        return;
    };

    let result = OperacaoNaoRealizada::new()
        .cert_path(&env.cert_path)
        .cert_pass(&env.cert_pass)
        .cnpj(&env.cnpj)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393")
        .justificativa("Operacao nao realizada para teste")
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao enviar operação não realizada: {:?}",
        result
    );
    println!("Operação não realizada:\n{:#?}", result.unwrap());
}
