use anyhow::{Ok, Result};
use openssl::pkcs12::Pkcs12;
//use openssl::pkey::PKey;
use base64::Engine;
use openssl::x509::X509;
use std::fs::File;
use std::io::Read;

pub struct Cert {
    pub identity: reqwest::Identity,
}

/// Certificado digital em formato PKCS12 (PFX)
/// Uso: Cert::from_pfx("caminho/para/o/certificado.pfx", "senha")
impl Cert {
    pub fn from_pfx(path: &str, password: &str) -> Result<Cert> {
        let mut buf = Vec::new();
        let mut pfx = File::open(path)?;
        pfx.read_to_end(&mut buf)?;

        let pkcs2 = reqwest::Identity::from_pkcs12_der(&buf, password)?;

        Ok(Cert { identity: pkcs2 })
    }
}

pub async fn raw_pub_key(pfx_path: &str, password: &str) -> Result<String> {
    let mut file = File::open(pfx_path)?;
    let mut der = vec![];
    file.read_to_end(&mut der)?;

    // generate a X509Certificate from the PFX file
    let pkcs12 = Pkcs12::from_der(&der)?;
    let x509: openssl::pkcs12::ParsedPkcs12_2 = pkcs12.parse2(password)?;

    // print the certificate
    let cert = x509.cert;
    if cert.is_none() {
        return Err(anyhow::anyhow!("Certificado não encontrado"));
    }
    let cert = cert.unwrap();
    let cert = X509::from_der(&cert.to_der()?)?;
    let cert = cert.to_pem()?;
    let cert = String::from_utf8(cert)?;
    // clean the certificate
    let cert = cert.replace("-----BEGIN CERTIFICATE-----", "");
    let cert = cert.replace("-----END CERTIFICATE-----", "");
    let cert = cert.replace("\n", "");
    Ok(cert)
}

pub async fn raw_private_key(pfx_path: &str, password: &str) -> Result<String> {
    let mut file = File::open(pfx_path)?;
    let mut der = vec![];
    file.read_to_end(&mut der)?;

    // generate a X509Certificate from the PFX file
    let pkcs12 = Pkcs12::from_der(&der)?;
    let x509: openssl::pkcs12::ParsedPkcs12_2 = pkcs12.parse2(password)?;

    // print the private key
    let pkey = x509.pkey;
    if pkey.is_none() {
        return Err(anyhow::anyhow!("Chave privada não encontrada"));
    }
    let pkey = pkey.unwrap();
    let pkey = pkey.private_key_to_pem_pkcs8()?;
    let pkey = String::from_utf8(pkey)?;
    // clean the private key
    let pkey = pkey.replace("-----BEGIN PRIVATE KEY-----", "");
    let pkey = pkey.replace("-----END PRIVATE KEY-----", "");
    let pkey = pkey.replace("\n", "");
    Ok(pkey)
}

pub async fn create_digest_value(xml: &str) -> Result<String> {
    let digest = openssl::sha::sha1(xml.as_bytes());
    let digest = base64::engine::general_purpose::STANDARD.encode(digest);
    Ok(digest)
}

#[cfg(test)]
#[tokio::test]
pub async fn raw_public() {
    // Returns raw public key without markers and LF's using openssl

    let result = raw_pub_key("D:/Projetos/dfe/skadao-1234.pfx", "123456")
        .await
        .unwrap();

    println!("{:?}", result);
}

