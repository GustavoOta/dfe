use crate::interno::cert::Cert;
use crate::manifestacao::{
    nfe_ciencia_operacao, nfe_confirmacao_operacao, nfe_desconhecimento_operacao,
    nfe_operacao_nao_realizada,
};
use crate::tipos::manifestacao::{
    Manifestacao as NfeManifestacao, OperacaoNaoRealizada as NfeOperacaoNaoRealizada,
};
use base64::Engine;
use chrono::Local;
use flate2::read::GzDecoder;
use quick_xml::de::from_str;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use super::{
    CienciaOperacao, ConfirmacaoOperacao, Consulta, ConsultaChaveAcesso, ConsultaNSU,
    DesconhecimentoOperacao, DistribuicaoResposta, ManifestacaoResposta, OperacaoNaoRealizada,
};

impl Consulta {
    pub(crate) async fn executar_consulta(&self) -> Result<DistribuicaoResposta, String> {
        let xml = format!(
            "<distDFeInt xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"1.01\"><tpAmb>{}</tpAmb><cUFAutor>{}</cUFAutor><CNPJ>{}</CNPJ><distNSU><ultNSU>000000000000000</ultNSU></distNSU></distDFeInt>",
            self.ambiente, self.uf, self.cnpj
        );

        self.enviar_soap12_xml(&xml).await
    }

    pub async fn enviar_soap12_xml(&self, xml: &str) -> Result<DistribuicaoResposta, String> {
        if self.check_flag.unwrap_or(false) {
            self.validar_flag_pendente()?;
        }

        let cert = Cert::from_pfx(&self.cert_path, &self.cert_pass).map_err(|e| {
            let mensagem = format!("Erro ao criar a identidade do certificado PKCS12: {}", e);
            self.registrar_erro_flag(&mensagem);
            self.log_and_return_error(mensagem)
        })?;

        let client = reqwest::Client::builder()
            .identity(cert.identity)
            .build()
            .map_err(|e| {
                let mensagem = format!("Erro ao construir o cliente HTTP: {}", e);
                self.registrar_erro_flag(&mensagem);
                self.log_and_return_error(mensagem)
            })?;

        let endpoint = if self.ambiente == 1 {
            "https://www1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx"
        } else {
            "https://hom1.nfe.fazenda.gov.br/NFeDistribuicaoDFe/NFeDistribuicaoDFe.asmx"
        };

        let soap_envelope = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>\
                <soap12:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap12=\"http://www.w3.org/2003/05/soap-envelope\" xmlns:nfe=\"http://www.portalfiscal.inf.br/nfe/wsdl/NFeDistribuicaoDFe\">\
                    <soap12:Header>\
                        <nfe:nfeCabecMsg>\
                            <nfe:cUFAutor>{}</nfe:cUFAutor>\
                            <nfe:versaoDados>1.01</nfe:versaoDados>\
                        </nfe:nfeCabecMsg>\
                    </soap12:Header>\
                    <soap12:Body>\
                        <nfe:nfeDistDFeInteresse>\
                            <nfe:nfeDadosMsg>{}</nfe:nfeDadosMsg>\
                        </nfe:nfeDistDFeInteresse>\
                    </soap12:Body>\
                </soap12:Envelope>",
            self.uf, xml
        );

        self.salvar_log("requests", "xml", &soap_envelope)?;

        let response = client
            .post(endpoint)
            .header(
                "Content-Type",
                "application/soap+xml; charset=utf-8; action=\"http://www.portalfiscal.inf.br/nfe/wsdl/NFeDistribuicaoDFe/nfeDistDFeInteresse\"",
            )
            .body(soap_envelope)
            .send()
            .await
            .map_err(|e| {
                let mensagem = format!("Erro ao enviar a requisição: {}", e);
                self.registrar_erro_flag(&mensagem);
                self.log_and_return_error(mensagem)
            })?;

        let body = response.text().await.map_err(|e| {
            let mensagem = format!("Erro ao ler o corpo da resposta: {}", e);
            self.registrar_erro_flag(&mensagem);
            self.log_and_return_error(mensagem)
        })?;

        self.salvar_log("responses", "xml", &body)?;
        self.salvar_nsu_descompactado(&body);

        let re = Regex::new(r"(?s)<retDistDFeInt\b[^>]*>.*?</retDistDFeInt>").map_err(|e| {
            let mensagem = format!("Erro ao compilar regex da resposta SOAP: {}", e);
            self.registrar_erro_flag(&mensagem);
            self.log_and_return_error(mensagem)
        })?;

        let ret_xml = re
            .find(&body)
            .ok_or_else(|| {
                let mensagem = format!(
                    "Tag retDistDFeInt não encontrada no SOAP de resposta: {}",
                    body
                );
                self.registrar_erro_flag(&mensagem);
                self.log_and_return_error(mensagem)
            })?
            .as_str();

