#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    struct RetEnviNFe {
        tp_amb: String,
        ver_aplic: String,
        c_stat: String,
        x_motivo: String,
        c_uf: String,
        dh_recbto: String,
        t_med: String,
    }

    struct ProtNFe {
        tp_amb: String,
        ver_aplic: String,
        ch_nfe: String,
        dh_recbto: String,
        n_prot: String,
        dig_val: String,
        c_stat: String,
        x_motivo: String,
    }

    fn extract_tags(xml: &str) -> (RetEnviNFe, ProtNFe) {
        let mut reader = Reader::from_str(xml);

        let mut ret_envi_nfe = RetEnviNFe {
            tp_amb: String::new(),
            ver_aplic: String::new(),
            c_stat: String::new(),
            x_motivo: String::new(),
            c_uf: String::new(),
            dh_recbto: String::new(),
            t_med: String::new(),
        };

        let mut prot_nfe = ProtNFe {
            tp_amb: String::new(),
            ver_aplic: String::new(),
            ch_nfe: String::new(),
            dh_recbto: String::new(),
            n_prot: String::new(),
            dig_val: String::new(),
            c_stat: String::new(),
            x_motivo: String::new(),
        };

        let mut buf: Vec<u8> = Vec::new();
        let mut is_ret_envi_nfe = false;
        let mut is_prot_nfe = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"retEnviNFe" {
                        is_ret_envi_nfe = true;
                    }
                    if e.name().as_ref() == b"protNFe" {
                        is_prot_nfe = true;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"retEnviNFe" {
                        is_ret_envi_nfe = false;
                    }
                    if e.name().as_ref() == b"protNFe" {
                        is_prot_nfe = false;
                    }
                }
                Ok(Event::Text(e)) => {
                    if is_ret_envi_nfe {
                        if ret_envi_nfe.tp_amb.is_empty() {
                            ret_envi_nfe.tp_amb = e.unescape().unwrap().trim().to_string();
                        } else if ret_envi_nfe.ver_aplic.is_empty() {
                            ret_envi_nfe.ver_aplic = e.unescape().unwrap().trim().to_string();
                        } else if ret_envi_nfe.c_stat.is_empty() {
                            ret_envi_nfe.c_stat = e.unescape().unwrap().trim().to_string();
                        } else if ret_envi_nfe.x_motivo.is_empty() {
                            ret_envi_nfe.x_motivo = e.unescape().unwrap().trim().to_string();
                        } else if ret_envi_nfe.c_uf.is_empty() {
                            ret_envi_nfe.c_uf = e.unescape().unwrap().trim().to_string();
                        } else if ret_envi_nfe.dh_recbto.is_empty() {
                            ret_envi_nfe.dh_recbto = e.unescape().unwrap().trim().to_string();
                        } else if ret_envi_nfe.t_med.is_empty() {
                            ret_envi_nfe.t_med = e.unescape().unwrap().trim().to_string();
                        }
                    }
                    if is_prot_nfe {
                        if prot_nfe.tp_amb.is_empty() {
                            prot_nfe.tp_amb = e.unescape().unwrap().trim().to_string();
                        } else if prot_nfe.ver_aplic.is_empty() {
                            prot_nfe.ver_aplic = e.unescape().unwrap().trim().to_string();
                        } else if prot_nfe.ch_nfe.is_empty() {
                            prot_nfe.ch_nfe = e.unescape().unwrap().trim().to_string();
                        } else if prot_nfe.dh_recbto.is_empty() {
                            prot_nfe.dh_recbto = e.unescape().unwrap().trim().to_string();
                        } else if prot_nfe.n_prot.is_empty() {
                            prot_nfe.n_prot = e.unescape().unwrap().trim().to_string();
                        } else if prot_nfe.dig_val.is_empty() {
                            prot_nfe.dig_val = e.unescape().unwrap().trim().to_string();
                        } else if prot_nfe.c_stat.is_empty() {
                            prot_nfe.c_stat = e.unescape().unwrap().trim().to_string();
                        } else if prot_nfe.x_motivo.is_empty() {
                            prot_nfe.x_motivo = e.unescape().unwrap().trim().to_string();
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (),
            }
            buf.clear();
        }

        (ret_envi_nfe, prot_nfe)
    }

    #[test]
    fn test_extract_tags() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope"
xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
xmlns:xsd="http://www.w3.org/2001/XMLSchema">
    <soap:Body>
        <nfeResultMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeAutorizacao4">
            <retEnviNFe versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
                <tpAmb>1</tpAmb><verAplic>SP_NFCE_PL_009_V400</verAplic>
                <cStat>104</cStat><xMotivo>Lote processado</xMotivo>
                <cUF>35</cUF><dhRecbto>2024-11-19T15:33:35-03:00</dhRecbto><tMed>0</tMed>
            </retEnviNFe>
            <protNFe versao="4.00"><infProt>
                <tpAmb>1</tpAmb><verAplic>SP_NFCE_PL_009_V400</verAplic>
                <chNFe>35241111111111111111650010000000031629033844</chNFe>
                <dhRecbto>2024-11-19T15:33:35-03:00</dhRecbto>
                <nProt>135241147772677</nProt><digVal>1ydNJGYEkjRlalqWHyrs35O26UY=</digVal>
                <cStat>100</cStat><xMotivo>Autorizado o uso da NF-e</xMotivo>
            </infProt></protNFe>
        </nfeResultMsg>
    </soap:Body>
</soap:Envelope>"#;

        let (ret, prot) = extract_tags(xml);

        assert_eq!(ret.tp_amb, "1");
        assert_eq!(ret.ver_aplic, "SP_NFCE_PL_009_V400");
        assert_eq!(ret.c_stat, "104");
        assert_eq!(ret.x_motivo, "Lote processado");
        assert_eq!(ret.c_uf, "35");
        assert_eq!(ret.dh_recbto, "2024-11-19T15:33:35-03:00");
        assert_eq!(ret.t_med, "0");

        assert_eq!(prot.tp_amb, "1");
        assert_eq!(prot.ver_aplic, "SP_NFCE_PL_009_V400");
        assert_eq!(prot.ch_nfe, "35241111111111111111650010000000031629033844");
        assert_eq!(prot.dh_recbto, "2024-11-19T15:33:35-03:00");
        assert_eq!(prot.n_prot, "135241147772677");
        assert_eq!(prot.dig_val, "1ydNJGYEkjRlalqWHyrs35O26UY=");
        assert_eq!(prot.c_stat, "100");
        assert_eq!(prot.x_motivo, "Autorizado o uso da NF-e");
    }
}
