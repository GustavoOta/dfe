mod certificate_info;
/// Direitos de autor e licença:
/// Este arquivo fonte é parte do projeto dfe em crates.io.
/// O projeto dfe pode ser usado de acordo com a Licença MIT
/// que pode ser encontrada no arquivo LICENSE na raiz do projeto.
/// Todos os arquivos fonte do projeto dfe, exceto indicado o contrário, são distribuídos
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
    use rust_decimal::Decimal;

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
            v_troco: Some(Decimal::new(1213, 2)), // 12.13 de troco
            ..Default::default()
        },
        inf_adic: None,
        active_ibs_cbs: None,
        desconto_rateio: None,
        acrescimo_rateio: None,
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

#[tokio::test]
async fn test_danfe_builder() {
    use dfe::pdf::DanfeBuilder;

    match DanfeBuilder::new()
        .xml("./sample55.xml")
        .paper_size("80mm")
        .as_file("./danfe_output.pdf")
        .build()
        .await
    {
        Ok(output) => {
            // Retorna o caminho fornecido do arquivo gerado
            println!("{}", output);
            // let is_file = std::path::Path::new(&output).exists();
            /*
            assert!(
                is_file,
                "O arquivo PDF não foi criado no caminho especificado"
            );
            */
        }
        Err(e) => {
            println!("Erro:{}", e);
            assert!(false); // Falha no teste se ocorrer um erro
        }
    };
}

