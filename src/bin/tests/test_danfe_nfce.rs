#[cfg(test)]
use dfe::DanfeBuilder;

/// XML mínimo de NFC-e (modelo 65) válido para testes de geração de DANFE.
/// Os dados são fictícios — apenas para exercitar o builder sem acesso à SEFAZ.
const NFCE_XML_HOMOLOG: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
  <NFe xmlns="http://www.portalfiscal.inf.br/nfe">
    <infNFe versao="4.00" Id="NFe35240100000000000000650010000000011000000011">
      <ide>
        <cUF>35</cUF>
        <cNF>10000001</cNF>
        <natOp>VENDA</natOp>
        <mod>65</mod>
        <serie>1</serie>
        <nNF>1</nNF>
        <dhEmi>2024-01-15T14:30:00-03:00</dhEmi>
        <tpNF>1</tpNF>
        <idDest>1</idDest>
        <cMunFG>3550308</cMunFG>
        <tpImp>4</tpImp>
        <tpEmis>1</tpEmis>
        <cDV>1</cDV>
        <tpAmb>2</tpAmb>
        <finNFe>1</finNFe>
        <indFinal>1</indFinal>
        <indPres>1</indPres>
        <procEmi>0</procEmi>
        <verProc>1.0</verProc>
      </ide>
      <emit>
        <CNPJ>11222333000181</CNPJ>
        <xNome>LOJA DE TESTE LTDA</xNome>
        <xFant>LOJA TESTE</xFant>
        <enderEmit>
          <xLgr>RUA DAS FLORES</xLgr>
          <nro>100</nro>
          <xBairro>CENTRO</xBairro>
          <cMun>3550308</cMun>
          <xMun>SAO PAULO</xMun>
          <UF>SP</UF>
          <CEP>01310100</CEP>
          <cPais>1058</cPais>
          <xPais>BRASIL</xPais>
          <fone>1133334444</fone>
        </enderEmit>
        <IE>111111111111</IE>
        <CRT>1</CRT>
      </emit>
      <det nItem="1">
        <prod>
          <cProd>001</cProd>
          <cEAN>SEM GTIN</cEAN>
          <xProd>AGUA MINERAL 500ML</xProd>
          <NCM>22011000</NCM>
          <CFOP>5102</CFOP>
          <uCom>UN</uCom>
          <qCom>2.0000</qCom>
          <vUnCom>3.5000</vUnCom>
          <vProd>7.00</vProd>
          <cEANTrib>SEM GTIN</cEANTrib>
          <uTrib>UN</uTrib>
          <qTrib>2.0000</qTrib>
          <vUnTrib>3.5000</vUnTrib>
          <indTot>1</indTot>
        </prod>
        <imposto>
          <ICMS>
            <ICMSSN102>
              <orig>0</orig>
              <CSOSN>400</CSOSN>
            </ICMSSN102>
          </ICMS>
          <PIS>
            <PISAliq>
              <CST>01</CST>
              <vBC>7.00</vBC>
              <pPIS>0.65</pPIS>
              <vPIS>0.05</vPIS>
            </PISAliq>
          </PIS>
          <COFINS>
            <COFINSAliq>
              <CST>01</CST>
              <vBC>7.00</vBC>
              <pCOFINS>3.00</pCOFINS>
              <vCOFINS>0.21</vCOFINS>
            </COFINSAliq>
          </COFINS>
        </imposto>
      </det>
      <det nItem="2">
        <prod>
          <cProd>002</cProd>
          <cEAN>SEM GTIN</cEAN>
          <xProd>REFRIGERANTE LATA 350ML</xProd>
          <NCM>22021000</NCM>
          <CFOP>5102</CFOP>
          <uCom>UN</uCom>
          <qCom>1.0000</qCom>
          <vUnCom>5.0000</vUnCom>
          <vProd>5.00</vProd>
          <cEANTrib>SEM GTIN</cEANTrib>
          <uTrib>UN</uTrib>
          <qTrib>1.0000</qTrib>
          <vUnTrib>5.0000</vUnTrib>
          <indTot>1</indTot>
        </prod>
        <imposto>
          <ICMS>
            <ICMSSN102>
              <orig>0</orig>
              <CSOSN>400</CSOSN>
            </ICMSSN102>
          </ICMS>
          <PIS>
            <PISNT>
              <CST>07</CST>
            </PISNT>
          </PIS>
          <COFINS>
            <COFINSNT>
              <CST>07</CST>
            </COFINSNT>
          </COFINS>
        </imposto>
      </det>
      <total>
        <ICMSTot>
          <vBC>7.00</vBC>
          <vICMS>0.00</vICMS>
          <vICMSDeson>0.00</vICMSDeson>
          <vFCP>0.00</vFCP>
          <vBCST>0.00</vBCST>
          <vST>0.00</vST>
          <vFCPST>0.00</vFCPST>
          <vFCPSTRet>0.00</vFCPSTRet>
          <vProd>12.00</vProd>
          <vFrete>0.00</vFrete>
          <vSeg>0.00</vSeg>
          <vDesc>0.00</vDesc>
          <vII>0.00</vII>
          <vIPI>0.00</vIPI>
          <vIPIDevol>0.00</vIPIDevol>
          <vPIS>0.05</vPIS>
          <vCOFINS>0.21</vCOFINS>
          <vOutro>0.00</vOutro>
          <vNF>12.00</vNF>
          <vTotTrib>1.50</vTotTrib>
        </ICMSTot>
      </total>
      <transp>
        <modFrete>9</modFrete>
      </transp>
      <pag>
        <detPag>
          <indPag>0</indPag>
          <tPag>01</tPag>
          <vPag>15.00</vPag>
        </detPag>
        <vTroco>3.00</vTroco>
      </pag>
      <infAdic>
        <infCpl>OBRIGADO PELA PREFERENCIA</infCpl>
      </infAdic>
    </infNFe>
    <infNFeSupl>
      <qrCode>https://www.nfce.fazenda.sp.gov.br/qrcode?chNFe=35240100000000000000650010000000011000000011&amp;nVersao=100&amp;tpAmb=2&amp;dhEmi=323032342d30312d31355431343a33303a30302d30333a3030&amp;vNF=12.00&amp;vICMS=0.00&amp;digVal=abc123&amp;cIdToken=000001&amp;cHashQRCode=ABC123DEF456</qrCode>
      <urlChave>https://www.nfce.fazenda.sp.gov.br/consulta</urlChave>
    </infNFeSupl>
  </NFe>
  <protNFe versao="4.00">
    <infProt>
      <tpAmb>2</tpAmb>
      <verAplic>SP_NFCe_v4.0</verAplic>
      <chNFe>35240100000000000000650010000000011000000011</chNFe>
      <dhRecbto>2024-01-15T14:30:05-03:00</dhRecbto>
      <nProt>135240000000001</nProt>
      <digVal>abc123def456=</digVal>
      <cStat>100</cStat>
      <xMotivo>Autorizado o uso da NF-e</xMotivo>
    </infProt>
  </protNFe>
