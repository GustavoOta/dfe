#[cfg(test)]
use dfe::DanfeBuilder;

/// XML mínimo de NF-e (modelo 55) válido para testes de geração de DANFE A4.
const NFE_XML_55: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
  <NFe xmlns="http://www.portalfiscal.inf.br/nfe">
    <infNFe versao="4.00" Id="NFe35240111222333000181550010000000011000000011">
      <ide>
        <cUF>35</cUF>
        <cNF>10000001</cNF>
        <natOp>VENDA DE MERCADORIA</natOp>
        <mod>55</mod>
        <serie>1</serie>
        <nNF>1</nNF>
        <dhEmi>2024-03-01T10:00:00-03:00</dhEmi>
        <dhSaiEnt>2024-03-01T10:30:00-03:00</dhSaiEnt>
        <tpNF>1</tpNF>
        <idDest>1</idDest>
        <cMunFG>3550308</cMunFG>
        <tpImp>1</tpImp>
        <tpEmis>1</tpEmis>
        <cDV>1</cDV>
        <tpAmb>2</tpAmb>
        <finNFe>1</finNFe>
        <indFinal>0</indFinal>
        <indPres>0</indPres>
        <procEmi>0</procEmi>
        <verProc>1.0</verProc>
      </ide>
      <emit>
        <CNPJ>11222333000181</CNPJ>
        <xNome>DISTRIBUIDORA DE PRODUTOS LTDA</xNome>
        <xFant>DISTRIBUIDORA LTDA</xFant>
        <enderEmit>
          <xLgr>RUA COMERCIAL</xLgr>
          <nro>500</nro>
          <xBairro>CENTRO</xBairro>
          <cMun>3550308</cMun>
          <xMun>SAO PAULO</xMun>
          <UF>SP</UF>
          <CEP>01310100</CEP>
          <cPais>1058</cPais>
          <xPais>BRASIL</xPais>
          <fone>1133334444</fone>
        </enderEmit>
        <IE>111222333444</IE>
        <CRT>3</CRT>
      </emit>
      <dest>
        <CNPJ>44555666000177</CNPJ>
        <xNome>EMPRESA COMPRADORA SA</xNome>
        <enderDest>
          <xLgr>AV INDUSTRIAL</xLgr>
          <nro>100</nro>
          <xBairro>DISTRITO INDUSTRIAL</xBairro>
          <cMun>3518800</cMun>
          <xMun>CAMPINAS</xMun>
          <UF>SP</UF>
          <CEP>13000000</CEP>
          <cPais>1058</cPais>
          <xPais>BRASIL</xPais>
        </enderDest>
        <indIEDest>1</indIEDest>
        <IE>222333444555</IE>
      </dest>
      <det nItem="1">
        <prod>
          <cProd>P001</cProd>
          <cEAN>7891234560001</cEAN>
          <xProd>PRODUTO DE TESTE ITEM UM DESCRICAO LONGA</xProd>
          <NCM>22030000</NCM>
          <CFOP>5102</CFOP>
          <uCom>CX</uCom>
          <qCom>10.0000</qCom>
          <vUnCom>25.0000</vUnCom>
          <vProd>250.00</vProd>
          <cEANTrib>7891234560001</cEANTrib>
          <uTrib>UN</uTrib>
          <qTrib>10.0000</qTrib>
          <vUnTrib>25.0000</vUnTrib>
          <indTot>1</indTot>
        </prod>
        <imposto>
          <ICMS>
            <ICMS00>
              <orig>0</orig>
              <CST>00</CST>
              <modBC>3</modBC>
              <vBC>250.00</vBC>
              <pICMS>12.00</pICMS>
              <vICMS>30.00</vICMS>
            </ICMS00>
          </ICMS>
          <PIS>
            <PISAliq>
              <CST>01</CST>
              <vBC>250.00</vBC>
              <pPIS>0.65</pPIS>
              <vPIS>1.63</vPIS>
            </PISAliq>
          </PIS>
          <COFINS>
            <COFINSAliq>
              <CST>01</CST>
              <vBC>250.00</vBC>
              <pCOFINS>3.00</pCOFINS>
              <vCOFINS>7.50</vCOFINS>
            </COFINSAliq>
          </COFINS>
        </imposto>
      </det>
      <det nItem="2">
        <prod>
          <cProd>P002</cProd>
          <cEAN>7891234560002</cEAN>
          <xProd>SEGUNDO PRODUTO TESTE COM NOME MAIS LONGO AINDA</xProd>
          <NCM>39231000</NCM>
          <CFOP>5102</CFOP>
          <uCom>PC</uCom>
          <qCom>5.0000</qCom>
          <vUnCom>40.0000</vUnCom>
          <vProd>200.00</vProd>
          <cEANTrib>7891234560002</cEANTrib>
          <uTrib>PC</uTrib>
          <qTrib>5.0000</qTrib>
          <vUnTrib>40.0000</vUnTrib>
          <indTot>1</indTot>
        </prod>
        <imposto>
          <ICMS>
            <ICMS00>
              <orig>0</orig>
              <CST>00</CST>
              <modBC>3</modBC>
              <vBC>200.00</vBC>
              <pICMS>12.00</pICMS>
              <vICMS>24.00</vICMS>
            </ICMS00>
          </ICMS>
          <PIS>
            <PISAliq>
              <CST>01</CST>
              <vBC>200.00</vBC>
              <pPIS>0.65</pPIS>
              <vPIS>1.30</vPIS>
            </PISAliq>
          </PIS>
          <COFINS>
            <COFINSAliq>
              <CST>01</CST>
              <vBC>200.00</vBC>
              <pCOFINS>3.00</pCOFINS>
              <vCOFINS>6.00</vCOFINS>
            </COFINSAliq>
          </COFINS>
        </imposto>
      </det>
      <total>
        <ICMSTot>
          <vBC>450.00</vBC>
          <vICMS>54.00</vICMS>
          <vICMSDeson>0.00</vICMSDeson>
          <vFCP>0.00</vFCP>
          <vBCST>0.00</vBCST>
          <vST>0.00</vST>
          <vFCPST>0.00</vFCPST>
          <vFCPSTRet>0.00</vFCPSTRet>
          <vProd>450.00</vProd>
          <vFrete>10.00</vFrete>
          <vSeg>0.00</vSeg>
          <vDesc>0.00</vDesc>
          <vII>0.00</vII>
          <vIPI>0.00</vIPI>
          <vIPIDevol>0.00</vIPIDevol>
          <vPIS>2.93</vPIS>
          <vCOFINS>13.50</vCOFINS>
          <vOutro>0.00</vOutro>
          <vNF>460.00</vNF>
          <vTotTrib>70.43</vTotTrib>
        </ICMSTot>
      </total>
      <transp>
        <modFrete>0</modFrete>
      </transp>
      <pag>
        <detPag>
          <indPag>1</indPag>
          <tPag>15</tPag>
          <vPag>460.00</vPag>
        </detPag>
      </pag>
      <infAdic>
        <infCpl>VENDA SUJEITA AO REGIME NORMAL. TRIBUTACAO CONFORME LEGISLACAO VIGENTE. OBRIGADO PELA PREFERENCIA.</infCpl>
      </infAdic>
    </infNFe>
  </NFe>
  <protNFe versao="4.00">
    <infProt>
      <tpAmb>2</tpAmb>
      <verAplic>SP_NFe_v4.0</verAplic>
      <chNFe>35240111222333000181550010000000011000000011</chNFe>
      <dhRecbto>2024-03-01T10:00:10-03:00</dhRecbto>
      <nProt>135240000000099</nProt>
      <digVal>xyz789abc123=</digVal>
      <cStat>100</cStat>
      <xMotivo>Autorizado o uso da NF-e</xMotivo>
    </infProt>
  </protNFe>
