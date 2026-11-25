//! Camada 2 (hermética) — `DanfeBuilder` para NF-e A4 (modelo 55).
//!
//! Movido na Fase 0.2 de `src/bin/tests/test_danfe_nfe_a4.rs`. Entradas/saídas em arquivo
//! usam a fixture versionada e o diretório temporário do SO (não dependem do CWD).

#![cfg(feature = "danfe")]

use base64::{engine::general_purpose::STANDARD, Engine as _};
use dfe::DanfeBuilder;
use printpdf::lopdf::Document;

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
    // Grava o XML da constante num arquivo temporário para exercitar o caminho de leitura
    // de arquivo do DanfeBuilder — self-contained, independente do CWD.
    let input_pathbuf = std::env::temp_dir().join("dfe_test_nfe_a4_input.xml");
    std::fs::write(&input_pathbuf, NFE_XML_55).expect("gravar XML de entrada temporário");
    let input_file = input_pathbuf.to_str().expect("caminho temp válido");

    let output_pathbuf = std::env::temp_dir().join("dfe_test_nfe_a4_output.pdf");
    let output_path = output_pathbuf.to_str().expect("caminho temp válido");
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

/// NT 2026.010 (DANFE A4 da Reforma): a mesma NF-e com IBS/CBS no item 1 (com `gRed` na CBS) e
/// `IBSCBSTot`. Gera o PDF com CRT no cabeçalho, bloco "TOTAL DO IBS / CBS / IS" e a sublinha do
/// item; o arquivo fica no temp do SO para conferência visual.
fn nfe_xml_55_com_rtc() -> String {
    let ibscbs = "</COFINS>\n          <IBSCBS><CST>200</CST><cClassTrib>200003</cClassTrib><gIBSCBS><vBC>211.87</vBC>\
<gIBSUF><pIBSUF>0.1000</pIBSUF><vIBSUF>0.21</vIBSUF></gIBSUF>\
<gIBSMun><pIBSMun>0.0000</pIBSMun><vIBSMun>0.00</vIBSMun></gIBSMun><vIBS>0.21</vIBS>\
<gCBS><pCBS>0.9000</pCBS><gRed><pRedAliq>60.0000</pRedAliq><pAliqEfet>0.3600</pAliqEfet></gRed><vCBS>0.76</vCBS></gCBS>\
</gIBSCBS></IBSCBS>";
    let tot = "</ICMSTot>\n        <IBSCBSTot><vBCIBSCBS>211.87</vBCIBSCBS><gIBS><gIBSUF><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vIBSUF>0.21</vIBSUF></gIBSUF>\
<gIBSMun><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vIBSMun>0.00</vIBSMun></gIBSMun><vIBS>0.21</vIBS><vCredPres>0.00</vCredPres><vCredPresCondSus>0.00</vCredPresCondSus></gIBS>\
<gCBS><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vCBS>0.76</vCBS><vCredPres>0.00</vCredPres><vCredPresCondSus>0.00</vCredPresCondSus></gCBS></IBSCBSTot>";
    NFE_XML_55.replacen("</COFINS>", ibscbs, 1).replacen("</ICMSTot>", tot, 1)
}

fn nfe_com_itens(quantidade: usize, alternar_rtc: bool) -> String {
    let inicio = NFE_XML_55.find("<det nItem=").unwrap();
    let fim = NFE_XML_55.rfind("</det>").unwrap() + "</det>".len();
    let rtc = nfe_xml_55_com_rtc();
    let item = |xml: &str| {
        let inicio = xml.find("<det nItem=").unwrap();
        let fim = xml.find("</det>").unwrap() + "</det>".len();
        inicio..fim
    };
    let normal = &NFE_XML_55[item(NFE_XML_55)];
    let com_rtc = &rtc[item(&rtc)];
    let mut xml = NFE_XML_55[..inicio].to_owned();
    for numero in 1..=quantidade {
        let modelo = if alternar_rtc && numero % 2 == 0 {
            com_rtc
        } else {
            normal
        };
        xml.push_str(
            &modelo
                .replacen("nItem=\"1\"", &format!("nItem=\"{numero}\""), 1)
                .replacen("P001", &format!("P{numero:03}"), 1)
                .replacen("200003", &format!("{:06}", 200000 + numero), 1),
        );
    }
    xml.push_str(&NFE_XML_55[fim..]);
    xml
}

async fn pdf_a4(xml: &str) -> Document {
    let base64 = DanfeBuilder::new()
        .xml(xml)
        .paper_size("a4")
        .as_base64()
        .build()
        .await
        .expect("gerar DANFE A4");
    Document::load_mem(&STANDARD.decode(base64).unwrap()).expect("ler o PDF gerado")
}

