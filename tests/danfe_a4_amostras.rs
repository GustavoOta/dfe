//! Amostras do DANFE A4 (NF-e modelo 55) para conferir o leiaute a olho.
//!
//! Cada teste monta uma NF-e autorizada fictícia, gera o PDF A4 e grava em disco — sem
//! certificado e sem SEFAZ, no `cargo test` padrão. Para ver os arquivos:
//!
//! ```bash
//! cargo test --test danfe_a4_amostras -- --nocapture
//! ```
//!
//! Os PDFs vão para `target/tmp/danfe-a4/` (ou para a pasta em `DFE_DANFE_SAIDA`), e cada teste
//! imprime o caminho. Cenários:
//!
//! | Arquivo | O que mostra |
//! |---|---|
//! | `01-simples-sem-ibs-cbs.pdf` | CRT 1, sem IBS/CBS: bloco "TOTAL DO IBS / CBS / IS" em branco e nenhuma sublinha |
//! | `02-regime-normal-ibs-cbs.pdf` | CRT 3 com IBS/CBS em todos os itens e os totais preenchidos |
//! | `03-reducao-e-imposto-seletivo.pdf` | `gRed` (alíquota efetiva impressa no lugar da vigente) e Imposto Seletivo |
//! | `04-monofasico.pdf` | `gMono` nos totais: aparece a linha da monofasia |
//! | `05-muitos-itens.pdf` | 60 itens, metade com IBS/CBS: a página cresce para caber tudo |
//! | `06-mei.pdf` | CRT 4 (MEI), sem grupo IBS/CBS |
//!
//! Os valores são fictícios e servem só para ilustrar a disposição dos campos (NT 2026.010).

#![cfg(feature = "danfe")]

use std::path::PathBuf;

use dfe::DanfeBuilder;

// ── Pasta de saída ──────────────────────────────────────────────────────────

