use anyhow::*;

pub fn nfe_status_servico<'a>(
    ambiente: u8,
    uf: &'a str,
    modelo: u32,
    svn: bool,
) -> Result<&'a str> {
    let url = url("NfeStatusServico", ambiente, uf, modelo, svn);
    Ok(url.unwrap())
}

pub fn nfe_autorizacao<'a>(ambiente: u8, uf: &'a str, modelo: u32, svn: bool) -> Result<&'a str> {
    let url = url("NFeAutorizacao", ambiente, uf, modelo, svn);
    Ok(url.unwrap())
}

pub fn nfe_recepcao_evento<'a>(
    ambiente: u8,
    uf: &'a str,
    modelo: u32,
    svn: bool,
) -> Result<&'a str> {
    let url = url("RecepcaoEvento", ambiente, uf, modelo, svn);
    Ok(url.unwrap())
}

/// Retorna a URL do Web Service conforme o serviço, ambiente, UF e SVN
/// # Exemplo
/// ```
/// use dfe::ws::Get;
/// let url = Get::nfe_status_servico("NfeStatusServico", 2, "SP", false);
/// println!("URL: {:?}", url);
/// ```
fn url<'a>(service: &'a str, ambiente: u8, uf: &'a str, modelo: u32, svn: bool) -> Result<&'a str> {
    // Se service = NfeInutilizacao, NfeConsultaProtocolo, NfeStatusServico, NfeConsultaCadastro, RecepcaoEvento, NFeAutorizacao, NFeRetAutorizacao
    // Se ambiente 0 = produção com validade jurídica ou 1 = homologação
    // Se UF = SP, RJ, MG, etc
    // se modelo = 55 ou 65
    // Se SVN = true, então é SVC-AN
    if modelo != 55 && modelo != 65 {
        return Err(anyhow!("Modelo inválido"));
    }
    // NFCE
    if modelo == 65 && service == "NFeAutorizacao" {
        let url = match ambiente {
            1 => "https://nfce.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx",
            2 => "https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx",
            _ => {
                return Err(anyhow!("Ambiente inválido"));
            }
        };
        return Ok(url);
    }

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
        let url = url("NfeStatusServico", 2, "SP", 55, false);
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


// NFCE webservices
WebServices
Endereços dos WebServices disponíveis na SEFAZ/SP no ambiente de homologação


1.1 WebServices do ambiente de homologação da versão 4.00 (NT2016.002) da SEFAZ/SP:

Serviço​
Endereço​
NfeAutorização4
ttps://homologacao.nfce.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx                       ​
NfeRetAutorizacao4​
​
https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeRetAutorizacao4.asmx
NfeInutilizacao4​

https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeInutilizacao4.asmx​
NfeConsultaProtocolo4​
https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeConsultaProtocolo4.asmx​
​NfeRecepcaoEvento4
https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeRecepcaoEvento4.asmx​
​NfeStatusServico4
https://homologacao.nfce.fazenda.sp.gov.br/ws/NFeStatusServico4.asmx​
1.2 WebService do ambiente de homologação da contingência EPEC:

Serviço​​
Endereço​​
RecepcaoEPEC​	https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asm​
NfeStatusServico2 ​
https://homologacao.nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asmx
​
2.  URL QR Code


A URL a ser utilizada na consulta via QR Code no ambiente de homologação deverá ser:

​https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx
https://www.homologacao.nfce.fazenda.sp.gov.br/qrcode​
3.  URL Consulta Pública

https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica     ​
https://www.homologacao.nfce.fazenda.sp.gov.br/consulta​



URL WEBSERVICES - AMBIENTE DE PRODUÇÃO

1. Endereços dos WebServices disponíveis na SEFAZ/SP no ambiente de produção

1.1 WebServices do ambiente de produção da versão 4.00 (NT2016.002) da SEFAZ/SP:
NfeAutorização4​
https://nfce.fazenda.sp.gov.br/ws/NFeAutorizacao4.asmx ​
NfeRetAutorizacao4​	https://nfce.fazenda.sp.gov.br/ws/NFeRetAutorizacao4.asmx​
NfeInutilizacao4​	https://nfce.fazenda.sp.gov.br/ws/NFeInutilizacao4.asmx ​
​NfeConsultaProtocolo4​
https://nfce.fazenda.sp.gov.br/ws/NFeConsultaProtocolo4.asmx ​​
​NfeRecepcaoEvento4
https://nfce.fazenda.sp.gov.br/ws/NFeRecepcaoEvento4.asmx ​
​NfeStatusServico4
https://nfce.fazenda.sp.gov.br/ws/NFeStatusServico4.asmx​


1.2 WebService do ambiente de produção da contingência EPEC:

Serviço​​
Endereço​​
RecepcaoEPEC	https://nfce.epec.fazenda.sp.gov.br/EPECws/RecepcaoEPEC.asm
NfeStatusServico2
https://nfce.epec.fazenda.sp.gov.br/EPECws/EPECStatusServico.asm



2.  URL QR Code

A URL a ser utilizada na consulta via QR Code no ambiente de produção deverá ser:

https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx
https://www.nfce.fazenda.sp.gov.br/qrcode



3. Consulta Pública

https://www.nfce.fazenda.sp.gov.br/NFCeConsultaPublica
*/