fn verificar_folhas(pdf: &Document, total: u32) -> Vec<String> {
    let paginas = pdf.get_pages();
    assert_eq!(paginas.len(), total as usize, "quantidade de folhas");
    paginas
        .into_iter()
        .map(|(numero, id)| {
            let pagina = pdf.get_dictionary(id).unwrap();
            let caixa = pagina.get(b"MediaBox").unwrap().as_array().unwrap();
            assert_eq!(caixa.len(), 4);
            for (medida, mm) in caixa.iter().zip([0.0, 0.0, 210.0, 297.0]) {
                assert!((medida.as_float().unwrap() - mm * 72.0 / 25.4).abs() < 0.01);
            }
            let texto = pdf.extract_text(&[numero]).unwrap();
            assert!(texto.contains(&format!("FOLHA {numero}/{total}")), "{texto}");
            assert!(texto.contains("VENDA DE MERCADORIA"), "{texto}");
            assert!(texto.contains("3524 0111 2223 3300 0181 5500 1000 0000 0110 0000 0011"));
            texto
        })
        .collect()
}

fn verificar_itens(folha: &str, numeros: std::ops::RangeInclusive<usize>) {
    let impressos: Vec<_> = folha
        .lines()
        .filter_map(|linha| linha.trim().strip_prefix('P')?.parse::<usize>().ok())
        .collect();
    assert_eq!(impressos, numeros.collect::<Vec<_>>());
}

#[tokio::test]
async fn danfe_a4_respeita_o_limite_da_primeira_folha() {
    verificar_folhas(&pdf_a4(NFE_XML_55).await, 1);
    let folhas = verificar_folhas(&pdf_a4(&nfe_com_itens(35, false)).await, 1);
    verificar_itens(&folhas[0], 1..=35);
    let folhas = verificar_folhas(&pdf_a4(&nfe_com_itens(36, false)).await, 2);
    verificar_itens(&folhas[0], 1..=35);
    verificar_itens(&folhas[1], 36..=36);
}

#[tokio::test]
async fn danfe_a4_pagina_60_itens_e_mantem_as_sublinhas_ibs_cbs_juntas() {
    let folhas = verificar_folhas(&pdf_a4(&nfe_com_itens(60, true)).await, 2);
    for (folha, numeros) in folhas.iter().zip([1..=25, 26..=60]) {
        assert!(folha.contains("NCM/SH") && folha.contains("CFOP"));
        verificar_itens(folha, numeros.clone());
        let sublinhas: Vec<_> = folha
            .lines()
            .filter(|linha| linha.contains("cClassTrib"))
            .collect();
        let esperadas: Vec<_> = numeros.filter(|n| n % 2 == 0).collect();
        assert_eq!(sublinhas.len(), esperadas.len());
        for (linha, numero) in sublinhas.iter().zip(esperadas) {
            assert!(linha.contains(&format!("{:06}", 200000 + numero)), "{linha}");
        }
    }
}

#[tokio::test]
async fn danfe_a4_continua_informacoes_complementares_em_folha_adicional() {
    let observacoes: Vec<_> = (1..=250).map(|n| format!("OBS{n:03}")).collect();
    let xml = nfe_com_itens(35, false);
    let inicio = xml.find("<infCpl>").unwrap() + "<infCpl>".len();
    let fim = xml.find("</infCpl>").unwrap();
    let xml = format!("{}{}{}", &xml[..inicio], observacoes.join(" "), &xml[fim..]);
    let folhas = verificar_folhas(&pdf_a4(&xml).await, 2);
    verificar_itens(&folhas[0], 1..=35);
    assert!(!folhas[1].contains("DADOS DOS PRODUTOS"));
    assert!(folhas[1].contains("COMPLEMENTARES (CONTINUA"));
    assert!(folhas[0].contains("OBS001") && folhas[1].contains("OBS250"));
    let impressas: Vec<_> = folhas
        .iter()
        .flat_map(|texto| texto.split_whitespace())
        .filter(|palavra| palavra.starts_with("OBS"))
        .collect();
    assert_eq!(impressas, observacoes);
}

#[tokio::test]
async fn test_danfe_nfe_a4_reforma_tributaria_nt_2026_010() {
    let xml = nfe_xml_55_com_rtc();
    assert!(xml.contains("<IBSCBS>") && xml.contains("<IBSCBSTot>"));
    let saida = std::env::temp_dir().join("dfe_test_nfe_a4_rtc.pdf");
    let r = DanfeBuilder::new()
        .xml(&xml)
        .paper_size("a4")
        .as_file(saida.to_str().unwrap())
        .build()
        .await;
    assert!(r.is_ok(), "DANFE A4 com IBS/CBS: {:?}", r.err());
    assert!(std::fs::metadata(&saida).unwrap().len() > 0);
}