</nfeProc>"#;

/// XML NFC-e com múltiplas formas de pagamento (PIX + dinheiro) e sem destinatário.
const NFCE_XML_MULTI_PAG: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
  <NFe xmlns="http://www.portalfiscal.inf.br/nfe">
    <infNFe versao="4.00" Id="NFe35240100000000000000650010000000021000000021">
      <ide>
        <cUF>35</cUF>
        <cNF>10000002</cNF>
        <natOp>VENDA</natOp>
        <mod>65</mod>
        <serie>1</serie>
        <nNF>2</nNF>
        <dhEmi>2024-01-16T09:00:00-03:00</dhEmi>
        <tpNF>1</tpNF>
        <idDest>1</idDest>
        <cMunFG>3550308</cMunFG>
        <tpImp>4</tpImp>
        <tpEmis>1</tpEmis>
        <cDV>1</cDV>
        <tpAmb>2</tpAmb>
        <finNFe>1</finNFe>
        <indFinal>1</indFinal>
        <indPres>1</indPres>
        <procEmi>0</procEmi>
        <verProc>1.0</verProc>
      </ide>
      <emit>
        <CNPJ>11222333000181</CNPJ>
        <xNome>MERCADO CENTRAL LTDA</xNome>
        <enderEmit>
          <xLgr>AV PAULISTA</xLgr>
          <nro>2000</nro>
          <xBairro>BELA VISTA</xBairro>
          <cMun>3550308</cMun>
          <xMun>SAO PAULO</xMun>
          <UF>SP</UF>
          <CEP>01310100</CEP>
          <cPais>1058</cPais>
          <xPais>BRASIL</xPais>
        </enderEmit>
        <IE>111111111111</IE>
        <CRT>3</CRT>
      </emit>
      <det nItem="1">
        <prod>
          <cProd>003</cProd>
          <cEAN>SEM GTIN</cEAN>
          <xProd>PRODUTO COM NOME MUITO LONGO QUE DEVE SER TRUNCADO NO PDF</xProd>
          <NCM>22030000</NCM>
          <CFOP>5102</CFOP>
          <uCom>KG</uCom>
          <qCom>0.500</qCom>
          <vUnCom>20.0000</vUnCom>
          <vProd>10.00</vProd>
          <cEANTrib>SEM GTIN</cEANTrib>
          <uTrib>KG</uTrib>
          <qTrib>0.500</qTrib>
          <vUnTrib>20.0000</vUnTrib>
          <indTot>1</indTot>
        </prod>
        <imposto>
          <ICMS>
            <ICMS00>
              <orig>0</orig>
              <modBC>3</modBC>
              <vBC>10.00</vBC>
              <pICMS>12.00</pICMS>
              <vICMS>1.20</vICMS>
            </ICMS00>
          </ICMS>
          <PIS>
            <PISAliq>
              <CST>01</CST>
              <vBC>10.00</vBC>
              <pPIS>0.65</pPIS>
              <vPIS>0.07</vPIS>
            </PISAliq>
          </PIS>
          <COFINS>
            <COFINSAliq>
              <CST>01</CST>
              <vBC>10.00</vBC>
              <pCOFINS>3.00</pCOFINS>
              <vCOFINS>0.30</vCOFINS>
            </COFINSAliq>
          </COFINS>
        </imposto>
      </det>
      <total>
        <ICMSTot>
          <vBC>10.00</vBC>
          <vICMS>1.20</vICMS>
          <vICMSDeson>0.00</vICMSDeson>
          <vFCP>0.00</vFCP>
          <vBCST>0.00</vBCST>
          <vST>0.00</vST>
          <vFCPST>0.00</vFCPST>
          <vFCPSTRet>0.00</vFCPSTRet>
          <vProd>10.00</vProd>
          <vFrete>0.00</vFrete>
          <vSeg>0.00</vSeg>
          <vDesc>0.00</vDesc>
          <vII>0.00</vII>
          <vIPI>0.00</vIPI>
          <vIPIDevol>0.00</vIPIDevol>
          <vPIS>0.07</vPIS>
          <vCOFINS>0.30</vCOFINS>
          <vOutro>0.00</vOutro>
          <vNF>10.00</vNF>
          <vTotTrib>0.00</vTotTrib>
        </ICMSTot>
      </total>
      <transp>
        <modFrete>9</modFrete>
      </transp>
      <pag>
        <detPag>
          <indPag>0</indPag>
          <tPag>17</tPag>
          <vPag>8.00</vPag>
        </detPag>
        <detPag>
          <indPag>0</indPag>
          <tPag>01</tPag>
          <vPag>2.00</vPag>
        </detPag>
        <vTroco>0.00</vTroco>
      </pag>
    </infNFe>
    <infNFeSupl>
      <qrCode>https://www.nfce.fazenda.sp.gov.br/qrcode?chNFe=35240100000000000000650010000000021000000021</qrCode>
      <urlChave>https://www.nfce.fazenda.sp.gov.br/consulta</urlChave>
    </infNFeSupl>
  </NFe>
  <protNFe versao="4.00">
    <infProt>
      <tpAmb>2</tpAmb>
      <verAplic>SP_NFCe_v4.0</verAplic>
      <chNFe>35240100000000000000650010000000021000000021</chNFe>
      <dhRecbto>2024-01-16T09:00:05-03:00</dhRecbto>
      <nProt>135240000000002</nProt>
      <digVal>xyz789=</digVal>
      <cStat>100</cStat>
      <xMotivo>Autorizado o uso da NF-e</xMotivo>
    </infProt>
  </protNFe>
