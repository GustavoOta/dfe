use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::io::Write;

use super::common::{cert, ws::nfe_status_servico};
use super::connection;
use super::types::service_status::Status;
use crate::nfe::decrypt;
use crate::nfe::types::config::*;

#[derive(Serialize, Deserialize, Debug)]
struct NFeConfiJson {
    cert_path: String,
    cert_pass: Password,
    federative_unit: String,
    environment: Environment,
}

#[derive(Debug)]
pub struct RawConfig {
    pub cert_path: String,
    pub cert_pass: String,
    pub federative_unit: String,
    pub environment: String,
}

pub async fn get(params: Use) -> Result<Status> {
    let raw_config = raw_config(params)?;
    let cert = cert::Cert::from_pfx(&raw_config.cert_path, &raw_config.cert_pass)?;
    let client = connection::WebService::client(cert.identity)?;

    let environment_num = match raw_config.environment.as_str() {
        "Production" => 1,
        "Homologation" => 2,
        _ => 2,
    };
    // TODO: implement svc parameter fallback
    let url = nfe_status_servico(environment_num, &raw_config.federative_unit, false)?;

    let response = connection::WebService::send(
        client,
        url,
        r#"<?xml version="1.0" encoding="utf-8"?><soap12:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soap12="http://www.w3.org/2003/05/soap-envelope"><soap12:Body><nfeDadosMsg xmlns="http://www.portalfiscal.inf.br/nfe/wsdl/NFeStatusServico4"><consStatServ xmlns="http://www.portalfiscal.inf.br/nfe" versao="4.00"><tpAmb>2</tpAmb><cUF>35</cUF><xServ>STATUS</xServ></consStatServ></nfeDadosMsg></soap12:Body></soap12:Envelope>"#.to_string(),
    ).await?;
    let body = response.text().await?;

    let status = response_parse(body)?;
    Ok(status)
}

fn response_parse(body: String) -> Result<Status> {
    let mut reader = Reader::from_str(&body);
    let mut buf: Vec<u32> = Vec::new();

    let mut tp_amb_content = String::new();
    let mut ver_aplic_content = String::new();
    let mut c_stat_content = String::new();
    let mut xmotivo_content = String::new();
    let mut c_uf_content = String::new();
    let mut dh_recbto_content = String::new();
    let mut t_med_content = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.name() {
                QName(b"tpAmb") => {
                    tp_amb_content = reader.read_text(e.name())?.into_owned();
                }
                QName(b"verAplic") => {
                    ver_aplic_content = reader.read_text(e.name())?.into_owned();
                }
                QName(b"cStat") => {
                    c_stat_content = reader.read_text(e.name())?.into_owned();
                }
                QName(b"xMotivo") => {
                    xmotivo_content = reader.read_text(e.name())?.into_owned();
                }
                QName(b"cUF") => {
                    c_uf_content = reader.read_text(e.name())?.into_owned();
                }
                QName(b"dhRecbto") => {
                    dh_recbto_content = reader.read_text(e.name())?.into_owned();
                }
                QName(b"tMed") => {
                    t_med_content = reader.read_text(e.name())?.into_owned();
                }
                _ => (),
            },
            Ok(Event::Eof) => {
                break Ok(Status {
                    tp_amb: tp_amb_content,
                    ver_aplic: ver_aplic_content,
                    c_stat: c_stat_content,
                    x_motivo: xmotivo_content,
                    c_uf: c_uf_content,
                    dh_recbto: dh_recbto_content,
                    t_med: t_med_content,
                });
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to read event: {}", e));
            }
            _ => (),
        }
        buf.clear();
    }
}

fn raw_config(params: Use) -> Result<RawConfig> {
    match params {
        Use::FileConfig => {
            let is_file_config = is_file_config()?;
            if !is_file_config {
                create_file_config()?;
            }
            let config: NFeConfiJson = load_config()?;
            // se cert_pass = File, então descriptografa
            let cert_pass = match config.cert_pass {
                Password::File(_) => {
                    let pass = decrypt()?;
                    Password::Phrase(pass)
                }
                Password::Phrase(pass) => Password::Phrase(pass),
            };
            // make cert_pass clean just trhe password string
            let cert_pass = match cert_pass {
                Password::Phrase(pass) => pass,
                _ => "".to_string(),
            };

            let raw_config = RawConfig {
                cert_path: config.cert_path,
                cert_pass,
                federative_unit: config.federative_unit,
                environment: format!("{:?}", config.environment),
            };
            Ok(raw_config)
        }
        Use::ManualConfig(fields) => {
            let config = NFeConfiJson {
                cert_path: fields.cert_path,
                cert_pass: fields.cert_pass,
                federative_unit: fields.federative_unit,
                environment: fields.environment,
            };
            println!("{:?}", config);

            let cert_pass = match config.cert_pass {
                Password::File(_) => {
                    let pass = decrypt()?;
                    Password::Phrase(pass)
                }
                Password::Phrase(pass) => Password::Phrase(pass),
            };
            // make cert_pass clean just trhe password string
            let cert_pass = match cert_pass {
                Password::Phrase(pass) => pass,
                _ => "".to_string(),
            };

            let raw_config = RawConfig {
                cert_path: config.cert_path,
                cert_pass,
                federative_unit: config.federative_unit,
                environment: format!("{:?}", config.environment),
            };
            Ok(raw_config)
        }
    }
}

fn is_file_config() -> Result<bool> {
    // Verifique se o arquivo de configuração existe
    let file_config = "nfe_config.json";
    Ok(std::path::Path::new(file_config).exists())
}

/// generate the file nfe_config.json at the root of the project
/// with the following content:
/// {
///    "cert_path": "path/to/cert.pfx",
///    "cert_pass": "String" or "FilePath",
///    "federative_unit": "SP",
///    "environment": "Production" or "Homologation",
/// }
fn create_file_config() -> Result<bool> {
    let file_config = "nfe_config.json";
    let nfe_config = NFeConfiJson {
        cert_path: "path/to/cert.pfx".to_string(),
        cert_pass: Password::File(PassFile {
            path:
                "path/to/cert_pass.txt Ps: To generate the file use the dfe::nfe::crypt() function. Use dfe::nfe::decrypt) to get the password"
                    .to_string(),
        }),
        federative_unit: "SP".to_string(),
        environment: Environment::Homologation,
    };

    let nfe_config_str =
        serde_json::to_string(&nfe_config).context("Failed to serialize config")?;

    let mut file = std::fs::File::create(file_config).context("Failed to create file")?;
    file.write_all(nfe_config_str.as_bytes())
        .context("Failed to write to file")?;

    Ok(true)
}

fn load_config() -> Result<NFeConfiJson> {
    let file_config = "nfe_config.json";
    let file = std::fs::File::open(file_config).context("Failed to open file")?;
    let reader = std::io::BufReader::new(file);
    let config: NFeConfiJson =
        serde_json::from_reader(reader).context("Failed to deserialize config")?;
    Ok(config)
}