#[cfg(test)]
#[tokio::test]
pub async fn create_digest() {
    // Returns a digest value for a given xml file

    let xml = r#"<infNFe Id="NFe35241254515633000161550010000000011976008519" versao="4.00">
        <ide>
            <cUF>35</cUF>
            <cNF>97600851</cNF>
            <natOp>VENDA</natOp>
            <mod>55</mod>
            <serie>1</serie>
            <nNF>1</nNF>
            <dhEmi>2024-12-06T10:06:11-03:00</dhEmi>
            <dhSaiEnt>2024-12-06T10:06:11-03:00</dhSaiEnt>
            <tpNF>1</tpNF>
            <idDest>1</idDest>
            <cMunFG>3550308</cMunFG>
            <tpImp>1</tpImp>
            <tpEmis>1</tpEmis>
            <cDV>9</cDV>
            <tpAmb>2</tpAmb>
            <finNFe>1</finNFe>
            <indFinal>1</indFinal>
            <indPres>1</indPres>
            <procEmi>0</procEmi>
            <verProc>1.0</verProc>
        </ide>
        <emit>
            <CNPJ>54515633000161</CNPJ>
            <xNome>CAIO VICTOR DE OLIVEIRA LEITE LARA LTDA:54515633000161</xNome>
            <xFant>SKADAO CAFETERIA LANCHONETE E RESTAURANTE</xFant>
            <enderEmit>
                <xLgr>RUA TESTE</xLgr>
                <nro>123</nro>
                <xBairro>CENTRO</xBairro>
                <cMun>3550308</cMun>
                <xMun>SÃO PAULO</xMun>
                <UF>SP</UF>
                <CEP>12345678</CEP>
                <cPais>1058</cPais>
                <xPais>BRASIL</xPais>
                <fone>1122223333</fone>
            </enderEmit>
            <IE>448045188118</IE>
            <CRT>1</CRT>
        </emit>
        <dest>
            <CPF>00496880578</CPF>
            <xNome>NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL</xNome>
            <enderDest>
                <xLgr>RUA TESTE</xLgr>
                <nro>123</nro>
                <xBairro>CENTRO</xBairro>
                <cMun>3550308</cMun>
                <xMun>SÃO PAULO</xMun>
                <UF>SP</UF>
                <CEP>12345678</CEP>
                <cPais>1058</cPais>
                <xPais>BRASIL</xPais>
                <fone>1122223333</fone>
            </enderDest>
            <indIEDest>9</indIEDest>
        </dest>
        <det nItem="1">
            <prod>
                <cProd>123456</cProd>
                <cEAN>SEM GTIN</cEAN>
                <xProd>PRODUTO TESTE</xProd>
                <NCM>22030000</NCM>
                <CFOP>5102</CFOP>
                <uCom>UN</uCom>
                <qCom>1.00</qCom>
                <vUnCom>10.00</vUnCom>
                <vProd>10.00</vProd>
                <cEANTrib>SEM GTIN</cEANTrib>
                <uTrib>UN</uTrib>
                <qTrib>1.00</qTrib>
                <vUnTrib>10.00</vUnTrib>
                <indTot>1</indTot>
            </prod>
            <imposto>
                <vTotTrib>10.00</vTotTrib>
                <ICMS>
                    <ICMSSN500>
                        <orig>0</orig>
                        <CSOSN>500</CSOSN>
                    </ICMSSN500>
                </ICMS>
                <PIS>
                    <PISOutr>
                        <CST>49</CST>
                        <qBCProd>0.00</qBCProd>
                        <vAliqProd>0.00</vAliqProd>
                        <vPIS>0.00</vPIS>
                    </PISOutr>
                </PIS>
                <COFINS>
                    <COFINSOutr>
                        <CST>49</CST>
                        <qBCProd>0.00</qBCProd>
                        <vAliqProd>0.00</vAliqProd>
                        <vCOFINS>0.00</vCOFINS>
                    </COFINSOutr>
                </COFINS>
            </imposto>
        </det>
        <total>
            <ICMSTot>
                <vBC>0.00</vBC>
                <vICMS>0.00</vICMS>
                <vICMSDeson>0.00</vICMSDeson>
                <vFCP>0.00</vFCP>
                <vBCST>0.00</vBCST>
                <vST>0.00</vST>
                <vFCPST>0.00</vFCPST>
                <vFCPSTRet>0.00</vFCPSTRet>
                <vProd>10.00</vProd>
                <vFrete>0.00</vFrete>
                <vSeg>0.00</vSeg>
                <vDesc>0.00</vDesc>
                <vII>0.00</vII>
                <vIPI>0.00</vIPI>
                <vIPIDevol>0.00</vIPIDevol>
                <vPIS>0.00</vPIS>
                <vCOFINS>0.00</vCOFINS>
                <vOutro>0.00</vOutro>
                <vNF>10.00</vNF>
                <vTotTrib>10.00</vTotTrib>
            </ICMSTot>
        </total>
        <transp>
            <modFrete>9</modFrete>
        </transp>
        <pag>
            <detPag>
                <tPag>01</tPag>
                <vPag>10.00</vPag>
            </detPag>
        </pag>
        <infAdic>
            <infCpl>TESTE</infCpl>
        </infAdic>
    </infNFe>"#;

    // clean /n/r/t
    let xml = xml.replace("\n", "");
    let xml = xml.replace("\r", "");
    let xml = xml.replace("\t", "");

    let result = create_digest_value(&xml).await.unwrap();
    println!("{:?}", result);
}
/*
<KeyInfo>
    <X509Data>
        <X509Certificate>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</X509Certificate>
    </X509Data>
</KeyInfo>
 */
