use crate::error::Result;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Identity;
use std::time::Duration;

#[derive(Debug)]
pub struct WebService {}

impl WebService {
    /// Constrói um cliente reqwest com o certificado A1 na camada TLS (mTLS exigido pela SEFAZ).
    /// Timeout de 30 s: cobre respostas lentas sem bloquear indefinidamente.
    /// Usado pelo [`crate::interno::transporte::MtlsTransport`].
    pub fn client(identity: Identity) -> Result<Client> {
        let client = Self::builder()
            .identity(identity)
            .timeout(Duration::from_secs(30))
            .build()?;
        Ok(client)
    }

    /// Ponto de partida de **todo** cliente HTTP que fala com a SEFAZ.
    ///
    /// Fora do Windows soma a cadeia ICP-Brasil (`data/icp_brasil.pem`) à lista de ACs do
    /// sistema: a maioria dos servidores da SEFAZ usa certificado ICP-Brasil, que o Linux não
    /// conhece — e vários não mandam a AC intermediária, que o Windows baixa sozinho e o
    /// OpenSSL não. No Windows nada muda (SChannel + repositório do sistema).
    pub fn builder() -> ClientBuilder {
        let builder = Client::builder();
        #[cfg(not(target_os = "windows"))]
        let builder = icp_brasil::raizes()
            .iter()
            .cloned()
            .fold(builder, ClientBuilder::add_root_certificate);
        builder
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) mod icp_brasil {
    use reqwest::Certificate;
    use std::sync::OnceLock;

    /// Gerado por `data/icp_brasil_gerar.sh` (cadeia publicada pelo ITI). Não editar à mão.
    pub(crate) const PEM: &[u8] = include_bytes!("../data/icp_brasil.pem");

    /// Cadeia lida uma vez por processo. Arquivo ilegível é bug de build (o teste o pega), não
    /// erro de quem emite: aí segue só com a lista do sistema.
    pub(crate) fn raizes() -> &'static [Certificate] {
        static RAIZES: OnceLock<Vec<Certificate>> = OnceLock::new();
        RAIZES.get_or_init(|| Certificate::from_pem_bundle(PEM).unwrap_or_default())
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use super::icp_brasil;
    use openssl::asn1::Asn1Time;
    use openssl::x509::X509;

    fn certificados() -> Vec<X509> {
        X509::stack_from_pem(icp_brasil::PEM).expect("icp_brasil.pem ilegível")
    }

    #[test]
    fn cadeia_icp_brasil_carrega_inteira() {
        let n = certificados().len();
        assert!(n > 100, "icp_brasil.pem com só {n} certificados");
        assert_eq!(icp_brasil::raizes().len(), n, "reqwest não leu todos os certificados");
    }

    /// As ACs que os servidores da SEFAZ usavam em 15-11-2026 (SP, PR, RS/SVRS, BA, GO, MS,
    /// MT, AM, PE). Se uma sumir na regeneração, aquele estado para de emitir no Linux.
    #[test]
    fn cadeia_tem_as_acs_dos_servidores_da_sefaz() {
        let nomes: Vec<String> = certificados()
            .iter()
            .filter_map(|c| {
                c.subject_name()
                    .entries_by_nid(openssl::nid::Nid::COMMONNAME)
                    .next()
                    .and_then(|e| e.data().to_string().ok())
            })
            .collect();
        for ac in [
            "Autoridade Certificadora Raiz Brasileira v10",
            "AC SOLUTI SSL EV G4",
            "AC Certisign ICP-Brasil SSL EV G4",
            "Autoridade Certificadora do SERPRO SSLv1",
        ] {
            assert!(nomes.iter().any(|n| n == ac), "falta na cadeia: {ac}");
        }
    }

    /// Certificado vencido na cadeia = arquivo velho: rodar `data/icp_brasil_gerar.sh`.
    /// Os que vencem em menos de 90 dias só aparecem como aviso.
    #[test]
    fn cadeia_sem_certificado_vencido() {
        let hoje = Asn1Time::days_from_now(0).unwrap();
        let em_90 = Asn1Time::days_from_now(90).unwrap();
        let mut vencidos = Vec::new();
        for c in certificados() {
            let nome = format!("{:?}", c.subject_name());
            if c.not_after() < hoje {
                vencidos.push(nome);
            } else if c.not_after() < em_90 {
                eprintln!("aviso: vence em menos de 90 dias ({}): {nome}", c.not_after());
            }
        }
        assert!(vencidos.is_empty(), "regerar icp_brasil.pem — vencidos: {vencidos:#?}");
    }
}

/// Camada `sefaz-live` (rede, sem certificado de cliente e sem enviar documento): o cliente da
/// crate confia no certificado de **cada** servidor da SEFAZ da tabela `webservices.json`.
/// Recusa por falta de certificado do cliente é esperada; só reprova erro de confiança no
/// servidor. Host que não resolve/conecta só aparece no relatório (ex.: SVC-AN fora do ar).
#[cfg(all(test, feature = "sefaz-live"))]
mod tests_live_tls {
    use super::WebService;
    use std::collections::BTreeSet;
    use std::time::Duration;

