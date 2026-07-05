use crate::error::{DfeError, Result};
use serde::Deserialize;
use std::sync::OnceLock;

const WEBSERVICES_JSON: &str = include_str!("../data/webservices.json");

#[derive(Deserialize)]
struct Endpoint {
    servico: String,
    ambiente: u8,
    uf: String,
    modelo: u32,
    svn: bool,
    url: String,
}

static ENDPOINTS: OnceLock<Vec<Endpoint>> = OnceLock::new();

fn endpoints() -> &'static Vec<Endpoint> {
    // `webservices.json` é um recurso embutido (`include_str!`), invariante de build.
    // Se por algum motivo não parsear, degrada para lista vazia (todo lookup vira
    // `DfeError::Webservice` "não encontrado") em vez de derrubar o worker do Actix (A2).
    // A validade real é garantida pelo teste `test_webservices_json_valido`.
    ENDPOINTS.get_or_init(|| serde_json::from_str(WEBSERVICES_JSON).unwrap_or_default())
}

fn lookup(servico: &str, ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    endpoints()
        .iter()
        .find(|e| {
            e.servico == servico
                && e.ambiente == ambiente
                && e.uf == uf
                && e.modelo == modelo
                && e.svn == svn
        })
        .map(|e| e.url.as_str())
        .ok_or_else(|| {
            DfeError::Webservice(format!(
                "Endpoint não encontrado: servico={} ambiente={} uf={} modelo={} svn={}",
                servico, ambiente, uf, modelo, svn
            ))
        })
}

pub fn nfe_status_servico(ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    lookup("NfeStatusServico", ambiente, uf, modelo, svn)
}

pub fn nfe_autorizacao(ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    lookup("NFeAutorizacao", ambiente, uf, modelo, svn)
}

pub fn nfe_recepcao_evento(ambiente: u8, uf: &str, modelo: u32, svn: bool) -> Result<&'static str> {
    lookup("RecepcaoEvento", ambiente, uf, modelo, svn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webservices_json_valido() {
        // Garante que o recurso embutido parseia (o fallback de `endpoints()` é silencioso,
        // então esta asserção é o que detecta corrupção do JSON em tempo de build/CI).
        assert!(
            !endpoints().is_empty(),
            "webservices.json embutido não parseou ou está vazio"
        );
    }

    #[test]
    fn test_get_ws_url() {
        let url = nfe_status_servico(2, "SP", 55, false);
        assert_eq!(
            url.unwrap(),
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx"
        );
    }

    #[test]
    fn test_endpoint_nao_encontrado() {
        let url = nfe_status_servico(1, "XX", 55, false);
        assert!(url.is_err());
    }

    // A6: roteamento multi-UF. UF com webservice próprio (SP), via SVRS (AC) e via SVAN (MA).
    #[test]
    fn roteamento_uf_propria_sp() {
        assert_eq!(
            nfe_autorizacao(1, "SP", 55, false).unwrap(),
            "https://nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx"
        );
    }

    #[test]
    fn roteamento_via_svrs_ac() {
        // AC delega a autorização de NF-e ao SVRS (resolvido na geração do webservices.json).
        assert_eq!(
            nfe_autorizacao(1, "AC", 55, false).unwrap(),
            "https://nfe.svrs.rs.gov.br/ws/NfeAutorizacao/NFeAutorizacao4.asmx"
        );
        // E o evento de NFC-e do AC também vai pelo SVRS.
        assert_eq!(
            nfe_recepcao_evento(2, "AC", 65, false).unwrap(),
            "https://nfce-homologacao.svrs.rs.gov.br/ws/recepcaoevento/recepcaoevento4.asmx"
        );
    }

    #[test]
    fn roteamento_via_svan_ma() {
        // MA delega a autorização de NF-e ao SVAN.
        assert_eq!(
            nfe_autorizacao(1, "MA", 55, false).unwrap(),
            "https://www.sefazvirtual.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx"
        );
    }

    #[test]
    fn todas_as_27_ufs_tem_autorizacao_55() {
        const UFS: &[&str] = &[
            "RO","AC","AM","RR","PA","AP","TO","MA","PI","CE","RN","PB","PE","AL","SE","BA",
            "MG","ES","RJ","SP","PR","SC","RS","MS","MT","GO","DF",
        ];
        for uf in UFS {
            for amb in [1u8, 2] {
                assert!(nfe_autorizacao(amb, uf, 55, false).is_ok(), "sem NFeAutorizacao 55 p/ {uf} amb {amb}");
                assert!(nfe_autorizacao(amb, uf, 65, false).is_ok(), "sem NFeAutorizacao 65 p/ {uf} amb {amb}");
                assert!(nfe_recepcao_evento(amb, uf, 55, false).is_ok(), "sem RecepcaoEvento 55 p/ {uf} amb {amb}");
                assert!(nfe_status_servico(amb, uf, 55, false).is_ok(), "sem StatusServico 55 p/ {uf} amb {amb}");
            }
        }
    }
}
