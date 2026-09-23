//! Montagem do `infInut` (nó assinado) e do envelope SOAP do `NFeInutilizacao4`.

use crate::error::Result;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::io::Cursor;

/// Dados do pedido de inutilização, já validados pelo builder.
pub(super) struct Pedido<'a> {
    pub tp_amb: u8,
    /// Código IBGE da UF do emitente (2 dígitos).
    pub c_uf: &'a str,
    /// Ano da numeração inutilizada, 2 dígitos.
    pub ano: &'a str,
    /// CNPJ do emitente, só alfanuméricos (14 posições).
    pub cnpj: &'a str,
    pub mod_: u32,
    pub serie: u32,
    pub n_ini: u64,
    pub n_fin: u64,
    pub justificativa: &'a str,
}

/// `Id` do `infInut`: `ID` + cUF(2) + ano(2) + CNPJ(14) + mod(2) + serie(3) + nNFIni(9) +
/// nNFFin(9) — 41 posições depois do prefixo. É também o alvo da assinatura (`#ID…`).
pub(super) fn inf_inut_id(p: &Pedido) -> String {
    format!(
        "ID{}{}{}{:02}{:03}{:09}{:09}",
        p.c_uf, p.ano, p.cnpj, p.mod_, p.serie, p.n_ini, p.n_fin
    )
}

/// Monta o `<infInut>` com o `xmlns` explícito — o digest é calculado sobre este nó, e a
/// canonicalização da SEFAZ reintroduz a declaração de namespace no ápice do subset, então ela
/// precisa estar aqui (mesmo raciocínio do `infEvento` em `interno::evento`).
pub(super) fn inf_inut_xml(p: &Pedido) -> Result<String> {
    let id = inf_inut_id(p);
    let tp_amb = p.tp_amb.to_string();
    let mod_ = p.mod_.to_string();
    let serie = p.serie.to_string();
    let n_ini = p.n_ini.to_string();
    let n_fin = p.n_fin.to_string();

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .create_element("infInut")
        .with_attribute(("xmlns", "http://www.portalfiscal.inf.br/nfe"))
        .with_attribute(("Id", id.as_str()))
        .write_inner_content(|w| {
            w.create_element("tpAmb")
                .write_text_content(BytesText::new(&tp_amb))?;
            w.create_element("xServ")
                .write_text_content(BytesText::new("INUTILIZAR"))?;
            w.create_element("cUF")
                .write_text_content(BytesText::new(p.c_uf))?;
            w.create_element("ano")
                .write_text_content(BytesText::new(p.ano))?;
            w.create_element("CNPJ")
                .write_text_content(BytesText::new(p.cnpj))?;
            w.create_element("mod")
                .write_text_content(BytesText::new(&mod_))?;
            w.create_element("serie")
                .write_text_content(BytesText::new(&serie))?;
            w.create_element("nNFIni")
                .write_text_content(BytesText::new(&n_ini))?;
            w.create_element("nNFFin")
                .write_text_content(BytesText::new(&n_fin))?;
            w.create_element("xJust")
                .write_text_content(BytesText::new(p.justificativa))?;
            Ok(())
        })?;

    Ok(String::from_utf8(writer.into_inner().into_inner())?)
}

/// Envelope SOAP 1.2 do `NFeInutilizacao4`, com o `inutNFe` (infInut assinado) no `nfeDadosMsg`.
/// `inf_inut` e `signature` entram como XML já pronto — são conteúdo canônico e não podem ser
/// re-escapados (mesma regra do `env_evento_xml`).
pub(super) fn inut_nfe_envelope(inf_inut: &str, signature: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeInutilizacao4"><inutNFe xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00">{}{}</inutNFe></nfeDadosMsg></soap12:Body></soap12:Envelope>"#,
        inf_inut, signature
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pedido() -> Pedido<'static> {
        Pedido {
            tp_amb: 2,
            c_uf: "35",
            ano: "26",
            cnpj: "11222333000181",
            mod_: 65,
            serie: 1,
            n_ini: 325,
            n_fin: 325,
            justificativa: "Numeracao pulada por falha de comunicacao na emissao",
        }
    }

    #[test]
    fn id_tem_41_posicoes_depois_do_prefixo() {
        let id = inf_inut_id(&pedido());
        assert!(id.starts_with("ID"));
        assert_eq!(id.len(), 43, "Id fora do layout: {id}");
        assert_eq!(id, "ID35261122233300018165001000000325000000325");
    }

    // CNPJ alfanumérico (NT 2025.002) entra no Id sem tratamento especial — as 14 posições são
    // alfanuméricas, não só dígitos.
    #[test]
    fn id_aceita_cnpj_alfanumerico() {
        let mut p = pedido();
        p.cnpj = "12ABC34501DE35";
        assert!(inf_inut_id(&p).contains("12ABC34501DE35"));
    }

    #[test]
    fn golden_inf_inut_xml() {
        insta::assert_snapshot!(inf_inut_xml(&pedido()).unwrap());
    }

    #[test]
    fn golden_envelope() {
        insta::assert_snapshot!(inut_nfe_envelope(
            "<infInut>FIXO</infInut>",
            "<Signature>FIXA</Signature>"
        ));
    }

    // xJust é texto livre digitado pelo operador — precisa ser escapado, senão um `&` quebra o XML.
    #[test]
    fn justificativa_e_escapada() {
        let mut p = pedido();
        p.justificativa = "Falha de comunicacao & numeracao <pulada>";
        let xml = inf_inut_xml(&p).unwrap();
        assert!(xml.contains("&amp;"));
        assert!(!xml.contains("<pulada>"));
    }
}