    const ERROS_DE_CONFIANCA: [&str; 6] = [
        "certificate verify failed",
        "unable to get local issuer",
        "self-signed certificate",
        "self signed certificate",
        "not trusted",
        "UnknownIssuer",
    ];

    fn hosts() -> BTreeSet<String> {
        let json = include_str!("../data/webservices.json");
        regex::Regex::new(r#"https://([^/"]+)"#)
            .unwrap()
            .captures_iter(json)
            .map(|c| c[1].to_string())
            .chain(["hom1.nfe.fazenda.gov.br".into(), "www1.nfe.fazenda.gov.br".into()])
            .collect()
    }

    #[tokio::test]
    async fn live_tls_confia_em_todos_os_servidores_da_sefaz() {
        let client = WebService::builder().timeout(Duration::from_secs(15)).build().unwrap();
        let (mut ok, mut sem_conexao, mut desconfia) = (0, Vec::new(), Vec::new());
        for h in hosts() {
            match client.get(format!("https://{h}/")).send().await {
                Ok(_) => ok += 1,
                Err(e) => {
                    let texto = format!("{e:?}");
                    if ERROS_DE_CONFIANCA.iter().any(|t| texto.contains(t)) {
                        desconfia.push(format!("{h}: {texto}"));
                    } else if e.is_connect() && (texto.contains("dns") || texto.contains("resolve") || texto.contains("timed out")) {
                        sem_conexao.push(h);
                    } else {
                        ok += 1; // TLS confiou; o servidor recusou depois (sem cert de cliente)
                    }
                }
            }
        }
        println!("confiou: {ok} | sem conexão: {sem_conexao:?}");
        assert!(desconfia.is_empty(), "servidor SEFAZ não confiável:\n{}", desconfia.join("\n"));
        assert!(ok > 30, "poucos servidores alcançados ({ok}) — sem rede?");
    }

    /// Prova de que o teste acima detecta o problema: no Linux, um cliente **sem** a cadeia
    /// ICP-Brasil recusa o servidor de SP (motivo de `WebService::builder` existir).
    #[cfg(not(target_os = "windows"))]
    #[tokio::test]
    async fn live_tls_sem_a_cadeia_icp_brasil_o_linux_recusa_sp() {
        let puro = reqwest::Client::builder().timeout(Duration::from_secs(15)).build().unwrap();
        let e = puro
            .get("https://homologacao.nfe.fazenda.sp.gov.br/")
            .send()
            .await
            .expect_err("SP passou sem a cadeia ICP-Brasil — o sistema já a tem?");
        let texto = format!("{e:?}");
        assert!(ERROS_DE_CONFIANCA.iter().any(|t| texto.contains(t)), "erro inesperado: {texto}");
    }
}
