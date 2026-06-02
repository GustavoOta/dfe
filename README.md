# dfe

**Documentos Fiscais Eletrônicos Brasileiros** — crate Rust para integração com os webservices da SEFAZ.

[![Sponsor GustavoOta](https://img.shields.io/badge/sponsor-GustavoOta-%23EA4AAA?style=flat&logo=github)](https://github.com/sponsors/GustavoOta)

---

## Funcionalidades

- Emissão de **NF-e** (modelo 55) e **NFC-e** (modelo 65)
- **Cancelamento** de NF-e / NFC-e
- **Manifestação do destinatário** (ciência, confirmação, desconhecimento, operação não realizada)
- **Distribuição de DF-e** (consulta ao Ambiente Nacional)
- **Geração de DANFE** em PDF (80mm modelo 55; outros formatos em desenvolvimento)
- **Validação XSD** do XML contra schemas oficiais da SEFAZ (embutidos no binário)
- **Status do webservice** SEFAZ

---

## Instalação

```toml
[dependencies]
dfe = "0.5.7"
```

> **Requisito:** OpenSSL instalado no ambiente. No Windows, o crate usa `openssl` com feature `vendored` — não requer instalação manual.

> Os schemas XSD oficiais da SEFAZ (versão `PL_010b_NT2025_002_v1.21`) estão **embutidos no binário**. Não é necessário copiar nenhuma pasta de schemas para o diretório do executável.

---

## Tratamento de erros

Todas as funções públicas retornam `Result<T, dfe::DfeError>`.

```rust
use dfe::DfeError;

match resultado {
    Ok(r) => { /* ... */ }
    Err(DfeError::Certificado(msg)) => eprintln!("Problema no .pfx: {}", msg),
    Err(DfeError::Validacao(msg))   => eprintln!("Dado inválido: {}", msg),
    Err(DfeError::Webservice(msg))  => eprintln!("Falha SEFAZ: {}", msg),
    Err(e)                          => eprintln!("Erro: {}", e),
}
```

| Variante          | Quando ocorre                                              |
|-------------------|------------------------------------------------------------|
| `Certificado`     | Falha ao abrir o `.pfx` ou senha incorreta                 |
| `Xml`             | Erro de parsing ou serialização XML                        |
| `Assinatura`      | Falha na assinatura digital RSA-SHA1                       |
| `Webservice`      | Erro HTTP ou resposta inesperada da SEFAZ                  |
| `Validacao`       | Campo obrigatório ausente ou valor fora das regras do XSD  |
| `Configuracao`    | Falha ao ler arquivo de configuração ou credenciais        |
| `Io`              | Erro de leitura/escrita em disco                           |

---

## Emissão de NF-e / NFC-e

Use o `NFeBuilder`. Cada método retorna `Self`, permitindo encadeamento fluente. `emitir()` valida os campos obrigatórios antes de enviar.

```rust
use dfe::nfe::autorizacao::NFeBuilder;
use dfe::nfe::types::autorizacao4::*;
use dfe::nfe::xml_rules::ide::models::Ide;
use dfe::nfe::xml_rules::dest::models::Dest;

let resposta = NFeBuilder::new()
    .cert("D:/Projetos/cert.pfx", "senha_do_pfx")
    .ide(Ide {
        c_uf: 35,           // SP
        mod_: 55,           // 55 = NF-e | 65 = NFC-e
        serie: 1,
        n_nf: 100,
        tp_amb: 2,          // 1 = Produção | 2 = Homologação
        tp_emis: 1,
        tp_nf: 1,           // 1 = Saída
        id_dest: 1,
        tp_imp: 1,
        ind_final: 1,
        ind_pres: 1,
        c_mun_fg: "3550308".to_string(),
        ..Default::default()
    })
    .emitente(Emit {
        cnpj: Some("11111111111111".to_string()),
        x_nome: "EMPRESA LTDA".to_string(),
        x_lgr: "RUA DAS FLORES".to_string(),
        nro: "100".to_string(),
        x_bairro: "CENTRO".to_string(),
        c_mun: "3550308".to_string(),
        x_mun: "SAO PAULO".to_string(),
        uf: "SP".to_string(),
        cep: "01001000".to_string(),
        ie: Some("111111111111".to_string()),
        crt: 3,             // 1 = SN | 3 = Regime Normal
        ..Default::default()
    })
    .destinatario(Dest {   // opcional para NFC-e
        cpf: Some("07068093868".to_string()),
        x_nome: Some("FULANO DA SILVA".to_string()),
        ind_ie_dest: Some(9),
        ..Default::default()
    })
    .item(Det {
        c_prod: "001".to_string(),
        x_prod: "PRODUTO EXEMPLO".to_string(),
        ncm: "22030000".to_string(),
        cfop: 5102,
        u_com: "UN".to_string(),
        q_com: 1.0,
        v_un_com: 100.0,
        v_prod: 100.0,
        u_trib: "UN".to_string(),
        q_trib: 1.0,
        v_un_trib: 100.0,
        ind_tot: 1,
        icms: "ICMSSN102".to_string(),  // Simples Nacional sem crédito
        csosn: Some("102".to_string()),
        orig: Some(0),
        pis: "PISAliq".to_string(),
        pis_cst: Some("01".to_string()),
        pis_v_bc: Some(100.0),
        pis_p_pis: Some(1.65),
        pis_v_pis: Some(1.65),
        cofins: "COFINSAliq".to_string(),
        cofins_cst: Some("01".to_string()),
        cofins_v_bc: Some(100.0),
        cofins_p_cofins: Some(7.6),
        cofins_v_cofins: Some(7.6),
        v_tot_trib: 9.25,
        ..Default::default()
    })
    .total(Total {
        v_prod: 100.0,
        v_nf: 100.0,
        v_pis: 1.65,
        v_cofins: 7.6,
        v_tot_trib: 9.25,
        ..Default::default()
    })
    .transporte(Transp {
        mod_frete: 9,  // 9 = Sem frete
        ..Default::default()
    })
    .pagamento(Pag {
        ind_pag: 0,
        t_pag: "01".to_string(),  // 01 = Dinheiro
        v_pag: 100.0,
        ..Default::default()
    })
    // opcionais:
    // .informacoes_adicionais(InfAdic { inf_cpl: Some("Obs...".to_string()), ..Default::default() })
    // .id_csc("ID_CSC")      // NFC-e: token do CSC cadastrado na SEFAZ
    // .csc("VALOR_CSC")      // NFC-e: valor do CSC
    // .desconto_rateio(Decimal::new(500, 2))  // R$ 5,00 de desconto rateado nos itens
    .emitir()
    .await?;

println!("Protocolo: {}", resposta.protocolo.inf_prot.n_prot.unwrap_or_default());
println!("cStat: {}",    resposta.protocolo.inf_prot.c_stat);
println!("xMotivo: {}",  resposta.protocolo.inf_prot.x_motivo);
// resposta.xml contém o XML autorizado (nfeProc) pronto para salvar
```

### Métodos do NFeBuilder

| Método | Obrigatório | Descrição |
|---|:---:|---|
| `cert(path, pass)` | ✅ | Caminho e senha do certificado `.pfx` |
| `ide(Ide)` | ✅ | Identificação do documento |
| `emitente(Emit)` | ✅ | Dados do emitente |
| `item(Det)` | ✅ | Adiciona um item (pode chamar várias vezes) |
| `total(Total)` | ✅ | Totalizações da NF-e |
| `transporte(Transp)` | ✅ | Dados do transporte |
| `pagamento(Pag)` | ✅ | Forma de pagamento |
| `destinatario(Dest)` | — | Destinatário (obrigatório para NF-e mod 55) |
| `informacoes_adicionais(InfAdic)` | — | Informações complementares |
| `id_csc(str)` | — | ID do CSC (NFC-e) |
| `csc(str)` | — | Valor do CSC (NFC-e) |
| `desconto_rateio(Decimal)` | — | Valor de desconto a ratear nos itens |
| `acrescimo_rateio(Decimal)` | — | Valor de acréscimo a ratear nos itens |
| `active_ibs_cbs(str)` | — | Desativa IBS/CBS quando `Some` |
| `emitir()` | — | Valida, assina e envia para a SEFAZ |

### Resposta da emissão

```rust
pub struct Response {
    pub protocolo: TagInfProt,  // dados do protocolo SEFAZ
    pub xml: String,            // XML autorizado completo (nfeProc)
}
```

---

## Cancelamento de NF-e / NFC-e

```rust
use dfe::nfe::cancelar::nfe_cancelar;
use dfe::nfe::types::cancelar::NFeCancelar;

let resposta = nfe_cancelar(NFeCancelar {
    cert_path: "D:/Projetos/cert.pfx".to_string(),
    cert_pass: "senha".to_string(),
    tp_amb: 2,                        // 1 = Produção | 2 = Homologação
    mod_: Some(55),                   // None = 55
    chave: "35241211111111111111550010000000361491395167".to_string(),
    protocolo: "135190000000000".to_string(),
    justificativa: "Nota emitida com erro de valor".to_string(),
})
.await?;

println!("cStat: {}",   resposta.response.c_stat);
println!("xMotivo: {}", resposta.response.x_motivo);
// resposta.send_xml    — XML enviado para a SEFAZ
// resposta.receive_xml — XML de resposta recebido
```

---

## Manifestação do Destinatário

Os eventos são enviados ao **Ambiente Nacional (AN)** da SEFAZ.

```rust
use dfe::nfe::manifestacao::{
    nfe_ciencia_operacao,
    nfe_confirmacao_operacao,
    nfe_desconhecimento_operacao,
    nfe_operacao_nao_realizada,
};
use dfe::nfe::types::manifestacao::{Manifestacao, OperacaoNaoRealizada};

let params = Manifestacao {
    cert_path: "D:/Projetos/cert.pfx".to_string(),
    cert_pass: "senha".to_string(),
    cnpj: "11111111111111".to_string(),
    tp_amb: 2,
    mod_: None,
    chave: "35241211111111111111550010000000361491395167".to_string(),
};

// 210210 — Ciência da Operação
let r = nfe_ciencia_operacao(params.clone()).await?;

// 210200 — Confirmação da Operação
let r = nfe_confirmacao_operacao(params.clone()).await?;

// 210220 — Desconhecimento da Operação
let r = nfe_desconhecimento_operacao(params.clone()).await?;

// 210240 — Operação Não Realizada (requer justificativa mín. 15 chars)
let r = nfe_operacao_nao_realizada(OperacaoNaoRealizada {
    cert_path: "D:/Projetos/cert.pfx".to_string(),
    cert_pass: "senha".to_string(),
    cnpj: "11111111111111".to_string(),
    tp_amb: 2,
    mod_: None,
    chave: "35241211111111111111550010000000361491395167".to_string(),
    justificativa: "Mercadoria não recebida pelo destinatário".to_string(),
})
.await?;

println!("cStat: {}",   r.response.c_stat);
println!("xMotivo: {}", r.response.x_motivo);
```

---

## Distribuição de DF-e

Consulta documentos fiscais de interesse do destinatário no Ambiente Nacional.

```rust
use dfe::distribuicao::{
    Distribuicao,               // últimos documentos (último NSU)
    DistribuicaoNSU,            // a partir de um NSU específico
    DistribuicaoChaveAcesso,    // por chave de acesso
};

// Consultar últimos documentos
let r = Distribuicao::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("senha")
    .cnpj("11111111111111")
    .uf(35)       // código IBGE da UF (SP = 35)
    .ambiente(2)  // 1 = Produção | 2 = Homologação
    .send()
    .await?;

println!("cStat: {} — {}", r.c_stat, r.x_motivo);
println!("ultNSU: {} | maxNSU: {}", r.ult_nsu, r.max_nsu);

if let Some(lote) = r.lote_dist_dfe_int {
    for doc in lote {
        println!("NSU: {} | Schema: {}", doc.nsu, doc.schema);
        if let Some(nfe) = doc.content {
            println!("  chNFe: {} | vNF: {}", nfe.ch_nfe, nfe.v_nf);
        }
    }
}

// Por NSU específico
let r = DistribuicaoNSU::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("senha")
    .cnpj("11111111111111")
    .uf(35)
    .ambiente(2)
    .nsu("000000000000100")
    .send()
    .await?;

// Por chave de acesso
let r = DistribuicaoChaveAcesso::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("senha")
    .cnpj("11111111111111")
    .uf(35)
    .ambiente(2)
    .chave_acesso("35241211111111111111550010000000361491395167")
    .send()
    .await?;
```

---

## Status do Webservice

```rust
use dfe::status::NFeService;

let r = NFeService::new()
    .cert_path("D:/Projetos/cert.pfx")
    .cert_pass("senha")
    .uf("SP")
    .environment(2)  // 1 = Produção | 2 = Homologação
    .send()
    .await?;

println!("cStat: {}",   r.c_stat);
println!("xMotivo: {}", r.x_motivo);
println!("URL: {}",     r.url);
```

---

## DANFE (PDF)

Gera o DANFE a partir do XML autorizado (`nfeProc`). O método `.xml()` aceita caminho de arquivo `.xml` ou string XML diretamente.

```rust
use dfe::pdf::DanfeBuilder;

// Salvar como arquivo PDF
let caminho = DanfeBuilder::new()
    .xml("./nota_autorizada.xml")   // ou .xml("<nfeProc>...</nfeProc>")
    .paper_size("80mm")
    .as_file("./danfe.pdf")
    .build()
    .await?;

println!("PDF salvo em: {}", caminho);

// Obter como base64
let base64 = DanfeBuilder::new()
    .xml("<nfeProc>...</nfeProc>")
    .paper_size("80mm")
    .as_base64()
    .build()
    .await?;
```

| Tamanho  | Modelo 55 (NF-e) | Modelo 65 (NFC-e) |
|----------|:----------------:|:-----------------:|
| `"80mm"` | ✅               | Em breve          |
| `"a4"`   | Em breve         | Em breve          |
| `"54mm"` | Em breve         | Em breve          |

---

## Validação XSD

Valida o XML contra os schemas XSD oficiais da SEFAZ (versão `PL_010b_NT2025_002_v1.21`, embutidos no binário).

```rust
use dfe::nfe::common::validation::{validate_xml, is_xml_valid};

// Versão async — recomendada em contexto async
let xml = std::fs::read_to_string("nfe_request.xml").unwrap();
match validate_xml(xml).await {
    Ok(_)  => println!("XML válido"),
    Err(e) => println!("Inválido: {}", e),
}

// Versão síncrona — use dentro de spawn_blocking
match is_xml_valid(&xml) {
    Ok(_)  => println!("XML válido"),
    Err(e) => println!("Inválido: {}", e),
}
```

Schemas embutidos:

| Arquivo                           | Descrição                    |
|-----------------------------------|------------------------------|
| `nfe_v4.00.xsd`                   | Schema principal NF-e 4.00   |
| `leiauteNFe_v4.00.xsd`            | Leiaute da NF-e              |
| `tiposBasico_v4.00.xsd`           | Tipos básicos                |
| `DFeTiposBasicos_v1.00.xsd`       | Tipos básicos DF-e           |
| `xmldsig-core-schema_v1.01.xsd`   | Schema de assinatura digital |

---

## Tipos de ICMS suportados

Configure o campo `icms` do `Det` com uma das strings abaixo e preencha os campos correspondentes.

| `icms` | Regime | Campos obrigatórios adicionais |
|---|---|---|
| `"ICMS00"` | Regime Normal — Tributada integralmente | `orig`, `cst="00"`, `mod_bc`, `v_bc`, `p_icms`, `v_icms` |
| `"ICMS40"` | Isenta / Não tributada / Suspensão | `orig`, `cst` (`40`/`41`/`50`) |
| `"ICMS60"` | ST cobrada anteriormente | `orig`, `cst="60"` |
| `"ICMS90"` | Outros | `orig`, `cst="90"` |
| `"ICMSSN101"` | Simples Nacional — com crédito | `orig`, `csosn="101"`, `p_cred_sn`, `v_cred_icmssn` |
| `"ICMSSN102"` | Simples Nacional — sem crédito | `orig`, `csosn="102"` |
| `"ICMSSN500"` | ST cobrada anteriormente (SN) | `orig`, `csosn="500"` |
| `"ICMSSN900"` | Outros (SN) | `orig`, `csosn="900"` |

---

## Notas importantes

- Este software está em **desenvolvimento ativo**. Não use em produção sem validar o fluxo completo em homologação.
- Sempre verifique o arquivo `flag_autorizacao.env` antes de operar — ele controla se o ambiente é homologação ou produção.
- O certificado `.pfx` nunca é cacheado em memória; é lido a cada operação por segurança.
- Os webservices cobertos atualmente são os da **SEFAZ/SP**. Para outras UFs, abra uma issue ou contribua com um PR adicionando as URLs em `src/data/webservices.json`.
