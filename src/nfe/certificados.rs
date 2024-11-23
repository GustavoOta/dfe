use openssl::pkcs12::Pkcs12;
use openssl::pkey::PKey;
use openssl::provider::Provider;
use openssl::x509::X509;
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
pub struct Certificado {
    pub certificado: X509,
    pub chave_privada: PKey<openssl::pkey::Private>,
}

impl Certificado {
    pub fn from_pfx(arquivo: String, senha: String) -> Result<Self, Box<dyn Error>> {
        // Carregar o provedor legado. {Bug?} se não atribuir o provedor a let provider = Provider::try_load(None, "legacy", true)?; o openssl não consegue carregar o certificado
        #[allow(unused_variables)]
        let provider = Provider::try_load(None, "legacy", true)?;

        let mut arquivo = File::open(arquivo)?;
        let mut buf = Vec::new();
        arquivo.read_to_end(&mut buf)?;
        let pkcs12 = Pkcs12::from_der(&buf)?;
        let parsed = pkcs12.parse2(&senha)?;

        Ok(Certificado {
            certificado: parsed.cert.unwrap(),
            chave_privada: parsed.pkey.unwrap(),
        })
    }
}

pub async fn load(arquivo: String, senha: String) -> Result<(), Box<dyn Error>> {
    // Lê o certificado
    match Certificado::from_pfx(arquivo, senha) {
        Ok(certificado) => {
            println!("Certificado carregado com sucesso {:?}", certificado);
            Ok(())
        }
        Err(e) => {
            eprintln!("Erro ao carregar o certificado: {:?}", e);
            Err(e)
        }
    }
}
