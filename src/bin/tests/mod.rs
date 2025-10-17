mod certificate_info;
/// Direitos de autor e licença:
/// Este arquivo fonte é parte do projeto dfe-rs.
/// O projeto dfe-rs pode ser usado de acordo com a Licença MIT
/// que pode ser encontrada no arquivo LICENSE na raiz do projeto.
/// Todos os arquivos fonte do projeto dfe-rs, exceto indicado o contrário, são distribuídos
/// sob a licença MIT. Se você não recebeu uma cópia da licença, consulte o arquivo LICENSE.
/// Autor: Gustavo Ota - Gravis Tec
/// WhatsApp: +55 13 99782 1459 - https://api.whatsapp.com/send?phone=5513997821459

/// Este software está em desenvolvimento e não deve ser usado em produção a não ser que você saiba o que está fazendo.
/// Este software é distribuído sem garantia e sem nenhuma responsabilidade de seus autores ou contribuidores.
mod test_xml_extractor;

#[cfg(test)]

/// TODO: Mudar o tipo USE para receber path, pass, environment, federative_unit, svc, nfe e nfce
#[tokio::test]
async fn test_service_status() {
    use dfe::nfe::service_status;
    use dfe::nfe::types::config::*;

    let teste = service_status(Use::ManualConfig(Fields {
        cert_path: "D:/Projetos/cert.pfx".to_string(),
        cert_pass: Password::Phrase("1234".to_string()),
        federative_unit: "SP".to_string(),
        environment: Environment::Homologation,
    }))
    .await;
    if teste.is_err() {
        println!("Error test_service_status_custom_pass:{:?}", teste.err());
        assert!(false);
        return;
    }
    let teste = teste.unwrap();

    println!("tp_amb: {}", teste.tp_amb);
    println!("ver_aplic: {}", teste.ver_aplic);
    println!("c_stat: {}", teste.c_stat);
    println!("x_motivo: {}", teste.x_motivo);
    println!("c_uf: {}", teste.c_uf);
    println!("dh_recbto: {}", teste.dh_recbto);
    println!("t_med: {}", teste.t_med);
}