</nfeProc>"#;

/// XML NFC-e com destinatário identificado por CPF.
const NFCE_XML_COM_CPF: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
  <NFe xmlns="http://www.portalfiscal.inf.br/nfe">
    <infNFe versao="4.00" Id="NFe35240100000000000000650010000000031000000031">
      <ide>
        <cUF>35</cUF>
        <cNF>10000003</cNF>
        <natOp>VENDA</natOp>
        <mod>65</mod>
        <serie>1</serie>
        <nNF>3</nNF>
        <dhEmi>2024-01-17T11:00:00-03:00</dhEmi>
        <tpNF>1</tpNF>
        <idDest>1</idDest>
        <cMunFG>3550308</cMunFG>
        <tpImp>4</tpImp>
        <tpEmis>1</tpEmis>
        <cDV>1</cDV>
        <tpAmb>2</tpAmb>
        <finNFe>1</finNFe>
        <indFinal>1</indFinal>
        <indPres>1</indPres>
        <procEmi>0</procEmi>
        <verProc>1.0</verProc>
      </ide>
      <emit>
        <CNPJ>11222333000181</CNPJ>
        <xNome>PADARIA BOA VISTA</xNome>
        <enderEmit>
          <xLgr>RUA DO PÃO</xLgr>
          <nro>5</nro>
          <xBairro>JARDIM</xBairro>
          <cMun>3550308</cMun>
          <xMun>SAO PAULO</xMun>
          <UF>SP</UF>
          <CEP>01001000</CEP>
          <cPais>1058</cPais>
          <xPais>BRASIL</xPais>
        </enderEmit>
        <IE>222222222222</IE>
        <CRT>1</CRT>
      </emit>
      <dest>
        <CPF>07068093868</CPF>
        <xNome>JOAO DA SILVA</xNome>
        <indIEDest>9</indIEDest>
      </dest>
      <det nItem="1">
        <prod>
          <cProd>PAO001</cProd>
          <cEAN>SEM GTIN</cEAN>
          <xProd>PAO FRANCES</xProd>
          <NCM>19053100</NCM>
          <CFOP>5102</CFOP>
          <uCom>KG</uCom>
          <qCom>0.300</qCom>
          <vUnCom>8.0000</vUnCom>
          <vProd>2.40</vProd>
          <cEANTrib>SEM GTIN</cEANTrib>
          <uTrib>KG</uTrib>
          <qTrib>0.300</qTrib>
          <vUnTrib>8.0000</vUnTrib>
          <indTot>1</indTot>
        </prod>
        <imposto>
          <ICMS>
            <ICMSSN102>
              <orig>0</orig>
              <CSOSN>400</CSOSN>
            </ICMSSN102>
          </ICMS>
          <PIS>
            <PISNT>
              <CST>07</CST>
            </PISNT>
          </PIS>
          <COFINS>
            <COFINSNT>
              <CST>07</CST>
            </COFINSNT>
          </COFINS>
        </imposto>
      </det>
      <total>
        <ICMSTot>
          <vBC>0.00</vBC>
          <vICMS>0.00</vICMS>
          <vICMSDeson>0.00</vICMSDeson>
          <vFCP>0.00</vFCP>
          <vBCST>0.00</vBCST>
          <vST>0.00</vST>
          <vFCPST>0.00</vFCPST>
          <vFCPSTRet>0.00</vFCPSTRet>
          <vProd>2.40</vProd>
          <vFrete>0.00</vFrete>
          <vSeg>0.00</vSeg>
          <vDesc>0.00</vDesc>
          <vII>0.00</vII>
          <vIPI>0.00</vIPI>
          <vIPIDevol>0.00</vIPIDevol>
          <vPIS>0.00</vPIS>
          <vCOFINS>0.00</vCOFINS>
          <vOutro>0.00</vOutro>
          <vNF>2.40</vNF>
          <vTotTrib>0.00</vTotTrib>
        </ICMSTot>
      </total>
      <transp>
        <modFrete>9</modFrete>
      </transp>
      <pag>
        <detPag>
          <indPag>0</indPag>
          <tPag>04</tPag>
          <vPag>2.40</vPag>
        </detPag>
      </pag>
    </infNFe>
    <infNFeSupl>
      <qrCode>https://www.nfce.fazenda.sp.gov.br/qrcode?chNFe=35240100000000000000650010000000031000000031</qrCode>
      <urlChave>https://www.nfce.fazenda.sp.gov.br/consulta</urlChave>
    </infNFeSupl>
  </NFe>
  <protNFe versao="4.00">
    <infProt>
      <tpAmb>2</tpAmb>
      <verAplic>SP_NFCe_v4.0</verAplic>
      <chNFe>35240100000000000000650010000000031000000031</chNFe>
      <dhRecbto>2024-01-17T11:00:05-03:00</dhRecbto>
      <nProt>135240000000003</nProt>
      <digVal>pqr789=</digVal>
      <cStat>100</cStat>
      <xMotivo>Autorizado o uso da NF-e</xMotivo>
    </infProt>
  </protNFe>