#[tokio::test]
async fn test_danfe_builder_base64() {
    use dfe::pdf::DanfeBuilder;

    let _xml_test_65 = r##"<?xml version="1.0" encoding="UTF-8"?><nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe Id="NFe35241100000000000191650010000000011000000001" versao="4.00"><ide><cUF>35</cUF><cNF>00000000</cNF><natOp>VENDA AO CONSUMIDOR</natOp><mod>65</mod><serie>1</serie><nNF>1</nNF><dhEmi>2024-11-18T14:01:12-03:00</dhEmi><tpNF>1</tpNF><idDest>1</idDest><cMunFG>3550308</cMunFG><tpImp>4</tpImp><tpEmis>1</tpEmis><cDV>1</cDV><tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal><indPres>1</indPres><procEmi>0</procEmi><verProc>1</verProc></ide><emit><CNPJ>00000000000191</CNPJ><xNome>EMPRESA FICTICIA LTDA</xNome><enderEmit><xLgr>RUA DAS FLORES</xLgr><nro>100</nro><xBairro>CENTRO</xBairro><cMun>3550308</cMun><xMun>Sao Paulo</xMun><UF>SP</UF><CEP>01001000</CEP><cPais>1058</cPais><xPais>BRASIL</xPais></enderEmit><IE>000000000000</IE><CRT>1</CRT></emit><det nItem="1"><prod><cProd>1</cProd><cEAN>00000000</cEAN><xProd>PRODUTO TESTE UNITARIO 500ML</xProd><NCM>22030000</NCM><CFOP>5405</CFOP><uCom>UN</uCom><qCom>1.0000</qCom><vUnCom>10.00</vUnCom><vProd>10.00</vProd><cEANTrib>00000000</cEANTrib><uTrib>UN</uTrib><qTrib>1.0000</qTrib><vUnTrib>10.00</vUnTrib><indTot>1</indTot></prod><imposto><vTotTrib>0.00</vTotTrib><ICMS><ICMSSN500><orig>0</orig><CSOSN>500</CSOSN></ICMSSN500></ICMS><PIS><PISOutr><CST>99</CST><qBCProd>0</qBCProd><vAliqProd>0</vAliqProd><vPIS>0.00</vPIS></PISOutr></PIS><COFINS><COFINSOutr><CST>99</CST><qBCProd>0</qBCProd><vAliqProd>0</vAliqProd><vCOFINS>0.00</vCOFINS></COFINSOutr></COFINS></imposto></det><total><ICMSTot><vBC>0.00</vBC><vICMS>0.00</vICMS><vICMSDeson>0.00</vICMSDeson><vFCP>0.00</vFCP><vBCST>0.00</vBCST><vST>0.00</vST><vFCPST>0.00</vFCPST><vFCPSTRet>0.00</vFCPSTRet><vProd>10.00</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg><vDesc>0.00</vDesc><vII>0.00</vII><vIPI>0.00</vIPI><vIPIDevol>0.00</vIPIDevol><vPIS>0.00</vPIS><vCOFINS>0.00</vCOFINS><vOutro>0.00</vOutro><vNF>10.00</vNF></ICMSTot></total><transp><modFrete>9</modFrete></transp><pag><detPag><indPag>1</indPag><tPag>03</tPag><vPag>10.00</vPag></detPag><vTroco>0.00</vTroco></pag><infAdic/><infRespTec><CNPJ>00000000000191</CNPJ><xContato>Joao Silva</xContato><email>teste@exemplo.com.br</email><fone>11999999999</fone></infRespTec></infNFe><infNFeSupl><qrCode>https://www.nfce.fazenda.sp.gov.br/qrcode?p=35241100000000000191650010000000011000000001|2|2|1|AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA</qrCode><urlChave>https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica</urlChave></infNFeSupl><Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><SignedInfo><CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/><SignatureMethod Algorithm="http://www.w3.org/2000/09/xmldsig#rsa-sha1"/><Reference URI="#NFe35241100000000000191650010000000011000000001"><Transforms><Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/><Transform Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/></Transforms><DigestMethod Algorithm="http://www.w3.org/2000/09/xmldsig#sha1"/><DigestValue>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</DigestValue></Reference></SignedInfo><SignatureValue>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==</SignatureValue><KeyInfo><X509Data><X509Certificate>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==</X509Certificate></X509Data></KeyInfo></Signature></NFe><protNFe versao="4.00"><infProt><tpAmb>2</tpAmb><verAplic>SP_NFCE_PL_009_V400</verAplic><chNFe>35241100000000000191650010000000011000000001</chNFe><dhRecbto>2024-11-18T14:01:13-03:00</dhRecbto><nProt>000000000000000</nProt><digVal>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</digVal><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe></nfeProc>"##;
    let _xml_test_55 = r##"<?xml version="1.0" encoding="UTF-8"?><nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe xmlns="http://www.portalfiscal.inf.br/nfe" Id="NFe35260300000000000191550010000005041000000001" versao="4.00"><ide><cUF>35</cUF><cNF>85265620</cNF><natOp>VENDA</natOp><mod>55</mod><serie>1</serie><nNF>504</nNF><dhEmi>2026-03-05T11:55:39-03:00</dhEmi><dhSaiEnt>2026-03-05T11:55:39-03:00</dhSaiEnt><tpNF>1</tpNF><idDest>1</idDest><cMunFG>3529906</cMunFG><tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>1</cDV><tpAmb>1</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal><indPres>1</indPres><procEmi>0</procEmi><verProc>1.0.0</verProc></ide><emit><CNPJ>00000000000191</CNPJ><xNome>EMPRESA FICTICIA LTDA</xNome><xFant>LOJA EXEMPLO</xFant><enderEmit><xLgr>RUA DAS FLORES</xLgr><nro>100</nro><xBairro>CENTRO</xBairro><cMun>3529906</cMun><xMun>Miracatu</xMun><UF>SP</UF><CEP>11850000</CEP><cPais>1058</cPais><xPais>Brasil</xPais></enderEmit><IE>000000000000</IE><CRT>3</CRT></emit><dest><CNPJ>11222333000181</CNPJ><xNome>EMPRESA EXEMPLO DE EMPRESA</xNome><enderDest><xLgr>AV BRASIL</xLgr><nro>500</nro><xBairro>JARDIM ESPERANCA</xBairro><cMun>3529906</cMun><xMun>Miracatu</xMun><UF>SP</UF><CEP>11850000</CEP><cPais>1058</cPais><xPais>Brasil</xPais></enderDest><indIEDest>9</indIEDest></dest><det nItem="1"><prod><cProd>2</cProd><cEAN>SEM GTIN</cEAN><xProd>PRODUTO TESTE UNITARIO</xProd><NCM>21069090</NCM><CFOP>5101</CFOP><uCom>UN</uCom><qCom>20.000</qCom><vUnCom>45.00</vUnCom><vProd>900.00</vProd><cEANTrib>SEM GTIN</cEANTrib><uTrib>UN</uTrib><qTrib>20.000</qTrib><vUnTrib>45.00</vUnTrib><indTot>1</indTot></prod><imposto><vTotTrib>72.00</vTotTrib><ICMS><ICMS00><orig>0</orig><CST>00</CST><modBC>3</modBC><vBC>900.00</vBC><pICMS>8.0000</pICMS><vICMS>72.00</vICMS></ICMS00></ICMS><PIS><PISOutr><CST>99</CST><qBCProd>0.00</qBCProd><vAliqProd>0.00</vAliqProd><vPIS>0.00</vPIS></PISOutr></PIS><COFINS><COFINSOutr><CST>99</CST><vBC>0.00</vBC><pCOFINS>0.00</pCOFINS><vCOFINS>0.00</vCOFINS></COFINSOutr></COFINS><IBSCBS><CST>000</CST><cClassTrib>000001</cClassTrib><gIBSCBS><vBC>828.00</vBC><gIBSUF><pIBSUF>0.1000</pIBSUF><vIBSUF>0.83</vIBSUF></gIBSUF><gIBSMun><pIBSMun>0.0000</pIBSMun><vIBSMun>0.00</vIBSMun></gIBSMun><vIBS>0.83</vIBS><gCBS><pCBS>0.9000</pCBS><vCBS>7.45</vCBS></gCBS></gIBSCBS></IBSCBS></imposto></det><total><ICMSTot><vBC>900.00</vBC><vICMS>72.00</vICMS><vICMSDeson>0.00</vICMSDeson><vFCPUFDest>0.00</vFCPUFDest><vICMSUFDest>0.00</vICMSUFDest><vICMSUFRemet>0.00</vICMSUFRemet><vFCP>0.00</vFCP><vBCST>0.00</vBCST><vST>0.00</vST><vFCPST>0.00</vFCPST><vFCPSTRet>0.00</vFCPSTRet><vProd>900.00</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg><vDesc>0.00</vDesc><vII>0.00</vII><vIPI>0.00</vIPI><vIPIDevol>0.00</vIPIDevol><vPIS>0.00</vPIS><vCOFINS>0.00</vCOFINS><vOutro>0.00</vOutro><vNF>900.00</vNF><vTotTrib>72.00</vTotTrib></ICMSTot><IBSCBSTot><vBCIBSCBS>828.00</vBCIBSCBS><gIBS><gIBSUF><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vIBSUF>0.83</vIBSUF></gIBSUF><gIBSMun><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vIBSMun>0.00</vIBSMun></gIBSMun><vIBS>0.83</vIBS><vCredPres>0.00</vCredPres><vCredPresCondSus>0.00</vCredPresCondSus></gIBS><gCBS><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vCBS>7.45</vCBS><vCredPres>0.00</vCredPres><vCredPresCondSus>0.00</vCredPresCondSus></gCBS></IBSCBSTot></total><transp><modFrete>9</modFrete></transp><pag><detPag><indPag>0</indPag><tPag>01</tPag><vPag>900.00</vPag></detPag></pag><infAdic><infCpl>INFORMACOES COMPLEMENTARES DE TESTE</infCpl></infAdic></infNFe><Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><SignedInfo xmlns="http://www.w3.org/2000/09/xmldsig#"><CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/><SignatureMethod Algorithm="http://www.w3.org/2000/09/xmldsig#rsa-sha1"/><Reference URI="#NFe35260300000000000191550010000005041000000001"><Transforms><Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/><Transform Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/></Transforms><DigestMethod Algorithm="http://www.w3.org/2000/09/xmldsig#sha1"/><DigestValue>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</DigestValue></Reference></SignedInfo><SignatureValue>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==</SignatureValue><KeyInfo><X509Data><X509Certificate>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==</X509Certificate></X509Data></KeyInfo></Signature></NFe><protNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><infProt><tpAmb>1</tpAmb><verAplic>SP_NFE_PL009_V4</verAplic><chNFe>35260300000000000191550010000005041000000001</chNFe><dhRecbto>2026-03-05T11:55:41-03:00</dhRecbto><nProt>000000000000000</nProt><digVal>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</digVal><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe></nfeProc>"##;

    match DanfeBuilder::new()
        .xml(_xml_test_55)
        .paper_size("80mm")
        .as_base64()
        .build()
        .await
    {
        Ok(output) => {
            println!("{}", output);
            // assert!(output.len() > 0); // Verifica se a string base64 não está vazia
            // assert!(output.starts_with("JVBERi0")); // Verifica se a string base64 começa com a assinatura de um PDF
        }
        Err(e) => {
            println!("Erro:{}", e);
            assert!(false); // Falha no teste se ocorrer um erro
        }
    }
}
