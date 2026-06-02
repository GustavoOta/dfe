mod test_danfe_nfe_a4;
mod test_danfe_nfce;
mod test_xml_extractor;

use dfe::tipos::emissao::Dest;
use dfe::tipos::{Cofins, Det, Emit, Icms, Ide, Pag, Pis, Total, Transp};
use dfe::CancelarBuilder;
use dfe::DanfeBuilder;
use dfe::NFeBuilder;

#[tokio::test]
async fn test_service_status() {
    use dfe::status::NFeService;

    let r = NFeService::new()
        .cert_path("D:/Projetos/cert.pfx")
        .cert_pass("1234")
        .uf("SP")
        .environment(2)
        .send()
        .await;

    match r {
        Err(e) => println!("Erro test_service_status: {:?}", e),
        Ok(r) => {
            println!("c_stat: {}", r.c_stat);
            println!("x_motivo: {}", r.x_motivo);
            println!("url: {}", r.url);
        }
    }
}

#[tokio::test]
async fn test_emit_nfe() {
    // Identificação da NF-e
    // mod_: 55 = NF-e | 65 = NFC-e
    // tp_amb: 1 = Produção | 2 = Homologação
    // tp_emis: 1 = Normal | 5 = Contingência EPEC | 7 = Contingência SVC-RS | 8 = Contingência SVC-SP
    // tp_imp: 1 = DANFE Normal Retrato | 2 = DANFE Normal Paisagem | 4 = DANFE NFC-e
    // ind_final: 0 = Normal | 1 = Consumidor Final
    // ind_pres: 1 = Operação presencial | 2 = Não presencial / Internet | 9 = Outros
    let ide = Ide {
        c_uf: 35, // 35 = São Paulo
        mod_: 55,
        serie: 1,
        n_nf: 3,
        c_mun_fg: "3507605".to_string(), // Código IBGE do município do emitente
        tp_nf: 1,                        // 0 = Entrada | 1 = Saída
        tp_emis: 1,
        tp_amb: 2,
        ind_final: 1,
        ind_pres: 1,
        tp_imp: 1,
        ..Default::default()
    };

    // Dados do emitente
    // crt: 1 = Simples Nacional | 2 = Simples Nacional — excesso | 3 = Regime Normal
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

    // Dados do destinatário
    // ind_ie_dest: 1 = Contribuinte ICMS | 2 = Contribuinte isento | 9 = Não contribuinte
    // Para CPF: deixar ie = None e ind_ie_dest = Some(9)
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

    // Itens da nota
    // icms: usar a variante correspondente ao regime tributário do emitente e da operação
    //   Regime Normal (CRT=3): Icms00, Icms40, Icms60, Icms90
    //   Simples Nacional (CRT=1): Sn101, Sn102, Sn500, Sn900
    // pis / cofins: Aliq (CST 01/02), Outr (CST 99), Nt (CST 04-09)
    // cfop: 5xxx = operação interna saída | 6xxx = interestadual saída
    // x_prod do primeiro item: em homologação (tp_amb=2) é substituído automaticamente
    //   pelo texto obrigatório da SEFAZ — não é necessário informar o valor correto aqui
    let itens = vec![
        Det {
            c_prod: "001".to_string(),
            x_prod: "PRODUTO TESTE 1".to_string(), // sobrescrito em homologação
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

    // Totais calculados automaticamente dos itens (v_bc, v_icms, v_prod, v_pis, v_cofins, v_nf).
    // Informar apenas o que não deriva dos itens: frete, seguro, ST, FCP, etc.
    // Para uma venda simples sem frete/seguro, Total::default() é suficiente.
    let total = Total::default();

    // Transporte
    // mod_frete: 0 = Emitente | 1 = Destinatário | 9 = Sem frete
    let transporte = Transp {
        mod_frete: 9,
        ..Default::default()
    };

    // Pagamento
    // ind_pag: 0 = À vista | 1 = A prazo
    // t_pag: "01" = Dinheiro | "02" = Cheque | "03" = Cartão Crédito | "04" = Cartão Débito | "99" = Outros
    let pagamento = Pag {
        ind_pag: 0,
        t_pag: "01".to_string(),
        v_pag: 30.0,
        ..Default::default()
    };

    let resultado = NFeBuilder::new()
        .cert("D:/Projetos/cert.pfx", "1234")
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
        Err(e) => println!("Erro: {:?}", e),
        Ok(response) => {
            println!("c_stat: {}", response.protocolo.inf_prot.c_stat);
            println!("x_motivo: {}", response.protocolo.inf_prot.x_motivo);
            std::fs::write("nfe_autorizada.xml", &response.xml).expect("Falha ao salvar o XML");
        }
    }
}

#[tokio::test]
async fn test_emit_nfce() {
    // NFC-e — modelo 65 (Nota Fiscal do Consumidor Eletrônico)
    //
    // Diferenças em relação à NF-e (modelo 55):
    //   mod_: 65 | tp_imp: 4 (DANFE NFC-e)
    //   id_csc + csc obrigatórios (registrados na SEFAZ por CNPJ)
    //   dh_sai_ent não é enviada (removida automaticamente)
    //   QR Code gerado automaticamente no XML
    //   Destinatário opcional (consumidor anônimo não exige identificação)
    //
    // ATENÇÃO: url do QR Code atualmente hardcoded para SP — ver emissao/mod.rs
    // CSC de homologação SP: registrar em https://www.nfce.fazenda.sp.gov.br/

    let ide = Ide {
        c_uf: 35,
        mod_: 65, // NFC-e
        serie: 1,
        n_nf: 1,
        c_mun_fg: "3507605".to_string(),
        tp_nf: 1, // 1 = Saída
        tp_emis: 1,
        tp_amb: 2,
        ind_final: 1, // 1 = Consumidor Final
        ind_pres: 1,  // 1 = Operação presencial
        tp_imp: 4,    // 4 = DANFE NFC-e
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

    // Destinatário opcional para NFC-e — omitir para consumidor anônimo.
    // Para identificar: informar cpf (pessoa física) ou cnpj (empresa).
    // let destinatario = Dest { cpf: Some("00000000000".to_string()), ..Default::default() };

    // Itens — x_prod do primeiro sobrescrito em homologação
    // CFOP 5102 = venda de mercadoria adquirida para comercialização (interna)
    // PIS/COFINS Outr (CST 99) é o mais comum em NFC-e de varejo
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

    // Pagamento — para NFC-e deve refletir o meio de pagamento real do consumidor
    // t_pag: "01" = Dinheiro | "03" = Cartão de Crédito | "04" = Cartão de Débito
    let pagamento = Pag {
        ind_pag: 0,
        t_pag: "01".to_string(),
        v_pag: 30.0,
        ..Default::default()
    };

    let resultado = NFeBuilder::new()
        .cert("D:/Projetos/cert.pfx", "1234")
        .ide(ide)
        .emitente(emitente)
        // .destinatario(destinatario)
        .itens(itens)
        .total(Total::default())
        .transporte(Transp {
            mod_frete: 9,
            ..Default::default()
        })
        .pagamento(pagamento)
        .id_csc("000001") // ID do CSC registrado na SEFAZ (6 dígitos)
        .csc("CODIGO_CSC_AQUI") // Código CSC de homologação (registrar no portal SP)
        .emitir()
        .await;

    match resultado {
        Err(e) => println!("Erro: {:?}", e),
        Ok(response) => {
            println!("c_stat: {}", response.protocolo.inf_prot.c_stat);
            println!("x_motivo: {}", response.protocolo.inf_prot.x_motivo);
            std::fs::write("nfce_autorizada.xml", &response.xml).expect("Falha ao salvar o XML");
        }
    }
}

#[tokio::test]
async fn test_cancelar_nfe() {
    let resultado = CancelarBuilder::new()
        .cert("D:/Projetos/cert.pfx", "1234")
        .tp_amb(2)
        .chave("35000000000000000000550010000000001000000001")
        .protocolo("135000000000001")
        .justificativa("Cancelamento de teste em homologacao")
        .send()
        .await;

    match resultado {
        Err(e) => println!("Erro: {:?}", e),
        Ok(r) => {
            println!("c_stat: {}", r.response.c_stat);
            println!("x_motivo: {}", r.response.x_motivo);
        }
    }
}

#[tokio::test]
async fn test_danfe_arquivo() {
    match DanfeBuilder::new()
        .xml("./sample55.xml")
        .paper_size("80mm")
        .as_file("./danfe_output.pdf")
        .build()
        .await
    {
        Ok(path) => println!("DANFE gerada: {}", path),
        Err(e) => println!("Erro: {}", e),
    }
}

#[tokio::test]
async fn test_danfe_base64() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?><nfeProc xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><NFe xmlns="http://www.portalfiscal.inf.br/nfe"><infNFe xmlns="http://www.portalfiscal.inf.br/nfe" Id="NFe35260300000000000191550010000005041000000001" versao="4.00"><ide><cUF>35</cUF><cNF>85265620</cNF><natOp>VENDA</natOp><mod>55</mod><serie>1</serie><nNF>504</nNF><dhEmi>2026-03-05T11:55:39-03:00</dhEmi><dhSaiEnt>2026-03-05T11:55:39-03:00</dhSaiEnt><tpNF>1</tpNF><idDest>1</idDest><cMunFG>3529906</cMunFG><tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>1</cDV><tpAmb>1</tpAmb><finNFe>1</finNFe><indFinal>1</indFinal><indPres>1</indPres><procEmi>0</procEmi><verProc>1.0.0</verProc></ide><emit><CNPJ>00000000000191</CNPJ><xNome>EMPRESA FICTICIA LTDA</xNome><xFant>LOJA EXEMPLO</xFant><enderEmit><xLgr>RUA DAS FLORES</xLgr><nro>100</nro><xBairro>CENTRO</xBairro><cMun>3529906</cMun><xMun>Miracatu</xMun><UF>SP</UF><CEP>11850000</CEP><cPais>1058</cPais><xPais>Brasil</xPais></enderEmit><IE>000000000000</IE><CRT>3</CRT></emit><dest><CNPJ>11222333000181</CNPJ><xNome>EMPRESA EXEMPLO</xNome><enderDest><xLgr>AV BRASIL</xLgr><nro>500</nro><xBairro>JARDIM ESPERANCA</xBairro><cMun>3529906</cMun><xMun>Miracatu</xMun><UF>SP</UF><CEP>11850000</CEP><cPais>1058</cPais><xPais>Brasil</xPais></enderDest><indIEDest>9</indIEDest></dest><det nItem="1"><prod><cProd>2</cProd><cEAN>SEM GTIN</cEAN><xProd>PRODUTO TESTE UNITARIO</xProd><NCM>21069090</NCM><CFOP>5101</CFOP><uCom>UN</uCom><qCom>20.000</qCom><vUnCom>45.00</vUnCom><vProd>900.00</vProd><cEANTrib>SEM GTIN</cEANTrib><uTrib>UN</uTrib><qTrib>20.000</qTrib><vUnTrib>45.00</vUnTrib><indTot>1</indTot></prod><imposto><vTotTrib>72.00</vTotTrib><ICMS><ICMS00><orig>0</orig><CST>00</CST><modBC>3</modBC><vBC>900.00</vBC><pICMS>8.0000</pICMS><vICMS>72.00</vICMS></ICMS00></ICMS><PIS><PISOutr><CST>99</CST><qBCProd>0.00</qBCProd><vAliqProd>0.00</vAliqProd><vPIS>0.00</vPIS></PISOutr></PIS><COFINS><COFINSOutr><CST>99</CST><vBC>0.00</vBC><pCOFINS>0.00</pCOFINS><vCOFINS>0.00</vCOFINS></COFINSOutr></COFINS></imposto></det><total><ICMSTot><vBC>900.00</vBC><vICMS>72.00</vICMS><vICMSDeson>0.00</vICMSDeson><vFCPUFDest>0.00</vFCPUFDest><vICMSUFDest>0.00</vICMSUFDest><vICMSUFRemet>0.00</vICMSUFRemet><vFCP>0.00</vFCP><vBCST>0.00</vBCST><vST>0.00</vST><vFCPST>0.00</vFCPST><vFCPSTRet>0.00</vFCPSTRet><vProd>900.00</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg><vDesc>0.00</vDesc><vII>0.00</vII><vIPI>0.00</vIPI><vIPIDevol>0.00</vIPIDevol><vPIS>0.00</vPIS><vCOFINS>0.00</vCOFINS><vOutro>0.00</vOutro><vNF>900.00</vNF><vTotTrib>72.00</vTotTrib></ICMSTot></total><transp><modFrete>9</modFrete></transp><pag><detPag><indPag>0</indPag><tPag>01</tPag><vPag>900.00</vPag></detPag></pag><infAdic><infCpl>INFORMACOES COMPLEMENTARES DE TESTE</infCpl></infAdic></infNFe><Signature xmlns="http://www.w3.org/2000/09/xmldsig#"><SignedInfo xmlns="http://www.w3.org/2000/09/xmldsig#"><CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/><SignatureMethod Algorithm="http://www.w3.org/2000/09/xmldsig#rsa-sha1"/><Reference URI="#NFe35260300000000000191550010000005041000000001"><Transforms><Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/><Transform Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/></Transforms><DigestMethod Algorithm="http://www.w3.org/2000/09/xmldsig#sha1"/><DigestValue>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</DigestValue></Reference></SignedInfo><SignatureValue>AAAAAAAAAAAAA==</SignatureValue><KeyInfo><X509Data><X509Certificate>AAAAAAAAAA==</X509Certificate></X509Data></KeyInfo></Signature></NFe><protNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><infProt><tpAmb>1</tpAmb><verAplic>SP_NFE_PL009_V4</verAplic><chNFe>35260300000000000191550010000005041000000001</chNFe><dhRecbto>2026-03-05T11:55:41-03:00</dhRecbto><nProt>000000000000000</nProt><digVal>AAAAAAAAAAAAAAAAAAAAAAAAAAAA</digVal><cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe></nfeProc>"##;

    match DanfeBuilder::new()
        .xml(xml)
        .paper_size("80mm")
        .as_base64()
        .build()
        .await
    {
        Ok(b64) => println!("DANFE base64 gerada ({} chars)", b64.len()),
        Err(e) => panic!("Falha ao gerar DANFE base64: {}", e),
    }
}