</nfeProc>"#;

// ── Testes ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_danfe_nfce_80mm_homologacao_as_base64() {
    let result = DanfeBuilder::new()
        .xml(NFCE_XML_HOMOLOG)
        .paper_size("80mm")
        .as_base64()
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao gerar DANFE NFC-e (homologação): {}", e),
        Ok(b64) => {
            assert!(!b64.is_empty(), "base64 não deve ser vazio");
            // PDF começa com "%PDF" → base64: "JVBE"
            assert!(
                b64.starts_with("JVBE"),
                "saída deve ser um PDF codificado em base64, mas começa com: {}",
                &b64[..8.min(b64.len())]
            );
            println!(
                "test_danfe_nfce_80mm_homologacao_as_base64: OK ({} bytes base64)",
                b64.len()
            );
        }
    }
}

#[tokio::test]
async fn test_danfe_nfce_80mm_multi_pagamento_as_base64() {
    let result = DanfeBuilder::new()
        .xml(NFCE_XML_MULTI_PAG)
        .paper_size("80mm")
        .as_base64()
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao gerar DANFE NFC-e (multi pagamento): {}", e),
        Ok(b64) => {
            assert!(!b64.is_empty());
            assert!(b64.starts_with("JVBE"), "saída deve ser PDF em base64");
            println!(
                "test_danfe_nfce_80mm_multi_pagamento_as_base64: OK ({} bytes base64)",
                b64.len()
            );
        }
    }
}

