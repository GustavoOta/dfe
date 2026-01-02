use std::fs::File;

pub struct FlagAutorizacao;

#[derive(Debug)]
pub enum FlagAutorizacaoEnum {
    Ready,
    XMLGerado,
    Requested,
    Autorizado,
    NaoAutorizado,
    SemResposta,
}

impl FlagAutorizacao {
    pub async fn start() -> Result<FlagAutorizacaoEnum, String> {
        match FlagAutorizacao::is_env_file() {
            true => {
                // leia o arquivo flag_autorizacao.env
                let contents = std::fs::read_to_string("flag_autorizacao.env")
                    .map_err(|e| format!("Erro ao ler o arquivo: {}", e))?;
                for line in contents.lines() {
                    if line.starts_with("FlagAutorizacao=") {
                        let flag_value = line.trim_start_matches("FlagAutorizacao=");
                        return match flag_value {
                            "Ready" => Ok(FlagAutorizacaoEnum::Ready),
                            "XMLGerado" => Ok(FlagAutorizacaoEnum::XMLGerado),
                            "Requested" => Ok(FlagAutorizacaoEnum::Requested),
                            "Autorizado" => Ok(FlagAutorizacaoEnum::Autorizado),
                            "NaoAutorizado" => Ok(FlagAutorizacaoEnum::NaoAutorizado),
                            "SemResposta" => Ok(FlagAutorizacaoEnum::SemResposta),
                            other => Err(format!(
                                "Valor desconhecido para FlagAutorizacao: {}",
                                other
                            )),
                        };
                    }
                }
                Err("FlagAutorizacao não encontrada no arquivo".to_string())
            }
            false => Err("Arquivo .env não encontrado".to_string()),
        }
    }
    fn is_env_file() -> bool {
        match File::open("flag_autorizacao.env") {
            Ok(_) => true,
            Err(_) => {
                // Se o arquivo não existir, cria com a flag Ready
                use std::io::Write;
                match File::create("flag_autorizacao.env") {
                    Ok(mut file) => {
                        if let Err(e) = writeln!(file, "FlagAutorizacao=Ready") {
                            eprintln!("Erro ao escrever no arquivo: {}", e);
                            return false;
                        }
                        true
                    }
                    Err(e) => {
                        eprintln!("Erro ao criar o arquivo: {}", e);
                        false
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_env_file() {
        print!("Teste de verificação do arquivo flag_autorizacao.env\n");
        let result = FlagAutorizacao::start().await;
        match result {
            Ok(flag) => match flag {
                FlagAutorizacaoEnum::Ready => {
                    print!("FlagAutorizacaoEnum::Ready\n");
                }
                FlagAutorizacaoEnum::XMLGerado => {
                    print!("FlagAutorizacaoEnum::XMLGerado\n");
                }
                FlagAutorizacaoEnum::Requested => {
                    print!("FlagAutorizacaoEnum::Requested\n");
                }
                FlagAutorizacaoEnum::Autorizado => {
                    print!("FlagAutorizacaoEnum::Autorizado\n");
                }
                FlagAutorizacaoEnum::NaoAutorizado => {
                    print!("FlagAutorizacaoEnum::NaoAutorizado\n");
                }
                FlagAutorizacaoEnum::SemResposta => {
                    print!("FlagAutorizacaoEnum::SemResposta\n");
                }
            },
            Err(e) => {
                print!("Erro: {}\n", e);
            }
        }
    }
}
