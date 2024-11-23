use std::error::Error;

pub fn get_ws_url(
    service: String,
    ambiente: i8,
    uf: String,
    svn: bool,
) -> Result<String, Box<dyn Error>> {
    // Se service = NfeInutilizacao, NfeConsultaProtocolo, NfeStatusServico, NfeConsultaCadastro, RecepcaoEvento, NFeAutorizacao, NFeRetAutorizacao
    // Se ambiente 0 = produção com validade jurídica ou 1 = homologação
    // Se UF = SP, RJ, MG, etc
    // Se SVN = true, então é SVC-AN

    let url = match (service.as_str(), ambiente, uf.as_str(), svn) {
        ("NfeInutilizacao", 0, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx".to_string()
        }
        ("NfeConsultaProtocolo", 0, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx".to_string()
        }
        ("NfeStatusServico", 0, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx".to_string()
        }
        ("NfeConsultaCadastro", 0, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx".to_string()
        }
        ("RecepcaoEvento", 0, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx".to_string()
        }
        ("NFeAutorizacao", 0, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx".to_string()
        }
        ("NFeRetAutorizacao", 0, "SP", false) => {
            "https://nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx".to_string()
        }
        ("NfeInutilizacao", 1, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeinutilizacao4.asmx".to_string()
        }
        ("NfeConsultaProtocolo", 1, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeconsultaprotocolo4.asmx".to_string()
        }
        ("NfeStatusServico", 1, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx".to_string()
        }
        ("NfeConsultaCadastro", 1, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/cadconsultacadastro4.asmx".to_string()
        }
        ("RecepcaoEvento", 1, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nferecepcaoevento4.asmx".to_string()
        }
        ("NFeAutorizacao", 1, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nfeautorizacao4.asmx".to_string()
        }
        ("NFeRetAutorizacao", 1, "SP", false) => {
            "https://homologacao.nfe.fazenda.sp.gov.br/ws/nferetautorizacao4.asmx".to_string()
        }
        ("NfeConsultaProtocolo", 0, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx"
                .to_string()
        }
        ("NfeStatusServico", 0, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx".to_string()
        }
        ("RecepcaoEvento", 0, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx".to_string()
        }
        ("NFeAutorizacao", 0, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx".to_string()
        }
        ("NFeRetAutorizacao", 0, "SP", true) => {
            "https://www.svc.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx".to_string()
        }
        ("NfeConsultaProtocolo", 1, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeConsultaProtocolo4/NFeConsultaProtocolo4.asmx"
                .to_string()
        }
        ("NfeStatusServico", 1, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeStatusServico4/NFeStatusServico4.asmx".to_string()
        }
        ("RecepcaoEvento", 1, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeRecepcaoEvento4/NFeRecepcaoEvento4.asmx".to_string()
        }
        ("NFeAutorizacao", 1, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeAutorizacao4/NFeAutorizacao4.asmx".to_string()
        }
        ("NFeRetAutorizacao", 1, "SP", true) => {
            "https://hom.svc.fazenda.gov.br/NFeRetAutorizacao4/NFeRetAutorizacao4.asmx".to_string()
        }
        // Adicione mais URLs conforme a necessidade
        _ => {
            return Err(format!("Error: Service name not found: {}", service).into());
        }
    };

    Ok(url)
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