fn pasta_saida() -> PathBuf {
    let pasta = std::env::var_os("DFE_DANFE_SAIDA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("danfe-a4"));
    std::fs::create_dir_all(&pasta).expect("criar a pasta de saída das amostras");
    pasta
}

/// Gera o PDF, confere que é um PDF de verdade e devolve o caminho.
async fn gerar(nome: &str, xml: &str) -> PathBuf {
    let caminho = pasta_saida().join(nome);
    let r = DanfeBuilder::new()
        .xml(xml)
        .paper_size("a4")
        .as_file(caminho.to_str().expect("caminho UTF-8"))
        .build()
        .await;
    assert!(r.is_ok(), "{nome}: {:?}", r.err());
    let bytes = std::fs::read(&caminho).expect("ler o PDF gerado");
    assert!(bytes.starts_with(b"%PDF"), "{nome}: não é PDF");
    assert!(bytes.len() > 1_000, "{nome}: PDF pequeno demais ({} bytes)", bytes.len());
    println!("DANFE A4 gerado: {}", caminho.display());
    caminho
}

// ── Montagem do XML ─────────────────────────────────────────────────────────

/// IBS/CBS de um item. `red_cbs` = alíquota efetiva com `gRed` na CBS.
#[derive(Clone)]
struct IbsCbs {
    class_trib: &'static str,
    p_ibs_uf: &'static str,
    p_ibs_mun: &'static str,
    p_cbs: &'static str,
    red_cbs: Option<&'static str>,
    is: Option<(&'static str, &'static str)>, // (pIS, vIS) sobre o vProd
}

#[derive(Clone)]
struct Item {
    descricao: String,
    ncm: &'static str,
    qtd: f64,
    unit: f64,
    ibs_cbs: Option<IbsCbs>,
}

struct Nota {
    crt: &'static str,
    itens: Vec<Item>,
    mono: bool,
    obs: &'static str,
}

fn dec(v: f64) -> String {
    format!("{:.2}", (v * 100.0).round() / 100.0)
}

fn item_xml(n: usize, it: &Item, crt: &str) -> (String, f64, f64, f64, f64, f64) {
    let v_prod = it.qtd * it.unit;
    let icms = if crt == "3" {
        format!(
            "<ICMS00><orig>0</orig><CST>00</CST><modBC>3</modBC><vBC>{}</vBC><pICMS>18.00</pICMS><vICMS>{}</vICMS></ICMS00>",
            dec(v_prod),
            dec(v_prod * 0.18)
        )
    } else {
        "<ICMSSN102><orig>0</orig><CSOSN>102</CSOSN></ICMSSN102>".to_string()
    };
    let (mut v_ibs_uf, mut v_ibs_mun, mut v_cbs, mut v_is) = (0.0, 0.0, 0.0, 0.0);
    let reforma = match &it.ibs_cbs {
        None => String::new(),
        Some(g) => {
            let pct = |p: &str| p.parse::<f64>().unwrap() / 100.0;
            v_ibs_uf = v_prod * pct(g.p_ibs_uf);
            v_ibs_mun = v_prod * pct(g.p_ibs_mun);
            let p_cbs_efetiva = g.red_cbs.unwrap_or(g.p_cbs);
            v_cbs = v_prod * pct(p_cbs_efetiva);
            let red = g
                .red_cbs
                .map(|ef| format!("<gRed><pRedAliq>60.0000</pRedAliq><pAliqEfet>{ef}</pAliqEfet></gRed>"))
                .unwrap_or_default();
            let is = g
                .is
                .map(|(p_is, _)| {
                    v_is = v_prod * pct(p_is);
                    format!(
                        "<IS><CSTIS>000</CSTIS><cClassTribIS>000001</cClassTribIS><vBCIS>{}</vBCIS><pIS>{p_is}</pIS>\
                         <uTrib>UN</uTrib><qTrib>{:.4}</qTrib><vIS>{}</vIS></IS>",
                        dec(v_prod),
                        it.qtd,
                        dec(v_is)
                    )
                })
                .unwrap_or_default();
            format!(
                "{is}<IBSCBS><CST>{}</CST><cClassTrib>{}</cClassTrib><gIBSCBS><vBC>{}</vBC>\
                 <gIBSUF><pIBSUF>{}</pIBSUF><vIBSUF>{}</vIBSUF></gIBSUF>\
                 <gIBSMun><pIBSMun>{}</pIBSMun><vIBSMun>{}</vIBSMun></gIBSMun><vIBS>{}</vIBS>\
                 <gCBS><pCBS>{}</pCBS>{red}<vCBS>{}</vCBS></gCBS></gIBSCBS></IBSCBS>",
                if g.red_cbs.is_some() { "200" } else { "000" },
                g.class_trib,
                dec(v_prod),
                g.p_ibs_uf,
                dec(v_ibs_uf),
                g.p_ibs_mun,
                dec(v_ibs_mun),
                dec(v_ibs_uf + v_ibs_mun),
                g.p_cbs,
                dec(v_cbs)
            )
        }
    };
    let xml = format!(
        "<det nItem=\"{n}\"><prod><cProd>P{n:03}</cProd><cEAN>SEM GTIN</cEAN><xProd>{}</xProd><NCM>{}</NCM>\
         <CFOP>5102</CFOP><uCom>UN</uCom><qCom>{:.4}</qCom><vUnCom>{:.4}</vUnCom><vProd>{}</vProd>\
         <cEANTrib>SEM GTIN</cEANTrib><uTrib>UN</uTrib><qTrib>{:.4}</qTrib><vUnTrib>{:.4}</vUnTrib><indTot>1</indTot></prod>\
         <imposto><ICMS>{icms}</ICMS><PIS><PISOutr><CST>99</CST><vBC>0.00</vBC><pPIS>0.00</pPIS><vPIS>0.00</vPIS></PISOutr></PIS>\
         <COFINS><COFINSOutr><CST>99</CST><vBC>0.00</vBC><pCOFINS>0.00</pCOFINS><vCOFINS>0.00</vCOFINS></COFINSOutr></COFINS>{reforma}</imposto></det>",
        it.descricao,
        it.ncm,
        it.qtd,
        it.unit,
        dec(v_prod),
        it.qtd,
        it.unit
    );
    (xml, v_prod, v_ibs_uf, v_ibs_mun, v_cbs, v_is)
}

fn nota_xml(n: &Nota) -> String {
    let (mut dets, mut v_prod, mut v_icms) = (String::new(), 0.0, 0.0);
    let (mut v_ibs_uf, mut v_ibs_mun, mut v_cbs, mut v_is, mut base_rtc) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut tem_reforma = false;
    for (i, it) in n.itens.iter().enumerate() {
        let (xml, vp, uf, mun, cbs, is) = item_xml(i + 1, it, n.crt);
        dets.push_str(&xml);
        v_prod += vp;
        if n.crt == "3" {
            v_icms += vp * 0.18;
        }
        if it.ibs_cbs.is_some() {
            tem_reforma = true;
            base_rtc += vp;
        }
        v_ibs_uf += uf;
        v_ibs_mun += mun;
        v_cbs += cbs;
        v_is += is;
    }
    let base_icms = if n.crt == "3" { v_prod } else { 0.0 };
    let mono = if n.mono {
        "<gMono><vIBSMono>12.40</vIBSMono><vCBSMono>27.90</vCBSMono><vIBSMonoReten>3.10</vIBSMonoReten>\
         <vCBSMonoReten>6.98</vCBSMonoReten><vIBSMonoRet>0.00</vIBSMonoRet><vCBSMonoRet>0.00</vCBSMonoRet></gMono>"
    } else {
        ""
    };
    let ibscbs_tot = if tem_reforma || n.mono {
        format!(
            "<IBSCBSTot><vBCIBSCBS>{}</vBCIBSCBS><gIBS><gIBSUF><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vIBSUF>{}</vIBSUF></gIBSUF>\
             <gIBSMun><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vIBSMun>{}</vIBSMun></gIBSMun><vIBS>{}</vIBS>\
             <vCredPres>0.00</vCredPres><vCredPresCondSus>0.00</vCredPresCondSus></gIBS>\
             <gCBS><vDif>0.00</vDif><vDevTrib>0.00</vDevTrib><vCBS>{}</vCBS><vCredPres>0.00</vCredPres>\
             <vCredPresCondSus>0.00</vCredPresCondSus></gCBS>{mono}</IBSCBSTot>",
            dec(base_rtc),
            dec(v_ibs_uf),
            dec(v_ibs_mun),
            dec(v_ibs_uf + v_ibs_mun),
            dec(v_cbs)
        )
    } else {
        String::new()
    };
    let is_tot = if v_is > 0.0 { format!("<ISTot><vIS>{}</vIS></ISTot>", dec(v_is)) } else { String::new() };
    let chave = "35261111222333000181550010000012341000012345";
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe"><NFe><infNFe versao="4.00" Id="NFe{chave}">
<ide><cUF>35</cUF><cNF>00001234</cNF><natOp>VENDA DE MERCADORIA</natOp><mod>55</mod><serie>1</serie><nNF>1234</nNF>
<dhEmi>2026-12-01T10:00:00-03:00</dhEmi><dhSaiEnt>2026-12-01T10:30:00-03:00</dhSaiEnt><tpNF>1</tpNF><idDest>1</idDest>
<cMunFG>3550308</cMunFG><tpImp>1</tpImp><tpEmis>1</tpEmis><cDV>5</cDV><tpAmb>2</tpAmb><finNFe>1</finNFe><indFinal>0</indFinal>
<indPres>1</indPres><procEmi>0</procEmi><verProc>dfe-amostra</verProc></ide>
<emit><CNPJ>11222333000181</CNPJ><xNome>COMERCIO EXEMPLO LTDA</xNome><xFant>COMERCIO EXEMPLO</xFant><enderEmit>
<xLgr>RUA DO COMERCIO</xLgr><nro>100</nro><xBairro>CENTRO</xBairro><cMun>3550308</cMun><xMun>SAO PAULO</xMun><UF>SP</UF>
<CEP>01001000</CEP><cPais>1058</cPais><xPais>BRASIL</xPais><fone>1130000000</fone></enderEmit><IE>111222333444</IE>
<CRT>{crt}</CRT></emit>
<dest><CNPJ>44555666000177</CNPJ><xNome>CLIENTE EXEMPLO SA</xNome><enderDest><xLgr>AV INDUSTRIAL</xLgr><nro>200</nro>
<xBairro>DISTRITO</xBairro><cMun>3509502</cMun><xMun>CAMPINAS</xMun><UF>SP</UF><CEP>13000000</CEP><cPais>1058</cPais>
<xPais>BRASIL</xPais></enderDest><indIEDest>1</indIEDest><IE>222333444555</IE></dest>
{dets}
<total><ICMSTot><vBC>{vbc}</vBC><vICMS>{vicms}</vICMS><vICMSDeson>0.00</vICMSDeson><vFCP>0.00</vFCP><vBCST>0.00</vBCST>
<vST>0.00</vST><vFCPST>0.00</vFCPST><vFCPSTRet>0.00</vFCPSTRet><vProd>{vprod}</vProd><vFrete>0.00</vFrete><vSeg>0.00</vSeg>
<vDesc>0.00</vDesc><vII>0.00</vII><vIPI>0.00</vIPI><vIPIDevol>0.00</vIPIDevol><vPIS>0.00</vPIS><vCOFINS>0.00</vCOFINS>
<vOutro>0.00</vOutro><vNF>{vprod}</vNF></ICMSTot>{is_tot}{ibscbs_tot}</total>
<transp><modFrete>9</modFrete></transp>
<pag><detPag><indPag>1</indPag><tPag>15</tPag><vPag>{vprod}</vPag></detPag></pag>
<infAdic><infCpl>{obs}</infCpl></infAdic>
</infNFe></NFe>
<protNFe versao="4.00"><infProt><tpAmb>2</tpAmb><verAplic>SP_NFE_PL_010</verAplic><chNFe>{chave}</chNFe>
<dhRecbto>2026-12-01T10:00:05-03:00</dhRecbto><nProt>135260000012345</nProt><digVal>AAAA</digVal><cStat>100</cStat>
<xMotivo>Autorizado o uso da NF-e</xMotivo></infProt></protNFe></nfeProc>"#,
        crt = n.crt,
        vbc = dec(base_icms),
        vicms = dec(v_icms),
        vprod = dec(v_prod),
        obs = n.obs,
    )
}