/// Emisão de uma NFe
#[tokio::test]
async fn test_emit_nfe_nfce() {
    use dfe::nfe::autorizacao::emit;
    use dfe::nfe::types::autorizacao4::*;
    use dfe::nfe::xml_rules::dest::models::Dest;
    use dfe::nfe::xml_rules::ide::models::Ide;

    let teste = emit(NFe {
        cert_path: "D:/Projetos/cert.pfx".to_string(),
        cert_pass: "1234".to_string(),
        id_csc: None,
        csc: None,
        ide: Ide {
            c_uf: 35,
            serie: 1,
            n_nf: 3,
            c_mun_fg: "3507605".to_string(),
            tp_emis: 1,
            tp_amb: 2,
            ind_final: 1,
            ind_pres: 1,
            mod_: 55,
            tp_imp: 1,
            ..Default::default()
        },
        emit: Emit {
            cnpj: Some("00000000000000".to_string()),
            ie: Some("000000000000".to_string()),
            crt: 3,
            x_nome: "EMPRESA DE TESTE".to_string(),
            x_fant: Some("TESTANDO EMPREENDIMENTOS".to_string()),
            x_lgr: "RUA TESTE".to_string(),
            nro: "123".to_string(),
            x_bairro: "CENTRO".to_string(),
            c_mun: "3529906".to_string(),
            x_mun: "SÃO PAULO".to_string(),
            uf: "SP".to_string(),
            cep: "11850000".to_string(),
            ..Default::default()
        },
        dest: Some(Dest {
            cpf: Some("07068093868".to_string()),
            //cnpj: Some("56196407000190".to_string()), // com ie
            //cnpj: Some("46395000000139".to_string()), // sem ie
            x_nome: Some("NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string()),
            x_lgr: Some("RUA TESTE".to_string()),
            nro: Some("123".to_string()),
            x_bairro: Some("CENTRO".to_string()),
            c_mun: Some("3550308".to_string()),
            x_mun: Some("SÃO PAULO".to_string()),
            uf: Some("SP".to_string()),
            cep: Some("11850000".to_string()),
            //c_pais: Some("1058".to_string()),
            //x_pais: Some("BRASIL".to_string()),
            //fone: Some("11999999999".to_string()),
            ind_ie_dest: Some(9),
            //ie: Some("150344006118".to_string()),
            ..Default::default()
        }),
        det: vec![
            Det {
                c_prod: "123456".to_string(),
                x_prod: "NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL"
                    .to_string(),
                ncm: "22030000".to_string(),
                cfop: 5102,
                u_com: "UN".to_string(),
                q_com: 1.0,
                v_un_com: 10.0,
                v_prod: 10.0,
                u_trib: "CX".to_string(),
                q_trib: 1.0,
                v_un_trib: 10.00,
                ind_tot: 1,
                // TODO: Dispobilizar todos os tipos de ICMS
                // Disponivel: -> ICMS40 ou ICMSSN102
                // orig -> 0
                // CST -> 41
                // csosn -> 102
                icms: "ICMS00".to_string(),
                orig: Some(0),
                cst: Some("00".to_string()),
                mod_bc: Some(3),
                v_bc: Some(10.0),
                p_icms: Some(12.0),
                v_icms: Some(1.20),
                pis: "PISAliq".to_string(),
                pis_cst: Some("77".to_string()),
                pis_v_bc: Some(8.80),
                pis_p_pis: Some(1.0),
                pis_v_pis: Some(0.88),
                cofins: "COFINSAliq".to_string(),
                cofins_cst: Some("01".to_string()),
                cofins_v_bc: Some(8.80),
                cofins_p_cofins: Some(1.0),
                cofins_v_cofins: Some(0.88),
                ..Default::default()
            },
            Det {
                c_prod: "123456".to_string(),
                x_prod: "PRODUTO TESTE 2".to_string(),
                ncm: "22030000".to_string(),
                cfop: 5102,
                u_com: "UN".to_string(),
                q_com: 2.0,
                v_un_com: 10.0,
                v_prod: 20.0,
                u_trib: "CX".to_string(),
                q_trib: 2.0,
                v_un_trib: 10.0,
                ind_tot: 1,
                icms: "ICMS00".to_string(),
                orig: Some(0),
                cst: Some("00".to_string()),
                mod_bc: Some(3),
                v_bc: Some(20.0),
                p_icms: Some(12.0),
                v_icms: Some(2.40),
                pis: "PISAliq".to_string(),
                pis_cst: Some("01".to_string()),
                pis_v_bc: Some(17.60),
                pis_p_pis: Some(1.0),
                pis_v_pis: Some(1.76),
                cofins: "COFINSAliq".to_string(),
                cofins_cst: Some("01".to_string()),
                cofins_v_bc: Some(17.60),
                cofins_p_cofins: Some(1.0),
                cofins_v_cofins: Some(1.76),
                ..Default::default()
            },
        ],
        total: Total {
            v_bc: 30.0,
            v_icms: 3.6,
            v_icms_deson: 0.0,
            v_fcpuf_dest: 0.0,
            v_icms_uf_dest: 0.0,
            v_icms_uf_remet: 0.0,
            v_fcp: 0.0,
            v_bc_st: 0.0,
            v_st: 0.0,
            v_fcpst: 0.0,
            v_fcpst_ret: 0.0,
            v_prod: 30.0,
            v_frete: 0.0,
            v_seg: 0.0,
            v_desc: 0.0,
            v_ii: 0.0,
            v_ipi: 0.0,
            v_ipi_devol: 0.0,
            v_pis: 2.64,
            v_cofins: 2.64,
            v_outro: 0.0,
            v_nf: 30.0,
            v_tot_trib: 0.0,
        },
        transp: Transp {
            mod_frete: 0,
            ..Default::default()
        },
        pag: Pag {
            ind_pag: 1,
            t_pag: "01".to_string(),
            v_pag: 30.0,
            ..Default::default()
        },
        inf_adic: None,
        active_ibs_cbs: None,
    })
    .await;

    if let Err(e) = teste {
        println!("Erro: {:?}", e);
    } else {
        if let Ok(response) = teste {
            println!("Response: {:?}", response.protocolo);

            std::fs::write("tested_response_from_sefaz.xml", response.xml)
                .expect("Falha ao salvar o XML");
            println!("XML salvo em ./tested_response_from_sefaz.xml");
        }
    }
}

/// Cancelamento de uma NFe

#[tokio::test]
async fn test_cancel_nfe_nfce() {
    /* use dfe::nfe::cancelar::nfe_cancelar;
    use dfe::nfe::types::cancelar::*;

    let teste = nfe_cancelar(NFeCancelar {
        cert_path: "D:/Projetos/cert.pfx".to_string(),
        cert_pass: "1234".to_string(),
        tp_amb: 2,
        mod_: Some(55),
        chave: "35241211111111111111550010000000381505051324".to_string(),
        protocolo: "1352400000006702".to_string(),
        justificativa: "TESTE DE CANCELAMENTO".to_string(),
    })
    .await;

    if let Err(e) = teste {
        println!("Erro: {:?}", e);
    } else {
        println!("Response: {:?}", teste.unwrap().response);
    } */
}