</nfeProc>"#;

// ── Testes ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_danfe_nfe_a4_modelo55_as_base64() {
    let result = DanfeBuilder::new()
        .xml(NFE_XML_55)
        .paper_size("a4")
        .as_base64()
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao gerar DANFE NF-e A4 (modelo 55): {}", e),
        Ok(b64) => {
            assert!(!b64.is_empty(), "base64 não deve ser vazio");
            assert!(
                b64.starts_with("JVBE"),
                "saída deve ser um PDF codificado em base64, mas começa com: {}",
                &b64[..8.min(b64.len())]
            );
            println!(
                "test_danfe_nfe_a4_modelo55_as_base64: OK ({} bytes base64)",
                b64.len()
            );
        }
    }
}

#[tokio::test]
async fn test_danfe_nfe_a4_modelo55_salva_arquivo() {
    // testar lendo a partir de arquivo xml em disco
    let input_file = "../teste.xml";
    std::fs::read_to_string(input_file).expect("arquivo de entrada XML para teste não encontrado");

    let output_path = "./test_nfe_a4_output.pdf";
    let result = DanfeBuilder::new()
        .xml(input_file)
        .paper_size("a4")
        .as_file(output_path)
        .build()
        .await;

    match &result {
        Err(e) => panic!("Erro ao salvar DANFE NF-e A4 (modelo 55): {}", e),
        Ok(path) => {
            assert_eq!(path, output_path);
            let meta = std::fs::metadata(output_path).expect("arquivo PDF não foi criado");
            assert!(meta.len() > 0, "arquivo PDF não deve ser vazio");
            println!(
                "test_danfe_nfe_a4_modelo55_salva_arquivo: OK ({} bytes)",
                meta.len()
            );
        }
    }
}

#[tokio::test]
async fn test_danfe_nfe_a4_modelo65_retorna_erro() {
    // A4 para modelo 65 (NFC-e) ainda não implementado — deve retornar Err
    let xml_65 = r#"<?xml version="1.0" encoding="utf-8"?>
<nfeProc versao="4.00"><NFe><infNFe versao="4.00" Id="NFe35240100000000000000650010000000011000000011">
<ide><mod>65</mod><tpAmb>2</tpAmb></ide>
<emit><CNPJ>11222333000181</CNPJ><xNome>TESTE</xNome><enderEmit><UF>SP</UF></enderEmit><IE>111</IE><CRT>1</CRT></emit>
<total><ICMSTot><vNF>0.00</vNF></ICMSTot></total>
<transp><modFrete>9</modFrete></transp>
<pag><detPag><tPag>01</tPag><vPag>0.00</vPag></detPag></pag>
</infNFe></NFe><protNFe versao="4.00"><infProt></infProt></protNFe></nfeProc>"#;

    let result = DanfeBuilder::new()
        .xml(xml_65)
        .paper_size("a4")
        .as_base64()
        .build()
        .await;

    assert!(
        result.is_err(),
        "a4 para NFC-e deveria retornar Err (não implementado)"
    );
    println!(
        "test_danfe_nfe_a4_modelo65_retorna_erro: OK — erro: {}",
        result.unwrap_err()
    );
}