fn item(desc: &str, ncm: &'static str, qtd: f64, unit: f64, ibs_cbs: Option<IbsCbs>) -> Item {
    Item { descricao: desc.to_string(), ncm, qtd, unit, ibs_cbs }
}

const PADRAO: IbsCbs = IbsCbs {
    class_trib: "000001",
    p_ibs_uf: "0.1000",
    p_ibs_mun: "0.0000",
    p_cbs: "0.9000",
    red_cbs: None,
    is: None,
};

// ── Amostras ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn amostra_01_simples_sem_ibs_cbs() {
    let nota = Nota {
        crt: "1",
        itens: vec![
            item("ARROZ TIPO 1 PACOTE 5 KG", "10063021", 10.0, 24.90, None),
            item("FEIJAO CARIOCA 1 KG", "07133399", 20.0, 8.49, None),
            item("OLEO DE SOJA 900 ML", "15079011", 12.0, 7.99, None),
        ],
        mono: false,
        obs: "EMPRESA OPTANTE PELO SIMPLES NACIONAL. Amostra da crate dfe - sem IBS/CBS no XML.",
    };
    gerar("01-simples-sem-ibs-cbs.pdf", &nota_xml(&nota)).await;
}

#[tokio::test]
async fn amostra_02_regime_normal_ibs_cbs() {
    let nota = Nota {
        crt: "3",
        itens: vec![
            item("NOTEBOOK 15 POL 16GB RAM", "84713012", 2.0, 3899.00, Some(PADRAO)),
            item("MOUSE SEM FIO", "84716053", 10.0, 59.90, Some(PADRAO)),
            item("CABO HDMI 2 METROS", "85444200", 15.0, 24.50, Some(PADRAO)),
        ],
        mono: false,
        obs: "Amostra da crate dfe - regime normal com IBS/CBS (aliquotas de teste de 2026).",
    };
    gerar("02-regime-normal-ibs-cbs.pdf", &nota_xml(&nota)).await;
}

