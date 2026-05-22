# dfe
DFE - Documentos Fiscais Eletrônicos Brasileiros.

# Doações:
[![Sponsor GustavoOta](https://img.shields.io/badge/sponsor-GustavoOta-%23EA4AAA?style=flat&logo=github)](https://github.com/sponsors/GustavoOta)


## Instalação:
-> É necessário instalar o OPENSSL no seu ambiente de desenvolvimento.
-> Copie a pasta dfe que contém os arquivos de SCHEMA para o diretório do seu executável.   
-> cargo add dfe  

## Exemplo de Uso: Emitir NFe

```rust
use dfe::nfe::autorizacao::emit;
use dfe::nfe::types::autorizacao4::*;

let teste = emit(NFe {
    cert_path: "D:/Projetos/cert.pfx".to_string(),
    cert_pass: "1234".to_string(),
    ide: Ide {
        c_uf: 35,
        serie: 1,
        n_nf: 35,
        c_mun_fg: 3550308,
        tp_emis: 1,
        tp_amb: 2,
        ind_final: 1,
        ind_pres: 1,
        mod_: 55,
        tp_imp: 1,
        ..Default::default()
    },
    emit: Emit {
        cnpj: Some("11111111111111".to_string()),
        ie: Some(111111111111),
        crt: 3,
        x_nome: "EMPRESA DE TESTE".to_string(),
        x_fant: Some("TESTANDO EMPREENDIMENTOS".to_string()),
        x_lgr: "RUA TESTE".to_string(),
        nro: "123".to_string(),
        x_bairro: "CENTRO".to_string(),
        c_mun: 3529906,
        x_mun: "SÃO PAULO".to_string(),
        uf: "SP".to_string(),
        cep: 10000000,
        ..Default::default()
    },
    dest: Dest {
        cpf: Some("07068093868".to_string()),
        //cnpj: Some("56196407000190".to_string()), // com ie
        //cnpj: Some("46395000000139".to_string()), // sem ie
        x_nome: Some("NF-E EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string()),
        x_lgr: Some("RUA TESTE".to_string()),
        nro: Some("123".to_string()),
        x_bairro: Some("CENTRO".to_string()),
        c_mun: Some(3550308),
        x_mun: Some("SÃO PAULO".to_string()),
        uf: Some("SP".to_string()),
        cep: Some(11850000),
        //c_pais: Some("1058".to_string()),
        //x_pais: Some("BRASIL".to_string()),
        //fone: Some("11999999999".to_string()),
        ind_ie_dest: Some(9),
        //ie: Some("150344006118".to_string()),
        ..Default::default()
    },
    det: vec![
        Det {
            c_prod: "123456".to_string(),
            x_prod: "NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 1.0,
            v_un_com: 10.0,
            v_prod: 10.0,
            u_trib: "CX".to_string(),
            q_trib: 1.0,
            v_un_trib: "10.00".to_string(),
            ind_tot: 1,
            // TODO: Dispobilizar todos os tipos de ICMS
            // Disponivel: -> ICMS40 ou ICMSSN102
            // orig -> 0
            // CST -> 41
            // csosn -> 102
            icms: "ICMS40".to_string(),
            // TODO: Dispobilizar
            pis: "PISNT".to_string(),
            // TODO: Dispobilizar
            cofins: "COFINSNT".to_string(),
            ..Default::default()
        },
        Det {
            c_prod: "123456".to_string(),
            x_prod: "PRODUTO TESTE 2".to_string(),
            ncm: "22030000".to_string(),
            cfop: 5102,
            u_com: "UN".to_string(),
            q_com: 2.0,
            v_un_com: 10.0,
            v_prod: 20.0,
            u_trib: "CX".to_string(),
            q_trib: 2.0,
            v_un_trib: "10.00".to_string(),
            ind_tot: 1,
            icms: "ICMS40".to_string(),
            pis: "PISNT".to_string(),
            cofins: "COFINSNT".to_string(),
            ..Default::default()
        },
    ],
    total: Total {
        v_bc: 0.0,
        v_icms: 0.0,
        v_icms_deson: 0.0,
        v_fcpuf_dest: 0.0,
        v_icms_uf_dest: 0.0,
        v_icms_uf_remet: 0.0,
        v_fcp: 0.0,
        v_bc_st: 0.0,
        v_st: 0.0,
        v_fcpst: 0.0,
        v_fcpst_ret: 0.0,
        v_prod: 30.0,
        v_frete: 0.0,
        v_seg: 0.0,
        v_desc: 0.0,
        v_ii: 0.0,
        v_ipi: 0.0,
        v_ipi_devol: 0.0,
        v_pis: 0.0,
        v_cofins: 0.0,
        v_outro: 0.0,
        v_nf: 30.0,
        v_tot_trib: 0.0,
    },
    transp: Transp {
        mod_frete: 0,
        ..Default::default()
    },
    pag: Pag {
        t_pag: "01".to_string(),
        v_pag: 30.0,
    },
    inf_adic: None,
})
.await;

if let Err(e) = teste {
    println!("Erro: {:?}", e);
} else {
    if let Ok(response) = teste {
        println!("Resposta: {}", response);
        println!("XML: {:?}", response.xml);
    }
}
```
# Exemplo de uso: Cancelar NF-e 
```rust
use dfe::nfe::cancelar::nfe_cancelar;
    use dfe::nfe::types::cancelar::*;

    let teste = nfe_cancelar(NFeCancelar {
        cert_path: "D:/Projetos/cert.pfx".to_string(),
        cert_pass: "1234".to_string(),
        tp_amb: 2,
        chave: "35241211111111111111550010000000381505051324".to_string(),
        protocolo: "1352400000006702".to_string(),
        justificativa: "TESTE DE CANCELAMENTO".to_string(),
    })
    .await;

    if let Err(e) = teste {
        println!("Erro: {:?}", e);
    } else {
        println!("Response: {:?}", teste.unwrap().response);
    }
```

# Exemplo de uso: Status do Serviço 
Webservice SP Produção
```rust
use dfe::nfe::service_status;
use dfe::nfe::types::config::*;
// TODO mudar o tipo para receber Estado, Ambiente, NFe ou NFCe Homologação ou Produção
let teste = service_status(Use::ManualConfig(Fields {
    cert_path: "D:/Projetos/cert.pfx".to_string(),
    cert_pass: Password::Phrase("1234".to_string()),
    federative_unit: "SP".to_string(),
    environment: Environment::Homologation,
}))
.await;
if teste.is_err() {
    println!("Error test_service_status_custom_pass:{:?}", teste.err());
    assert!(false);
    return;
}
let teste = teste.unwrap();

println!("tp_amb: {}", teste.tp_amb);
println!("ver_aplic: {}", teste.ver_aplic);
println!("c_stat: {}", teste.c_stat);
println!("x_motivo: {}", teste.x_motivo);
println!("c_uf: {}", teste.c_uf);
println!("dh_recbto: {}", teste.dh_recbto);
println!("t_med: {}", teste.t_med);
```

## Notas importantes:

Este software está em desenvolvimento e não deve ser usado em produção a não ser que você saiba o que está fazendo.

## DanfeBuilder - Geração de DANFE em PDF

O `DanfeBuilder` é um builder para geração de DANFE (Documento Auxiliar da Nota Fiscal Eletrônica) em PDF a partir do XML autorizado da NF-e (`nfeProc`).

### Funcionalidades

- Gera DANFE Simplificado para impressoras térmicas 80mm, 56mm ou A4
- Suporta saída como arquivo PDF ou string base64

### Tamanhos de papel suportados

| Tamanho | Modelo 55 (NF-e) | Modelo 65 (NFC-e) |
| ------- | ---------------- | ----------------- |
| `a4`    | Em breve         | Em breve          |
| `80mm`  | ✅ Disponível     | Em breve          |
| `56mm`  | Em breve         | Em breve          |

### Exemplo: Gerar DANFE como arquivo PDF

```rust
use dfe::pdf::DanfeBuilder;

let resultado = DanfeBuilder::new()
    .xml("./nota_autorizada.xml")
    .paper_size("80mm")
    .as_file("./danfe.pdf")
    .build()
    .await;

match resultado {
    Ok(path) => println!("PDF salvo em: {}", path),
    Err(e) => println!("Erro: {}", e),
}
```

### Exemplo: Gerar DANFE como base64

```rust
use dfe::pdf::DanfeBuilder;

let resultado = DanfeBuilder::new()
    .xml("<nfeProc>...</nfeProc>")
    .paper_size("80mm")
    .as_base64()
    .build()
    .await;

match resultado {
    Ok(base64) => println!("Base64: {}", base64),
    Err(e) => println!("Erro: {}", e),
}
```

### Métodos

| Método                   | Descrição                                              |
| ------------------------ | ------------------------------------------------------ |
| `new()`                  | Cria uma nova instância do builder                     |
| `xml(xml: &str)`         | Define o XML (caminho do arquivo `.xml` ou string XML) |
| `paper_size(size: &str)` | Define o tamanho do papel (`"a4"`, `"80mm"`, `"56mm"`) |
| `as_file(path: &str)`    | Configura saída como arquivo PDF no caminho indicado   |
| `as_base64()`            | Configura saída como string base64                     |
| `build()`                | Gera o PDF e retorna `Result<String, String>`          |

---

## Distribuição de DF-e

O módulo `distribuicao` permite consultar documentos fiscais eletrônicos de interesse do destinatário (NF-e, CT-e, etc.) via webservice da SEFAZ.

### Exemplo: Consultar novos documentos (último NSU)

```rust
use dfe::distribuicao::Distribuicao;

let resposta = Distribuicao::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("1234")
    .cnpj("11111111111111")
    .uf(35)         // código IBGE do estado do autor (SP = 35)
    .ambiente(2)    // 1 = Produção, 2 = Homologação
    .send()
    .await;

match resposta {
    Ok(r) => {
        println!("cStat: {}", r.c_stat);
        println!("xMotivo: {}", r.x_motivo);
        println!("ultNSU: {}", r.ult_nsu);
        println!("maxNSU: {}", r.max_nsu);
        if let Some(lote) = r.lote_dist_dfe_int {
            for doc in lote {
                println!("NSU: {} | Schema: {}", doc.nsu, doc.schema);
            }
        }
    }
    Err(e) => println!("Erro: {}", e),
}
```

### Exemplo: Consultar por NSU específico

```rust
use dfe::distribuicao::DistribuicaoNSU;

let resposta = DistribuicaoNSU::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("1234")
    .cnpj("11111111111111")
    .uf(35)
    .ambiente(2)
    .nsu("000000000000100")
    .send()
    .await;
```

### Exemplo: Consultar por chave de acesso

```rust
use dfe::distribuicao::DistribuicaoChaveAcesso;

let resposta = DistribuicaoChaveAcesso::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("1234")
    .cnpj("11111111111111")
    .uf(35)
    .ambiente(2)
    .chave_acesso("35241211111111111111550010000000381505051324")
    .send()
    .await;
```

### Métodos comuns (Distribuição)

| Método                      | Descrição                                                           |
| --------------------------- | ------------------------------------------------------------------- |
| `new()`                     | Cria uma nova instância do builder                                  |
| `cert_path(path: &str)`     | Caminho do certificado `.pfx`                                       |
| `cert_pass(pass: &str)`     | Senha do certificado                                                |
| `cnpj(cnpj: &str)`          | CNPJ do destinatário                                                |
| `uf(uf: u8)`                | Código IBGE da UF do autor                                          |
| `ambiente(amb: u8)`         | `1` = Produção, `2` = Homologação                                   |
| `nsu(nsu: &str)`            | NSU específico (apenas `DistribuicaoNSU`)                           |
| `chave_acesso(chave: &str)` | Chave de acesso (apenas `DistribuicaoChaveAcesso`)                  |
| `check_flag()`              | Ativa verificação de flag de pendência                              |
| `send()`                    | Executa a consulta e retorna `Result<DistribuicaoResposta, String>` |

---

## Manifestação do Destinatário

O módulo `distribuicao` também expõe os builders de manifestação, que permitem registrar eventos de ciência, confirmação, desconhecimento ou operação não realizada para uma NF-e.

### Exemplo: Ciência da Operação

```rust
use dfe::distribuicao::CienciaOperacao;

let resposta = CienciaOperacao::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("1234")
    .cnpj("11111111111111")
    .ambiente(2)
    .chave_acesso("35241211111111111111550010000000381505051324")
    .send()
    .await;

match resposta {
    Ok(r) => {
        println!("cStat: {}", r.response.c_stat);
        println!("xMotivo: {}", r.response.x_motivo);
    }
    Err(e) => println!("Erro: {}", e),
}
```

### Exemplo: Confirmação da Operação

```rust
use dfe::distribuicao::ConfirmacaoOperacao;

let resposta = ConfirmacaoOperacao::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("1234")
    .cnpj("11111111111111")
    .ambiente(2)
    .chave_acesso("35241211111111111111550010000000381505051324")
    .send()
    .await;
```

### Exemplo: Desconhecimento da Operação

```rust
use dfe::distribuicao::DesconhecimentoOperacao;

let resposta = DesconhecimentoOperacao::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("1234")
    .cnpj("11111111111111")
    .ambiente(2)
    .chave_acesso("35241211111111111111550010000000381505051324")
    .send()
    .await;
```

### Exemplo: Operação Não Realizada

```rust
use dfe::distribuicao::OperacaoNaoRealizada;

let resposta = OperacaoNaoRealizada::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("1234")
    .cnpj("11111111111111")
    .ambiente(2)
    .chave_acesso("35241211111111111111550010000000381505051324")
    .justificativa("Mercadoria não recebida pelo destinatário")
    .send()
    .await;
```

### Métodos comuns (Manifestação)

| Método                      | Descrição                                                         |
| --------------------------- | ----------------------------------------------------------------- |
| `new()`                     | Cria uma nova instância do builder                                |
| `cert_path(path: &str)`     | Caminho do certificado `.pfx`                                     |
| `cert_pass(pass: &str)`     | Senha do certificado                                              |
| `cnpj(cnpj: &str)`          | CNPJ do destinatário                                              |
| `ambiente(amb: u8)`         | `1` = Produção, `2` = Homologação                                 |
| `chave_acesso(chave: &str)` | Chave de acesso da NF-e                                           |
| `justificativa(just: &str)` | Justificativa (apenas `OperacaoNaoRealizada`, mín. 15 chars)      |
| `send()`                    | Executa o evento e retorna `Result<ManifestacaoResposta, String>` |

---

## Validação de XML (XSD)

O módulo `nfe::common::validation` fornece funções para validar o XML de uma NF-e contra os schemas XSD oficiais da SEFAZ antes do envio.

### Como funciona

1. **Verificação dos schemas**: na primeira chamada, a rotina verifica se a pasta de schemas `./dfe/shema/PL_010b_NT2025_002_v1.21/` existe e se todos os 5 arquivos XSD necessários estão presentes.
2. **Download automático**: qualquer arquivo ausente é baixado automaticamente do repositório oficial em `https://raw.githubusercontent.com/GustavoOta/dfe/main/dfe/shema/PL_010b_NT2025_002_v1.21/`. Se o download falhar, a função retorna `Err` com o nome do arquivo e o motivo.
3. **Parse do XML**: o XML recebido é parseado via `libxml`.
4. **Validação XSD**: o documento parseado é validado contra `nfe_v4.00.xsd`. Em caso de erro, a mensagem do primeiro erro de validação é retornada.

### Arquivos XSD utilizados

| Arquivo                         | Descrição                     |
| ------------------------------- | ----------------------------- |
| `nfe_v4.00.xsd`                 | Schema principal da NF-e 4.00 |
| `leiauteNFe_v4.00.xsd`          | Leiaute da NF-e               |
| `tiposBasico_v4.00.xsd`         | Tipos básicos                 |
| `DFeTiposBasicos_v1.00.xsd`     | Tipos básicos de DF-e         |
| `xmldsig-core-schema_v1.01.xsd` | Schema de assinatura digital  |

### Exemplo de uso

```rust
use dfe::nfe::common::validation::{validate_xml, is_xml_valid};

// Versão async (recomendada — executa em thread de blocking do Tokio)
let xml = std::fs::read_to_string("nfe_request.xml").unwrap();
match validate_xml(xml).await {
    Ok(xml_valido) => println!("XML válido"),
    Err(e) => println!("Erro de validação: {}", e),
}

// Versão síncrona (use dentro de spawn_blocking ou em contexto não-async)
match is_xml_valid(&xml) {
    Ok(_) => println!("XML válido"),
    Err(e) => println!("Erro: {}", e),
}
```

### Assinaturas

| Função                                                | Descrição                                                                                               |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `validate_xml(xml: String) -> Result<String, String>` | Async — delega para `is_xml_valid` via `spawn_blocking`. Retorna o XML original em caso de sucesso.     |
| `is_xml_valid(xml: &str) -> Result<String, String>`   | Síncrona — verifica schemas, faz download se necessário, valida e retorna o XML ou um `Err` descritivo. |

### Erros possíveis

| Situação                              | Mensagem retornada                                                       |
| ------------------------------------- | ------------------------------------------------------------------------ |
| Falha ao criar diretório de schemas   | `"Erro ao criar diretório './dfe/shema/...': <motivo>"`                  |
| Arquivo XSD ausente e download falhou | `"Arquivo '<nome>' não existia, tentei baixar e não consegui: <motivo>"` |
| XML mal formado                       | `"Erro ao parsear o XML"`                                                |
| Contexto XSD inválido                 | `"Erro ao criar contexto de validação XSD: ..."`                         |
| XML inválido segundo o schema         | Mensagem do primeiro erro retornado pelo validador XSD                   |