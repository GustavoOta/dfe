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
use dfe::{CancelarBuilder, CartaCorrecaoBuilder, InutilizacaoBuilder, NFeBuilder, NFeService};

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
        .environment(common::HOMOLOGACAO)
        .send()
        .await;

    // Falha de certificado/mTLS/transporte derruba o teste — antes só era impressa e o teste
    // passava (no Linux, sem assinatura, ficaria verde sem funcionar).
    let r = r.unwrap_or_else(|e| panic!("live_status_servico: {e:?}"));
    println!("c_stat: {} | x_motivo: {} | url: {}", r.c_stat, r.x_motivo, r.url);
    assert_eq!(r.c_stat, "107", "serviço fora de operação: {}", r.x_motivo);
}

/// Ciclo completo de uma NF-e 55 em **homologação**, com o emitente real do `.env`:
/// emitir (100) → carta de correção (135) → cancelar (135) → inutilizar o número seguinte (102).
/// Cada passo exige o cStat certo; erro da crate (certificado, assinatura, transporte) derruba.
/// O número sai do relógio, na série `DFE_TEST_SERIE` (padrão 99, exclusiva destes testes), para
/// nunca repetir nem colidir com as notas de homologação de quem usa o mesmo CNPJ.
#[tokio::test]
async fn live_ciclo_nfe_55_homologacao() {
    let (Some(env), Some(em)) = (common::live_env(), common::live_emitente()) else {
        eprintln!("skip live_ciclo_nfe_55_homologacao: .env sem certificado ou sem DFE_TEST_IE/COD_MUNICIPIO");
        return;
    };
    let n_nf = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 900_000_000
        + 1;

    // 1. Emissão
    let emitente = Emit {
        cnpj: Some(env.cnpj.clone()),
        ie: Some(em.ie.clone()),
        crt: em.crt,
        x_nome: em.razao.clone(),
        x_lgr: em.logradouro.clone(),
        nro: em.numero.clone(),
        x_cpl: em.complemento.clone(),
        x_bairro: em.bairro.clone(),
        c_mun: em.cod_municipio.clone(),
        x_mun: em.municipio.clone(),
        uf: env.uf.clone(),
        cep: em.cep.clone(),
        ..Default::default()
    };
    let destinatario = Dest {
        cpf: Some("11144477735".to_string()), // CPF de exemplo (DV válido), sem dono real
        x_nome: Some("NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string()),
        x_lgr: Some(em.logradouro.clone()),
        nro: Some(em.numero.clone()),
        x_bairro: Some(em.bairro.clone()),
        c_mun: Some(em.cod_municipio.clone()),
        x_mun: Some(em.municipio.clone()),
        uf: Some(env.uf.clone()),
        cep: Some(em.cep.clone()),
        ind_ie_dest: Some(9),
        ..Default::default()
    };
    let item = Det {
        c_prod: "001".to_string(),
        x_prod: "PRODUTO TESTE DFE".to_string(), // a crate troca pelo texto de homologação
        ncm: "22030000".to_string(),
        cfop: 5102,
        u_com: "UN".to_string(),
        q_com: 1.0,
        v_un_com: 10.0,
        v_prod: 10.0,
        u_trib: "UN".to_string(),
        q_trib: 1.0,
        v_un_trib: 10.0,
        icms: Icms::sn102(0, "102"),
        pis: Pis::Nt { cst: "07".to_string() },
        cofins: Cofins::Nt { cst: "07".to_string() },
        ..Default::default()
    };
    let emissao = NFeBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .ide(Ide {
            c_uf: 35,
            mod_: 55,
            serie: em.serie,
            n_nf,
            c_mun_fg: em.cod_municipio.clone(),
            tp_amb: common::HOMOLOGACAO,
            ..Default::default()
        })
        .emitente(emitente)
        .destinatario(destinatario)
        .itens(vec![item])
        .total(Total::default())
        .transporte(Transp { mod_frete: 9, ..Default::default() })
        .pagamento(Pag { ind_pag: 0, t_pag: "01".to_string(), v_pag: 10.0, ..Default::default() })
        .emitir()
        .await
        .unwrap_or_else(|e| panic!("emissão: {e:?}"));
    let prot = &emissao.protocolo.inf_prot;
    println!("1. emissão série {} nº {n_nf}: cStat {} — {}", em.serie, prot.c_stat, prot.x_motivo);
    assert!(dfe::cstat_autorizado(prot.c_stat), "NF-e não autorizada: {} {}", prot.c_stat, prot.x_motivo);
    let chave = prot.ch_nfe.clone();
    let protocolo = prot.n_prot.clone().expect("autorizada sem nProt");
    println!("   chave {chave} | protocolo {protocolo}");

    // 2. Carta de correção
    let cce = CartaCorrecaoBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .tp_amb(common::HOMOLOGACAO)
        .chave(&chave)
        .correcao("Teste automatizado da crate dfe: correcao do texto das informacoes complementares")
        .n_seq_evento(1)
        .send()
        .await
        .unwrap_or_else(|e| panic!("CC-e: {e:?}"));
    println!("2. CC-e: cStat {} — {}", cce.response.c_stat, cce.response.x_motivo);
    assert_eq!(cce.response.c_stat, "135", "CC-e não registrada: {}", cce.response.x_motivo);

    // 3. Cancelamento
    let canc = CancelarBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .tp_amb(common::HOMOLOGACAO)
        .chave(&chave)
        .protocolo(&protocolo)
        .justificativa("Teste automatizado da crate dfe em homologacao")
        .send()
        .await
        .unwrap_or_else(|e| panic!("cancelamento: {e:?}"));
    println!("3. cancelamento: cStat {} — {}", canc.response.c_stat, canc.response.x_motivo);
    assert_eq!(canc.response.c_stat, "135", "cancelamento não registrado: {}", canc.response.x_motivo);
}

/// UFs que dispensaram a inutilização (Ajustes SINIEF 38/2026 — NF-e — e 40/2026 — NFC-e, a partir
/// de 01/11/2026, segundo fontes de terceiros; SP confirmado em 15-11-2026: `998 Serviço
/// Desativado` para 55 e 65, em homologação, no Windows e no Linux). Lá, salto de numeração não
/// se inutiliza: segue-se do número seguinte.
const UFS_SEM_INUTILIZACAO: [&str; 3] = ["SP", "AC", "BA"];

/// Inutilização em **homologação** de um número nunca usado (do relógio, série `DFE_TEST_SERIE`),
/// para NF-e 55 e NFC-e 65. Nas UFs de [`UFS_SEM_INUTILIZACAO`] o certo é `998` (serviço
/// desativado); nas outras, `102` (homologada). Qualquer outra resposta reprova.
#[tokio::test]
async fn live_inutilizacao_homologacao() {
    let (Some(env), Some(em)) = (common::live_env(), common::live_emitente()) else {
        eprintln!("skip live_inutilizacao_homologacao: .env sem certificado ou sem DFE_TEST_IE/COD_MUNICIPIO");
        return;
    };
    let numero = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 900_000_000
        + 1;
    let esperado = if UFS_SEM_INUTILIZACAO.contains(&env.uf.as_str()) { "998" } else { "102" };
    for modelo in [55u32, 65] {
        let r = InutilizacaoBuilder::new()
            .cert(&env.cert_path, &env.cert_pass)
            .tp_amb(common::HOMOLOGACAO)
            .uf(&env.uf)
            .cnpj(&env.cnpj)
            .mod_(modelo)
            .serie(em.serie)
            .faixa(numero, numero)
            .justificativa("Teste automatizado da crate dfe em homologacao")
            .send()
            .await
            .unwrap_or_else(|e| panic!("inutilização modelo {modelo}: {e:?}"));
        println!("inutilização modelo {modelo} nº {numero}: cStat {} — {}", r.response.c_stat, r.response.x_motivo);
        assert_eq!(
            r.response.c_stat, esperado,
            "inutilização modelo {modelo} em {}: esperado {esperado}, veio {} {} — se a UF mudou de regra, ajustar UFS_SEM_INUTILIZACAO",
            env.uf, r.response.c_stat, r.response.x_motivo
        );
    }
}

/// Ciclo da NFC-e 65 em **homologação**: emitir para consumidor não identificado (100, com QR
/// Code pelo CSC de homologação do `.env`) → cancelar (135). Mesmo emitente e série do ciclo da
/// NF-e; a numeração do modelo 65 é independente. Sem CSC no `.env`, pula.
#[tokio::test]
async fn live_ciclo_nfce_65_homologacao() {
    let (Some(env), Some(em), Some((id_csc, csc))) =
        (common::live_env(), common::live_emitente(), common::live_csc())
    else {
        eprintln!("skip live_ciclo_nfce_65_homologacao: .env sem certificado, emitente ou DFE_TEST_ID_CSC/CSC");
        return;
    };
    let n_nf = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 900_000_000
        + 1;

    let emitente = Emit {
        cnpj: Some(env.cnpj.clone()),
        ie: Some(em.ie.clone()),
        crt: em.crt,
        x_nome: em.razao.clone(),
        x_lgr: em.logradouro.clone(),
        nro: em.numero.clone(),
        x_cpl: em.complemento.clone(),
        x_bairro: em.bairro.clone(),
        c_mun: em.cod_municipio.clone(),
        x_mun: em.municipio.clone(),
        uf: env.uf.clone(),
        cep: em.cep.clone(),
        ..Default::default()
    };
    let item = Det {
        c_prod: "001".to_string(),
        x_prod: "PRODUTO TESTE DFE".to_string(),
        ncm: "22030000".to_string(),
        cfop: 5102,
        u_com: "UN".to_string(),
        q_com: 1.0,
        v_un_com: 10.0,
        v_prod: 10.0,
        u_trib: "UN".to_string(),
        q_trib: 1.0,
        v_un_trib: 10.0,
        icms: Icms::sn102(0, "102"),
        pis: Pis::Nt { cst: "07".to_string() },
        cofins: Cofins::Nt { cst: "07".to_string() },
        ..Default::default()
    };

    // 1. Emissão
    let emissao = NFeBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .ide(Ide {
            c_uf: 35,
            mod_: 65,
            serie: em.serie,
            n_nf,
            c_mun_fg: em.cod_municipio.clone(),
            tp_amb: common::HOMOLOGACAO,
            tp_imp: 4,
            ind_final: 1,
            ind_pres: 1,
            ..Default::default()
        })
        .emitente(emitente)
        .itens(vec![item])
        .total(Total::default())
        .transporte(Transp { mod_frete: 9, ..Default::default() })
        .pagamento(Pag { ind_pag: 0, t_pag: "01".to_string(), v_pag: 10.0, ..Default::default() })
        .id_csc(&id_csc)
        .csc(&csc)
        .emitir()
        .await
        .unwrap_or_else(|e| panic!("emissão NFC-e: {e:?}"));
    let prot = &emissao.protocolo.inf_prot;
    println!("1. NFC-e série {} nº {n_nf}: cStat {} — {}", em.serie, prot.c_stat, prot.x_motivo);
    assert!(dfe::cstat_autorizado(prot.c_stat), "NFC-e não autorizada: {} {}", prot.c_stat, prot.x_motivo);
    let chave = prot.ch_nfe.clone();
    let protocolo = prot.n_prot.clone().expect("autorizada sem nProt");
    println!("   chave {chave} | protocolo {protocolo}");
    assert!(emissao.xml.contains("<qrCode>"), "nfeProc sem QR Code");

    // 2. Cancelamento
    let canc = CancelarBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .tp_amb(common::HOMOLOGACAO)
        .chave(&chave)
        .protocolo(&protocolo)
        .justificativa("Teste automatizado da crate dfe em homologacao")
        .mod_(65)
        .send()
        .await
        .unwrap_or_else(|e| panic!("cancelamento NFC-e: {e:?}"));
    println!("2. cancelamento: cStat {} — {}", canc.response.c_stat, canc.response.x_motivo);
    assert_eq!(canc.response.c_stat, "135", "cancelamento não registrado: {}", canc.response.x_motivo);
}

/// NFC-e com destinatário **CNPJ** (NT 2026.007, regra 12E02-10 → rejeição 181). A SEFAZ
/// procura o CNPJ na LCC-RFB pela chave **UF do destinatário + CNPJ**, e a UF só existe no XML
/// dentro do `<enderDest>`. A homologação consulta a LCC de produção, então o destinatário
/// precisa ser um CNPJ real e ativo — e diferente do emitente (senão 220). Padrão: Banco
/// Bradesco (Osasco/SP, dado público); `DFE_TEST_DEST_CNPJ`/`_UF`/`_CMUN`/`_XMUN` trocam
/// (o município tem de ser da UF, senão 275). Rua e bairro são os do emitente: não entram na LCC. Autoriza e cancela.
#[tokio::test]
async fn live_nfce_65_destinatario_cnpj_homologacao() {
    let (Some(env), Some(em), Some((id_csc, csc))) =
        (common::live_env(), common::live_emitente(), common::live_csc())
    else {
        eprintln!("skip live_nfce_65_destinatario_cnpj_homologacao: .env sem certificado, emitente ou CSC");
        return;
    };
    let n_nf = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        % 900_000_000
        + 1;

    let emitente = Emit {
        cnpj: Some(env.cnpj.clone()),
        ie: Some(em.ie.clone()),
        crt: em.crt,
        x_nome: em.razao.clone(),
        x_lgr: em.logradouro.clone(),
        nro: em.numero.clone(),
        x_cpl: em.complemento.clone(),
        x_bairro: em.bairro.clone(),
        c_mun: em.cod_municipio.clone(),
        x_mun: em.municipio.clone(),
        uf: env.uf.clone(),
        cep: em.cep.clone(),
        ..Default::default()
    };
    let dest_cnpj = std::env::var("DFE_TEST_DEST_CNPJ").unwrap_or_else(|_| "60746948000112".to_string());
    let dest_uf = std::env::var("DFE_TEST_DEST_UF").unwrap_or_else(|_| "SP".to_string());
    let dest_c_mun = std::env::var("DFE_TEST_DEST_CMUN").unwrap_or_else(|_| em.cod_municipio.clone());
    let dest_x_mun = std::env::var("DFE_TEST_DEST_XMUN").unwrap_or_else(|_| em.municipio.clone());
    let destinatario = Dest {
        cnpj: Some(dest_cnpj),
        x_nome: Some("NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string()),
        x_lgr: Some(em.logradouro.clone()),
        nro: Some(em.numero.clone()),
        x_bairro: Some(em.bairro.clone()),
        c_mun: Some(dest_c_mun),
        x_mun: Some(dest_x_mun),
        uf: Some(dest_uf),
        cep: Some(em.cep.clone()),
        ind_ie_dest: Some(9),
        ..Default::default()
    };
    let item = Det {
        c_prod: "001".to_string(),
        x_prod: "PRODUTO TESTE DFE".to_string(),
        ncm: "22030000".to_string(),
        cfop: 5102,
        u_com: "UN".to_string(),
        q_com: 1.0,
        v_un_com: 10.0,
        v_prod: 10.0,
        u_trib: "UN".to_string(),
        q_trib: 1.0,
        v_un_trib: 10.0,
        icms: Icms::sn102(0, "102"),
        pis: Pis::Nt { cst: "07".to_string() },
        cofins: Cofins::Nt { cst: "07".to_string() },
        ..Default::default()
    };

    let emissao = NFeBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .ide(Ide {
            c_uf: 35,
            mod_: 65,
            serie: em.serie,
            n_nf,
            c_mun_fg: em.cod_municipio.clone(),
            tp_amb: common::HOMOLOGACAO,
            tp_imp: 4,
            ind_final: 1,
            ind_pres: 1,
            ..Default::default()
        })
        .emitente(emitente)
        .destinatario(destinatario)
        .itens(vec![item])
        .total(Total::default())
        .transporte(Transp { mod_frete: 9, ..Default::default() })
        .pagamento(Pag { ind_pag: 0, t_pag: "01".to_string(), v_pag: 10.0, ..Default::default() })
        .id_csc(&id_csc)
        .csc(&csc)
        .emitir()
        .await
        .unwrap_or_else(|e| panic!("emissão NFC-e com destinatário CNPJ: {e:?}"));
    let prot = &emissao.protocolo.inf_prot;
    println!(
        "1. NFC-e série {} nº {n_nf} (dest CNPJ, enderDest no XML: {}): cStat {} — {}",
        em.serie,
        emissao.xml.contains("<enderDest>"),
        prot.c_stat,
        prot.x_motivo
    );
    assert!(dfe::cstat_autorizado(prot.c_stat), "NFC-e não autorizada: {} {}", prot.c_stat, prot.x_motivo);
    let chave = prot.ch_nfe.clone();
    let protocolo = prot.n_prot.clone().expect("autorizada sem nProt");

    let canc = CancelarBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .tp_amb(common::HOMOLOGACAO)
        .chave(&chave)
        .protocolo(&protocolo)
        .justificativa("Teste automatizado da crate dfe em homologacao")
        .mod_(65)
        .send()
        .await
        .unwrap_or_else(|e| panic!("cancelamento NFC-e: {e:?}"));
    println!("2. cancelamento: cStat {} — {}", canc.response.c_stat, canc.response.x_motivo);
    assert_eq!(canc.response.c_stat, "135", "cancelamento não registrado: {}", canc.response.x_motivo);
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
        .ambiente(common::HOMOLOGACAO)
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
        .ambiente(common::HOMOLOGACAO)
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
        .ambiente(common::HOMOLOGACAO)
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
        .ambiente(common::HOMOLOGACAO)
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
        .ambiente(common::HOMOLOGACAO)
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
        .ambiente(common::HOMOLOGACAO)
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
        .ambiente(common::HOMOLOGACAO)
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
