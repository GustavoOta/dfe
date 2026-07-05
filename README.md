# dfe

**Documentos Fiscais Eletrônicos Brasileiros** — crate Rust para integração com os webservices da SEFAZ.

Cobre o ciclo fiscal de NF-e e NFC-e — emissão, eventos, assinatura, validação, DANFE e impressão — em
uma única biblioteca Rust `async`, com roteamento para as 27 UFs.

[![Crates.io](https://img.shields.io/crates/v/dfe)](https://crates.io/crates/dfe)
[![Docs.rs](https://docs.rs/dfe/badge.svg)](https://docs.rs/dfe)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Sponsor GustavoOta](https://img.shields.io/badge/sponsor-GustavoOta-%23EA4AAA?style=flat&logo=github)](https://github.com/sponsors/GustavoOta)

---

## Características

- **API `async` em Rust.** Cada operação é um *builder* fluente que retorna a resposta da SEFAZ tipada.
- **Roteamento multi-UF.** Seleciona o webservice a partir da UF do documento (autorizadores próprios e
  compartilhados), em produção e homologação, para as 27 UFs.
- **Certificado A1 (.pfx).** Assinatura XML-DSig (RSA-SHA1) e TLS mútuo resolvidos internamente. Não
  depende de OpenSSL; `libxml2` é linkado estaticamente.
- **Validação XSD.** Schemas oficiais da SEFAZ embutidos no binário, sem arquivos externos.
- **Erros tipados.** Todas as operações retornam `Result<T, DfeError>`; o caminho de request não dá
  `panic`.
- **Build modular.** As camadas de DANFE, ESC/POS e distribuição são *features* opcionais (ligadas por
  padrão) — veja [Features de compilação](#features-de-compilação).

---

## Funcionalidades

### Documentos e eventos fiscais
| Funcionalidade | Descrição |
|---|---|
| **Emissão NF-e / NFC-e** | Autorização via SOAP (modelos 55 e 65), com totais calculados automaticamente |
| **Cancelamento** | Evento 110111 (NF-e e NFC-e) |
| **Cancelamento por substituição** | Evento 110112 para NFC-e (NT 2018.004) |
| **Carta de Correção (CC-e)** | Evento 110110, com `nSeqEvento` acumulativo |
| **Manifestação do destinatário** | Ciência, confirmação, desconhecimento e operação não realizada |
| **Distribuição de DF-e** | Consulta ao Ambiente Nacional por NSU ou chave de acesso |
| **Status do webservice** | Disponibilidade por UF e ambiente |

### Fiscal, impressão e infraestrutura
| Funcionalidade | Descrição |
|---|---|
| **Roteamento multi-UF** | 27 UFs (autorizadores próprios e compartilhados), produção e homologação |
| **IBS/CBS (Reforma Tributária)** | Grupo IBS/CBS por item e nos totais (omitido quando não aplicável) |
| **CNPJ alfanumérico** | Suporte ao novo formato de CNPJ (letras e dígitos) |
| **DANFE em PDF** | A4 e 80mm, NF-e e NFC-e, com suporte a logotipo |
| **ESC/POS** | `EscPosBuilder` (layout livre) e `EscPosNFCeBuilder` (NFC-e pronto) |
| **Validação XSD** | Schemas oficiais da SEFAZ embutidos no binário |
| **Validação CNPJ/CPF** | Dígito verificador (inclui CNPJ alfanumérico) |
| **Extração de XML** | Leitura de campos de XML autorizado |

---

## Início rápido

```toml
[dependencies]
dfe = "0.6"
```

```rust
use dfe::{NFeBuilder, DfeError};
use dfe::tipos::{Det, Emit, Icms, Ide, Pag, Pis, Cofins, Total, Transp};

async fn emitir() -> Result<(), DfeError> {
    let resp = NFeBuilder::new()
        .cert("./cert.pfx", "senha")
        .ide(Ide { c_uf: 35, mod_: 55, serie: 1, n_nf: 1, tp_amb: 2, ..Default::default() })
        .emitente(Emit { cnpj: Some("11111111111111".into()), ..Default::default() })
        .itens(vec![Det {
            c_prod: "001".into(), x_prod: "PRODUTO".into(), ncm: "22030000".into(),
            cfop: 5102, q_com: 1.0, v_un_com: 10.0, v_prod: 10.0,
            icms: Icms::sn102(0, "400"),
            pis: Pis::Nt { cst: "07".into() },
            cofins: Cofins::Nt { cst: "07".into() },
            ..Default::default()
        }])
        .total(Total::default())
        .transporte(Transp::default())
        .pagamento(Pag::default())
        .emitir()
        .await?;

    println!("cStat: {}", resp.protocolo.inf_prot.c_stat);
    Ok(())
}
```

> **Windows:** antes de compilar, leia [docs/instalacao.md](docs/instalacao.md) — é necessário
> configurar o vcpkg para linking estático do `libxml2`. OpenSSL **não** é uma dependência.

---

## Features de compilação

Todas as features são ligadas por padrão. Para reduzir o build e as dependências, desligue as que não
usar:

| Feature | Liga | Padrão |
|---|---|:--:|
| `danfe` | Geração de DANFE em PDF | ✅ |
| `escpos` | Impressão ESC/POS | ✅ |
| `distribuicao` | Distribuição de DF-e + manifestação | ✅ |

```toml
# Só emissão/cancelamento + DANFE, sem ESC/POS nem distribuição:
dfe = { version = "0.6", default-features = false, features = ["danfe"] }
```

O núcleo — emissão, cancelamento, substituição, carta de correção, status, extração de XML, assinatura
e validação — está sempre disponível, independente das features.

---

## Documentação

| Seção | Descrição |
|---|---|
| [Instalação](docs/instalacao.md) | Plataformas suportadas, libxml2/vcpkg (linking estático no Windows) |
| [Emissão NF-e / NFC-e](docs/emissao-nfe-nfce.md) | `NFeBuilder`, métodos, totais automáticos |
| [Cancelamento](docs/cancelamento.md) | `CancelarBuilder` |
| [Manifestação do Destinatário](docs/manifestacao.md) | Ciência, confirmação, desconhecimento, op. não realizada |
| [Distribuição de DF-e](docs/distribuicao.md) | Consulta por NSU e chave de acesso |
| [DANFE](docs/danfe.md) | Geração de PDF A4 e 80mm |
| [ESC/POS](docs/escpos.md) | `EscPosBuilder` e `EscPosNFCeBuilder` |
| [Status do Webservice](docs/status-webservice.md) | Consulta de disponibilidade por UF |
| [Tratamento de Erros](docs/erros.md) | `DfeError` — variantes e quando ocorrem |
| [ICMS, PIS, COFINS](docs/icms-pis-cofins.md) | Tipos de ICMS, IPI, PIS/COFINS e validação CNPJ/CPF |
| [Testes](docs/testes.md) | Suites disponíveis e requisitos |
| [Notas e Roadmap](docs/notas-roadmap.md) | Boas práticas e funcionalidades planejadas |

Referência completa dos *builders* (incluindo `SubstituicaoBuilder`, `CartaCorrecaoBuilder` e as
features) em [API.md](API.md).

---

## Licença

MIT — veja [LICENSE](LICENSE).