#[tokio::test]
async fn amostra_03_reducao_e_imposto_seletivo() {
    let reduzida = IbsCbs { class_trib: "200003", red_cbs: Some("0.3600"), ..PADRAO };
    let seletivo = IbsCbs { class_trib: "000001", is: Some(("10.0000", "")), ..PADRAO };
    let nota = Nota {
        crt: "3",
        itens: vec![
            item("LEITE UHT INTEGRAL 1 L (ALIQUOTA REDUZIDA)", "04012010", 48.0, 4.79, Some(reduzida)),
            item("REFRIGERANTE COLA 2 L (IMPOSTO SELETIVO)", "22021000", 24.0, 9.99, Some(seletivo)),
            item("SABAO EM PO 1 KG", "34022000", 6.0, 18.90, Some(PADRAO)),
        ],
        mono: false,
        obs: "Amostra da crate dfe - gRed (aliquota efetiva da CBS) e Imposto Seletivo.",
    };
    gerar("03-reducao-e-imposto-seletivo.pdf", &nota_xml(&nota)).await;
}

#[tokio::test]
async fn amostra_04_monofasico() {
    let nota = Nota {
        crt: "3",
        itens: vec![item("OLEO DIESEL S10 (LITRO)", "27101921", 500.0, 6.19, Some(PADRAO))],
        mono: true,
        obs: "Amostra da crate dfe - totais com gMono (linha da monofasia).",
    };
    gerar("04-monofasico.pdf", &nota_xml(&nota)).await;
}

