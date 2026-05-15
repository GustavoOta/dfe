use crate::distribuicao::{
    CienciaOperacao, ConfirmacaoOperacao, Consulta, ConsultaChaveAcesso, ConsultaNSU,
    DesconhecimentoOperacao, DistribuicaoResposta, OperacaoNaoRealizada,
};

use std::{collections::HashMap, fs, path::Path};

#[tokio::test]
async fn test_consulta() {
    // Usar um arquivo de configuração para evitar hardcoding de dados sensíveis no código de teste. O arquivo deve conter:
    // CERT_PATH: caminho para o arquivo PFX do certificado digital
    // CERT_PASS: senha do certificado digital
    // CNPJ: CNPJ do destinatário sem formatação
    // Esse arquivo é usado apenas para fins de teste e não deve ser incluído no controle de versão.
    // O teste irá criar um arquivo de configuração padrão se ele não existir,
    // e falhará com uma mensagem clara solicitando que o usuário preencha os dados necessários antes de executar o teste novamente.
    let config = load_test_config();

    let result = Consulta::new()
        .cert_path(&config.cert_path) // Caminho para o arquivo PFX do certificado digital
        .cert_pass(&config.cert_pass) // Senha do certificado digital
        .cnpj(&config.cnpj) // CNPJ do destinatário sem formatação
        .uf(35) // 35 = SP
        .ambiente(1) // 1- Produção 2- Homologação
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao consultar distribuição: {:?}",
        result
    );

    let distribuicao = result.unwrap();
    println!("Distribuição Resposta:\n{:#?}", distribuicao);
}

#[tokio::test]

async fn teste_consulta_nsu() {
    let config = load_test_config();

    let result = ConsultaNSU::new()
        .cert_path(&config.cert_path)
        .cert_pass(&config.cert_pass)
        .cnpj(&config.cnpj)
        .uf(35)
        .ambiente(1)
        .nsu("000000000000083")
        .check_flag()
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao consultar distribuição por NSU: {:?}",
        result
    );

    let distribuicao = result.unwrap();
    println!("Distribuição Resposta por NSU:\n{:#?}", distribuicao);
}

#[tokio::test]
async fn teste_consulta_chave_acesso() {
    let config = load_test_config();

    let result: Result<DistribuicaoResposta, String> = ConsultaChaveAcesso::new()
        .cert_path(&config.cert_path)
        .cert_pass(&config.cert_pass)
        .cnpj(&config.cnpj)
        .uf(35)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393")
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao consultar distribuição por chave de acesso: {:?}",
        result
    );

    let distribuicao = result.unwrap();
    println!(
        "Distribuição Resposta por Chave de Acesso:\n{:#?}",
        distribuicao
    );
}

#[tokio::test]
async fn teste_ciencia_operacao() {
    let config = load_manifestacao_test_config();

    let result = CienciaOperacao::new()
        .cert_path(&config.cert_path)
        .cert_pass(&config.cert_pass)
        .cnpj(&config.cnpj)
        .ambiente(2)
        .chave_acesso("35260402084385000148550010008921991199917393") // Chave de acesso de teste
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao enviar ciência da operação: {:?}",
        result
    );

    let response = result.unwrap();
    println!("Resposta ciência da operação:\n{:#?}", response);
}

#[tokio::test]
async fn teste_confirmacao_operacao() {
    let config = load_manifestacao_test_config();

    let result = ConfirmacaoOperacao::new()
        .cert_path(&config.cert_path)
        .cert_pass(&config.cert_pass)
        .cnpj(&config.cnpj)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393") // Chave de acesso de teste
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao enviar confirmação da operação: {:?}",
        result
    );

    let response = result.unwrap();
    println!("Resposta confirmação da operação:\n{:#?}", response);
}

#[tokio::test]
async fn teste_desconhecimento_operacao() {
    let config = load_manifestacao_test_config();

    let result = DesconhecimentoOperacao::new()
        .cert_path(&config.cert_path)
        .cert_pass(&config.cert_pass)
        .cnpj(&config.cnpj)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393") // Chave de acesso de teste
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao enviar desconhecimento da operação: {:?}",
        result
    );

    let response = result.unwrap();
    println!("Resposta desconhecimento da operação:\n{:#?}", response);
}

#[tokio::test]
async fn teste_operacao_nao_realizada() {
    let config = load_manifestacao_test_config();

    let result = OperacaoNaoRealizada::new()
        .cert_path(&config.cert_path)
        .cert_pass(&config.cert_pass)
        .cnpj(&config.cnpj)
        .ambiente(1)
        .chave_acesso("35260402084385000148550010008921991199917393") // Chave de acesso de teste
        .justificativa("Operacao nao realizada para teste")
        .send()
        .await;

    assert!(
        result.is_ok(),
        "Falha ao enviar operação não realizada: {:?}",
        result
    );

    let response = result.unwrap();
    println!("Resposta operação não realizada:\n{:#?}", response);
}

struct TestConfig {
    cert_path: String,
    cert_pass: String,
    cnpj: String,
}

fn load_test_config() -> TestConfig {
    let config_path = Path::new("distribuicao.conf.ini");

    if !config_path.exists() {
        create_default_config(config_path);
        panic!(
            "Arquivo de configuração criado em '{}'. Preencha CERT_PATH, CERT_PASS e CNPJ antes de executar o teste.",
            config_path.display()
        );
    }

    let contents = fs::read_to_string(config_path).unwrap_or_else(|error| {
        panic!(
            "Falha ao ler o arquivo de configuração '{}': {}",
            config_path.display(),
            error
        )
    });
    let values = parse_config(&contents);

    TestConfig {
        cert_path: require_config_value(&values, "CERT_PATH", config_path),
        cert_pass: require_config_value(&values, "CERT_PASS", config_path),
        cnpj: require_config_value(&values, "CNPJ", config_path),
    }
}

fn create_default_config(config_path: &Path) {
    let default_contents = [
        "# Configuração local para o teste de distribuição.",
        "# Preencha os valores abaixo com os dados do seu ambiente antes de executar o teste.",
        "# CERT_PATH: caminho completo para o arquivo .pfx do certificado digital.",
        "# CERT_PASS: senha do certificado digital.",
        "# CNPJ: CNPJ do destinatário, somente números.",
        "",
        "CERT_PATH=",
        "CERT_PASS=",
        "CNPJ=",
        "",
    ]
    .join("\n");

    fs::write(config_path, default_contents).unwrap_or_else(|error| {
        panic!(
            "Falha ao criar o arquivo de configuração '{}': {}",
            config_path.display(),
            error
        )
    });
}

fn load_manifestacao_test_config() -> TestConfig {
    load_test_config()
}

fn parse_config(contents: &str) -> HashMap<String, String> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                return None;
            }

            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn require_config_value(values: &HashMap<String, String>, key: &str, config_path: &Path) -> String {
    let value = values
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            panic!(
                "A chave '{}' não foi preenchida em '{}'.",
                key,
                config_path.display()
            )
        });

    value.to_string()
}
