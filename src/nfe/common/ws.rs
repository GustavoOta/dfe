use anyhow::*;

pub fn nfe_status_servico(ambiente: i8, uf: &str, svn: bool) -> Result<&str> {
    let url = url("NfeStatusServico", ambiente, uf, svn);
    Ok(url.unwrap())
}
/// Retorna a URL do Web Service conforme o serviço, ambiente, UF e SVN
/// # Exemplo
/// ```
/// use dfe::ws::Get;
/// let url = Get::nfe_status_servico("NfeStatusServico", 2, "SP", false);
/// println!("URL: {:?}", url);
/// ```
fn url<'a>(service: &'a str, ambiente: i8, uf: &'a str, svn: bool) -> Result<&'a str> {
    // Se service = NfeInutilizacao, NfeConsultaProtocolo, NfeStatusServico, NfeConsultaCadastro, RecepcaoEvento, NFeAutorizacao, NFeRetAutorizacao
    // Se ambiente 0 = produção com validade jurídica ou 1 = homologação
    // Se UF = SP, RJ, MG, etc
    // Se SVN = true, então é SVC-AN

    let url = match (service, ambiente, uf, svn) {
        ("NfeInutilizacao", 1, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx"
        }
        ("NfeConsultaProtocolo", 1, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx"
        }
        ("NfeStatusServico", 1, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx"
        }
        ("NfeConsultaCadastro", 1, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx"
        }
        ("RecepcaoEvento", 1, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx"
        }
        ("NFeAutorizacao", 1, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx"
        }
        ("NFeRetAutorizacao", 1, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx"
        }
        ("NfeInutilizacao", 2, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx"
        }
        ("NfeConsultaProtocolo", 2, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx"
        }
        ("NfeStatusServico", 2, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx"
        }
        ("NfeConsultaCadastro", 2, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx"
        }
        ("RecepcaoEvento", 2, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx"
        }
        ("NFeAutorizacao", 2, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx"
        }
        ("NFeRetAutorizacao", 2, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx"
        }
        ("NfeConsultaProtocolo", 1, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx"
        }
        ("NfeStatusServico", 1, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx"
        }
        ("RecepcaoEvento", 1, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx"
        }
        ("NFeAutorizacao", 1, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx"
        }
        ("NFeRetAutorizacao", 1, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx"
        }
        ("NfeConsultaProtocolo", 2, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx"
        }
        ("NfeStatusServico", 2, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx"
        }
        ("RecepcaoEvento", 2, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx"
        }
        ("NFeAutorizacao", 2, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx"
        }
        ("NFeRetAutorizacao", 2, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx"
        }
        // Adicione mais URLs conforme a necessidade
        _ => {
            return Err(anyhow!("Service endpoint not found"));
        }
    };

    Ok(url)
}

// Teste
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ws_url() {
        let url = url("NfeStatusServico", 2, "SP", false);
        println!("URL: {:?}", url);
        assert_eq!(
            url.unwrap(),
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx"
        );
    }
}

/*
URLs dos Web Services
Endereços dos Web Services disponibilizados pela SEFAZ/SP


Ambiente de homologação externa
​Serviço	Versão​	URL​
​NfeInutilizacao
4.0​0
​https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx
​NfeConsultaProtocolo
​4.00	https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx
NfeStatusServico​
​4.00	https://homologacao.nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx
NfeConsultaCadastro​
​4.00	https://homologacao.nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx​
​RecepcaoEvento
​4.00	​https://homologacao.nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx
​NFeAutorizacao
​4.00	​https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx
NFeRetAutorizacao​
​4.00	​https://homologacao.nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx


Ambiente de produção com validade jurídica
​Serviço
Versão​
URL​​​
​​NfeInutilizacao
4.0​0	https://nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx
​NfeConsultaProtocolo
​4.00
https://nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx
NfeStatusServico​
​4.00	https://nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx
NfeConsultaCadastro​
​4.00	https://nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx
​RecepcaoEvento
​4.00	https://nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx​
​NFeAutorizacao
​4.00	https://nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx​
NFeRetAutorizacao​
​4.00	https://nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx​

Endereço dos Web Services disponibilizados pela SEFAZ Virtual de Contingência Ambiente Nacional - (SVC-AN)
Ambiente de homologação externa
​Serviço	Versão​	URL​
NfeConsultaProtocolo​
4.00​	https://hom.svc.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx
​NfeStatusServico
4.00​	https://hom.svc.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx
​RecepcaoEvento
4.00​	https://hom.svc.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx
​NFeAutorizacao
4.00​	https://hom.svc.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx
​NFeRetAutorizacao
4.00​	https://hom.svc.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx​


Ambiente de produção com validade jurídica
​Serviço
Versão​	URL​
NfeConsultaProtocolo​
4.00​	https://www.svc.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx
​NfeStatusServico
4.00​	https://www.svc.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx
​RecepcaoEvento
4.00​	https://www.svc.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx
​NFeAutorizacao
4.00​	https://www.svc.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx
​NFeRetAutorizacao
4.00​	https://www.svc.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx​




Endereço dos Web Services disponibilizados pelo Ambiente Nacional (AN)
Ambiente de homologação externa
​Serviço	Versão​	URL​
RecepcaoEvento​
4.00	https://hom.nfe.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx
NFeDistribuicaoDFe​
​1.00	https://hom.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx​

Ambiente de produção com validade jurídica
​Serviço
Versão​	URL​
RecepcaoEvento​
4.00	https://www.nfe.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx
NFeDistribuicaoDFe​
​1.00	https://www1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx​
*/