        let parsed: DistribuicaoResposta = from_str(ret_xml).map_err(|e| {
            let mensagem = format!("Erro ao desserializar retDistDFeInt: {}", e);
            self.registrar_erro_flag(&mensagem);
            self.log_and_return_error(mensagem)
        })?;

        self.salvar_arquivo_flag(0, "Processamento concluído com sucesso", Some(&parsed))?;

        Ok(parsed)
    }

    fn validar_flag_pendente(&self) -> Result<(), String> {
        let flag_dir = self.flag_dir()?;

        if !flag_dir.exists() {
            fs::create_dir_all(&flag_dir).map_err(|e| {
                self.log_and_return_error(format!(
                    "Erro ao criar diretório de flag em {}: {}",
                    flag_dir.display(),
                    e
                ))
            })?;

            return Ok(());
        }

        let flag_path = flag_dir.join("flag.json");
        if flag_path.exists() {
            let mensagem = format!(
                "Existe um arquivo pendente para processamento em {}.",
                flag_path.display()
            );
            return Err(self.log_and_return_error(mensagem));
        }

        Ok(())
    }

    fn registrar_erro_flag(&self, mensagem: &str) {
        let _ = self.salvar_arquivo_flag::<DistribuicaoResposta>(1, mensagem, None);
    }

    fn salvar_arquivo_flag<T: Serialize>(
        &self,
        erro: u8,
        msg: &str,
        data: Option<&T>,
    ) -> Result<(), String> {
        if self.check_flag.is_none() {
            return Ok(());
        }

        let flag_dir = self.flag_dir()?;
        fs::create_dir_all(&flag_dir).map_err(|e| {
            self.log_and_return_error(format!(
                "Erro ao criar diretório de flag em {}: {}",
                flag_dir.display(),
                e
            ))
        })?;

        let flag_path = flag_dir.join("flag.json");
        let payload = serde_json::json!({
            "erro": erro,
            "msg": msg,
            "data": data,
        });

        let payload_str = serde_json::to_string_pretty(&payload).map_err(|e| {
            self.log_and_return_error(format!("Erro ao serializar arquivo de flag: {}", e))
        })?;

        fs::write(&flag_path, payload_str).map_err(|e| {
            self.log_and_return_error(format!(
                "Erro ao salvar arquivo de flag em {}: {}",
                flag_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    fn flag_dir(&self) -> Result<PathBuf, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Erro ao obter caminho do executável: {}", e))?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| "Erro ao obter diretório do executável".to_string())?;
        Ok(exe_dir.join("distribuicao-logs").join("flag"))
    }

    fn salvar_log(&self, categoria: &str, extensao: &str, conteudo: &str) -> Result<(), String> {
        let now = Local::now();
        let ano = now.format("%Y").to_string();
        let mes = now.format("%m").to_string();
        let arquivo = format!(
            "{}-{}{}.{}",
            now.format("%d"),
            now.format("%H%M%S"),
            now.format("%3f"),
            extensao
        );
        let cnpj_dir: String = self.cnpj.chars().filter(|c| c.is_ascii_digit()).collect();
        let ambiente_dir = self.ambiente_dir();

        let mut dir = PathBuf::from("./distribuicao-logs");
        dir.push(categoria);
        dir.push(cnpj_dir);
        dir.push(ambiente_dir);
        dir.push(ano);
        dir.push(mes);

        fs::create_dir_all(&dir).map_err(|e| {
            format!(
                "Erro ao criar diretórios de saída ({}) para logs da distribuição: {}",
                categoria, e
            )
        })?;

        let mut file_path = dir;
        file_path.push(arquivo);

        fs::write(&file_path, conteudo).map_err(|e| {
            format!(
                "Erro ao salvar arquivo de log da distribuição em {}: {}",
                file_path.display(),
                e
            )
        })?;

        Ok(())
    }

    fn ambiente_dir(&self) -> &'static str {
        match self.ambiente {
            1 => "producao",
            2 => "homologacao",
            _ => "desconhecido",
        }
    }

    fn log_and_return_error(&self, mensagem: String) -> String {
        let _ = self.salvar_log("errors", "txt", &mensagem);

        mensagem
    }

    fn salvar_nsu_descompactado(&self, body: &str) {
        let re = match Regex::new(r#"<docZip\s+NSU="(\d+)"[^>]*>([^<]+)</docZip>"#) {
            Ok(r) => r,
            Err(_) => return,
        };

        let dir = PathBuf::from(format!("./distribuicao-logs/nsu/{}", self.cnpj));
        if fs::create_dir_all(&dir).is_err() {
            return;
        }

        for cap in re.captures_iter(body) {
            let nsu = &cap[1];
            let b64 = cap[2].trim();

            let compressed = match base64::engine::general_purpose::STANDARD.decode(b64) {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };

            let mut decoder = GzDecoder::new(compressed.as_slice());
            let mut xml_str = String::new();
            if decoder.read_to_string(&mut xml_str).is_err() {
                continue;
            }

            let ext = if xml_str.trim_start().starts_with('<') {
                "xml"
            } else {
                "txt"
            };
            let file_path = dir.join(format!("{}-{}.{}", nsu, self.cnpj, ext));
            let _ = fs::write(&file_path, &xml_str);
        }
    }
}

impl ConsultaNSU {
    pub(crate) async fn executar_consulta_nsu(&self) -> Result<DistribuicaoResposta, String> {
        let xml = format!(
            "<distDFeInt xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"1.01\"><tpAmb>{}</tpAmb><cUFAutor>{}</cUFAutor><CNPJ>{}</CNPJ><consNSU><NSU>{}</NSU></consNSU></distDFeInt>",
            self.ambiente, self.uf, self.cnpj, self.nsu
        );

        self.enviar_soap12_xml_nsu(&xml).await
    }

    async fn enviar_soap12_xml_nsu(&self, xml: &str) -> Result<DistribuicaoResposta, String> {
        let consulta = Consulta {
            cert_path: self.cert_path.clone(),
            cert_pass: self.cert_pass.clone(),
            cnpj: self.cnpj.clone(),
            uf: self.uf,
            ambiente: self.ambiente,
            check_flag: self.check_flag,
        };

        consulta.enviar_soap12_xml(xml).await
    }
}

impl ConsultaChaveAcesso {
    pub(crate) async fn executar_consulta_chave_acesso(
        &self,
    ) -> Result<DistribuicaoResposta, String> {
        let xml = format!(
            "<distDFeInt xmlns=\"http://www.portalfiscal.inf.br/nfe\" versao=\"1.01\"><tpAmb>{}</tpAmb><cUFAutor>{}</cUFAutor><CNPJ>{}</CNPJ><consChNFe><chNFe>{}</chNFe></consChNFe></distDFeInt>",
            self.ambiente, self.uf, self.cnpj, self.chave_acesso
        );

        self.enviar_soap12_xml_chave_acesso(&xml).await
    }

    async fn enviar_soap12_xml_chave_acesso(
        &self,
        xml: &str,
    ) -> Result<DistribuicaoResposta, String> {
        let consulta = Consulta {
            cert_path: self.cert_path.clone(),
            cert_pass: self.cert_pass.clone(),
            cnpj: self.cnpj.clone(),
            uf: self.uf,
            ambiente: self.ambiente,
            check_flag: self.check_flag,
        };

        consulta.enviar_soap12_xml(xml).await
    }
}

impl CienciaOperacao {
    pub(crate) async fn executar_ciencia_operacao(&self) -> Result<ManifestacaoResposta, String> {
        let payload = NfeManifestacao {
            cert_path: self.cert_path.clone(),
            cert_pass: self.cert_pass.clone(),
            cnpj: self.cnpj.clone(),
            tp_amb: self.ambiente,
            mod_: Some(55),
            chave: self.chave_acesso.clone(),
        };

        nfe_ciencia_operacao(payload)
            .await
            .map_err(|e| e.to_string())
    }
}

impl ConfirmacaoOperacao {
    pub(crate) async fn executar_confirmacao_operacao(
        &self,
    ) -> Result<ManifestacaoResposta, String> {
        let payload = NfeManifestacao {
            cert_path: self.cert_path.clone(),
            cert_pass: self.cert_pass.clone(),
            cnpj: self.cnpj.clone(),
            tp_amb: self.ambiente,
            mod_: Some(55),
            chave: self.chave_acesso.clone(),
        };

        nfe_confirmacao_operacao(payload)
            .await
            .map_err(|e| e.to_string())
    }
}

impl DesconhecimentoOperacao {
    pub(crate) async fn executar_desconhecimento_operacao(
        &self,
    ) -> Result<ManifestacaoResposta, String> {
        let payload = NfeManifestacao {
            cert_path: self.cert_path.clone(),
            cert_pass: self.cert_pass.clone(),
            cnpj: self.cnpj.clone(),
            tp_amb: self.ambiente,
            mod_: Some(55),
            chave: self.chave_acesso.clone(),
        };

        nfe_desconhecimento_operacao(payload)
            .await
            .map_err(|e| e.to_string())
    }
}

impl OperacaoNaoRealizada {
    pub(crate) async fn executar_operacao_nao_realizada(
        &self,
    ) -> Result<ManifestacaoResposta, String> {
        let payload = NfeOperacaoNaoRealizada {
            cert_path: self.cert_path.clone(),
            cert_pass: self.cert_pass.clone(),
            cnpj: self.cnpj.clone(),
            tp_amb: self.ambiente,
            mod_: Some(55),
            chave: self.chave_acesso.clone(),
            justificativa: self.justificativa.clone(),
        };

        nfe_operacao_nao_realizada(payload)
            .await
            .map_err(|e| e.to_string())
    }
}