#[tokio::test]
async fn test_danfe_nfce_80mm_com_cpf_as_base64() {
    let result = DanfeBuilder::new()
        .xml(NFCE_XML_COM_CPF)
        .paper_size("80mm")
        .as_base64()
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao gerar DANFE NFC-e (com CPF): {}", e),
        Ok(b64) => {
            assert!(!b64.is_empty());
            assert!(b64.starts_with("JVBE"), "saída deve ser PDF em base64");
            println!(
                "test_danfe_nfce_80mm_com_cpf_as_base64: OK ({} bytes base64)",
                b64.len()
            );
        }
    }
}

#[tokio::test]
async fn test_danfe_nfce_80mm_salva_arquivo() {
    let output_path = "./test_nfce_output.pdf";
    let result = DanfeBuilder::new()
        .xml(NFCE_XML_MULTI_PAG)
        .paper_size("80mm")
        .as_file(output_path)
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao salvar DANFE NFC-e como arquivo: {}", e),
        Ok(path) => {
            assert_eq!(path, output_path);
            let meta = std::fs::metadata(output_path).expect("arquivo PDF não foi criado");
            assert!(meta.len() > 0, "arquivo PDF não deve ser vazio");
            println!(
                "test_danfe_nfce_80mm_salva_arquivo: OK ({} bytes)",
                meta.len()
            );
            //let _ = std::fs::remove_file(output_path);
        }
    }
}

