//! SONDAGEM TEMPORÁRIA (19-11-2026) — emissão em homologação com o IBS/CBS que o ecossistema
//! manda hoje para o Simples (CRT 1): CST 000 / cClassTrib 000001, `gIBSCBS` calculado com as
//! alíquotas de teste 0,10 / 0 / 0,90. Serve para ver o que a SEFAZ responde antes de codar a
//! NT 2026.005. Rodar: `cargo test --features sefaz-live --test sondagem_simples_rtc -- --nocapture --test-threads=1`
#![cfg(feature = "sefaz-live")]

use std::str::FromStr;

use dfe::tipos::emissao::{Dest, IbsCbs};
use dfe::tipos::{Cofins, Det, Emit, Icms, Ide, Pag, Pis, Total, Transp};
use dfe::{CancelarBuilder, NFeBuilder};
use rust_decimal::Decimal;

mod common;

fn n_nf() -> u64 {
    (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() % 900_000_000 + 1)
        as u64
}

fn ibs_cbs_de_hoje() -> IbsCbs {
    IbsCbs {
        cst: "000".into(),
        class_trib: "000001".into(),
        v_bc: Decimal::ZERO,
        p_ibs_uf: Decimal::from_str("0.10").unwrap(),
        v_ibs_uf: Decimal::ZERO,
        p_ibs_mun: Decimal::ZERO,
        v_ibs_mun: Decimal::ZERO,
        p_cbs: Decimal::from_str("0.90").unwrap(),
        v_cbs: Decimal::ZERO,
        calcular: true,
        p_red_ibs: None,
        p_red_cbs: None,
    }
}

fn emitente(env: &common::LiveEnv, em: &common::LiveEmitente) -> Emit {
    Emit {
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
    }
}

fn item() -> Det {
    Det {
        c_prod: "001".into(),
        x_prod: "PRODUTO TESTE DFE".into(),
        ncm: "22030000".into(),
        cfop: 5102,
        u_com: "UN".into(),
        q_com: 1.0,
        v_un_com: 10.0,
        v_prod: 10.0,
        u_trib: "UN".into(),
        q_trib: 1.0,
        v_un_trib: 10.0,
        icms: Icms::sn102(0, "102"),
        pis: Pis::Nt { cst: "07".into() },
        cofins: Cofins::Nt { cst: "07".into() },
        ibs_cbs: Some(ibs_cbs_de_hoje()),
        ..Default::default()
    }
}

fn trecho<'a>(xml: &'a str, abre: &str, fecha: &str) -> &'a str {
    match (xml.find(abre), xml.find(fecha)) {
        (Some(i), Some(f)) => &xml[i..f + fecha.len()],
        _ => "(ausente)",
    }
}

#[tokio::test]
async fn sondagem_nfce_65_simples_ibscbs_de_hoje() {
    let (Some(env), Some(em), Some((id_csc, csc))) =
        (common::live_env(), common::live_emitente(), common::live_csc())
    else {
        eprintln!("skip: .env incompleto");
        return;
    };
    println!("CRT do emitente: {}", em.crt);
    let r = NFeBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .ide(Ide {
            c_uf: 35,
            mod_: 65,
            serie: em.serie,
            n_nf: n_nf(),
            c_mun_fg: em.cod_municipio.clone(),
            tp_amb: common::HOMOLOGACAO,
            tp_imp: 4,
            ind_final: 1,
            ind_pres: 1,
            ..Default::default()
        })
        .emitente(emitente(&env, &em))
        .itens(vec![item()])
        .total(Total::default())
        .transporte(Transp { mod_frete: 9, ..Default::default() })
        .pagamento(Pag { ind_pag: 0, t_pag: "01".into(), v_pag: 10.0, ..Default::default() })
        .id_csc(&id_csc)
        .csc(&csc)
        .emitir()
        .await;
    match r {
        Err(e) => println!("NFC-e 65 — erro antes/fora da SEFAZ: {e:?}"),
        Ok(r) => {
            let p = &r.protocolo.inf_prot;
            println!("NFC-e 65 — cStat {} — {}", p.c_stat, p.x_motivo);
            println!("IBSCBS item: {}", trecho(&r.xml, "<IBSCBS>", "</IBSCBS>"));
            println!("IBSCBSTot:   {}", trecho(&r.xml, "<IBSCBSTot>", "</IBSCBSTot>"));
            if dfe::cstat_autorizado(p.c_stat) {
                let c = CancelarBuilder::new()
                    .cert(&env.cert_path, &env.cert_pass)
                    .tp_amb(common::HOMOLOGACAO)
                    .chave(&p.ch_nfe)
                    .protocolo(p.n_prot.as_deref().unwrap_or_default())
                    .justificativa("Sondagem IBS CBS Simples em homologacao")
                    .mod_(65)
                    .send()
                    .await;
                println!("cancelamento: {:?}", c.map(|c| (c.response.c_stat, c.response.x_motivo)));
            }
        }
    }
}

#[tokio::test]
async fn sondagem_nfe_55_simples_ibscbs_de_hoje() {
    let (Some(env), Some(em)) = (common::live_env(), common::live_emitente()) else {
        eprintln!("skip: .env incompleto");
        return;
    };
    let dest = Dest {
        cpf: Some("11144477735".into()),
        x_nome: Some("NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".into()),
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
    let r = NFeBuilder::new()
        .cert(&env.cert_path, &env.cert_pass)
        .ide(Ide {
            c_uf: 35,
            mod_: 55,
            serie: em.serie,
            n_nf: n_nf() + 7,
            c_mun_fg: em.cod_municipio.clone(),
            tp_amb: common::HOMOLOGACAO,
            ..Default::default()
        })
        .emitente(emitente(&env, &em))
        .destinatario(dest)
        .itens(vec![item()])
        .total(Total::default())
        .transporte(Transp { mod_frete: 9, ..Default::default() })
        .pagamento(Pag { ind_pag: 0, t_pag: "01".into(), v_pag: 10.0, ..Default::default() })
        .emitir()
        .await;
    match r {
        Err(e) => println!("NF-e 55 — erro antes/fora da SEFAZ: {e:?}"),
        Ok(r) => {
            let p = &r.protocolo.inf_prot;
            println!("NF-e 55 — cStat {} — {}", p.c_stat, p.x_motivo);
            println!("IBSCBS item: {}", trecho(&r.xml, "<IBSCBS>", "</IBSCBS>"));
            println!("IBSCBSTot:   {}", trecho(&r.xml, "<IBSCBSTot>", "</IBSCBSTot>"));
            if dfe::cstat_autorizado(p.c_stat) {
                let c = CancelarBuilder::new()
                    .cert(&env.cert_path, &env.cert_pass)
                    .tp_amb(common::HOMOLOGACAO)
                    .chave(&p.ch_nfe)
                    .protocolo(p.n_prot.as_deref().unwrap_or_default())
                    .justificativa("Sondagem IBS CBS Simples em homologacao")
                    .mod_(55)
                    .send()
                    .await;
                println!("cancelamento: {:?}", c.map(|c| (c.response.c_stat, c.response.x_motivo)));
            }
        }
    }
}