#[tokio::test]
async fn amostra_05_muitos_itens() {
    let itens = (1..=60)
        .map(|i| {
            item(
                &format!("PRODUTO DE AMOSTRA NUMERO {i:02} COM DESCRICAO MEDIA"),
                "39231000",
                (i % 7 + 1) as f64,
                3.5 * i as f64,
                (i % 2 == 0).then_some(PADRAO),
            )
        })
        .collect();
    let nota = Nota {
        crt: "3",
        itens,
        mono: false,
        obs: "Amostra da crate dfe - 60 itens, metade com IBS/CBS (sublinha so nesses).",
    };
    gerar("05-muitos-itens.pdf", &nota_xml(&nota)).await;
}

#[tokio::test]
async fn amostra_06_mei() {
    let nota = Nota {
        crt: "4",
        itens: vec![item("BOLO CASEIRO DE CENOURA", "19059090", 3.0, 35.00, None)],
        mono: false,
        obs: "Amostra da crate dfe - MEI (CRT 4), sem grupo IBS/CBS.",
    };
    gerar("06-mei.pdf", &nota_xml(&nota)).await;
}

/// A amostra sem IBS/CBS e a com IBS/CBS geram arquivos diferentes (a sublinha e os totais
/// entram no PDF).
#[tokio::test]
async fn amostras_com_e_sem_reforma_diferem() {
    let sem = Nota { crt: "3", itens: vec![item("ITEM", "39231000", 1.0, 10.0, None)], mono: false, obs: "x" };
    let com = Nota { crt: "3", itens: vec![item("ITEM", "39231000", 1.0, 10.0, Some(PADRAO))], mono: false, obs: "x" };
    let a = std::fs::read(gerar("zz-comparacao-sem.pdf", &nota_xml(&sem)).await).unwrap();
    let b = std::fs::read(gerar("zz-comparacao-com.pdf", &nota_xml(&com)).await).unwrap();
    assert_ne!(a, b);
}