#[tokio::test]
async fn test_danfe_nfce_80mm_xml_invalido_retorna_erro() {
    let result = DanfeBuilder::new()
        .xml("<nfeProc><NFe><infNFe></infNFe></NFe></nfeProc>")
        .paper_size("80mm")
        .as_base64()
        .build()
        .await;

    assert!(
        result.is_err(),
        "XML inválido/incompleto deveria retornar Err"
    );
    println!(
        "test_danfe_nfce_80mm_xml_invalido_retorna_erro: OK — erro: {}",
        result.unwrap_err()
    );
}

#[tokio::test]
async fn test_danfe_nfce_modelo_errado_retorna_erro() {
    // paper_size "a4" para modelo 65 ainda não implementado — deve retornar Err
    let result = DanfeBuilder::new()
        .xml(NFCE_XML_HOMOLOG)
        .paper_size("a4")
        .as_base64()
        .build()
        .await;

    assert!(
        result.is_err(),
        "a4 para NFC-e deveria retornar Err (não implementado)"
    );
    println!(
        "test_danfe_nfce_modelo_errado_retorna_erro: OK — erro: {}",
        result.unwrap_err()
    );
}

#[tokio::test]
async fn test_danfe_nfce_80mm_qr_side_as_base64() {
    let result = DanfeBuilder::new()
        .xml(NFCE_XML_HOMOLOG)
        .paper_size("80mm")
        .qr_side()
        .as_base64()
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao gerar DANFE NFC-e (qr_side): {}", e),
        Ok(b64) => {
            assert!(!b64.is_empty(), "base64 não deve ser vazio");
            assert!(
                b64.starts_with("JVBE"),
                "saída deve ser um PDF codificado em base64, mas começa com: {}",
                &b64[..8.min(b64.len())]
            );
            println!(
                "test_danfe_nfce_80mm_qr_side_as_base64: OK ({} bytes base64)",
                b64.len()
            );
        }
    }
}

#[tokio::test]
async fn test_danfe_nfce_80mm_qr_side_salva_arquivo() {
    let output_path = "./test_nfce_qr_side_output.pdf";
    let result = DanfeBuilder::new()
        .xml(NFCE_XML_MULTI_PAG)
        .paper_size("80mm")
        .qr_side()
        .as_file(output_path)
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao salvar DANFE NFC-e (qr_side) como arquivo: {}", e),
        Ok(path) => {
            assert_eq!(path, output_path);
            let meta = std::fs::metadata(output_path).expect("arquivo PDF não foi criado");
            assert!(meta.len() > 0, "arquivo PDF não deve ser vazio");
            println!(
                "test_danfe_nfce_80mm_qr_side_salva_arquivo: OK ({} bytes)",
                meta.len()
            );
        }
    }
}
