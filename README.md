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
- **Certificado A1 (.pfx).** Assinatura XML-DSig (RSA-SHA1) e TLS mútuo resolvidos internamente,
  inclusive para os PFX ICP-Brasil antigos (RC2/3DES).
- **Windows e Linux.** No Windows usa a CryptoAPI do próprio sistema (sem OpenSSL, `libxml2`
  estático); no Linux, o OpenSSL do sistema com o provider `legacy` e a cadeia ICP-Brasil embutida
  para confiar nos servidores da SEFAZ — mesmo resultado, byte a byte. Detalhes em
  [Certificado digital e plataformas](docs/certificado-e-plataformas.md).
- **Validação XSD.** Schemas oficiais da SEFAZ embutidos no binário, sem arquivos externos.
- **Erros tipados.** Todas as operações retornam `Result<T, DfeError>`; o caminho de request não dá
  `panic`.
- **Build modular.** As camadas de DANFE, ESC/POS e distribuição são *features* opcionais (ligadas por
  padrão) — veja [Features de compilação](#features-de-compilação).

---

## Plataformas

| Sistema | Certificado e assinatura | TLS com a SEFAZ | Situação |
|---|---|---|---|
| **Windows** | CryptoAPI do sistema (sem OpenSSL) | SChannel | ✅ em produção |
| **Linux** | OpenSSL do sistema + provider `legacy` (PFX ICP-Brasil em RC2/3DES) | OpenSSL + cadeia ICP-Brasil embutida | ✅ desde a 0.8 — NF-e e NFC-e autorizadas, corrigidas e canceladas em homologação com A1 real |
| **macOS** | — | — | ⚠️ não testado |

A API é a mesma nos dois sistemas suportados, e o resultado também: a assinatura sai idêntica byte a
byte. Como cada parte funciona, por que o provider `legacy` é necessário, a cadeia ICP-Brasil, o
build no Linux e os testes: **[Certificado digital e plataformas](docs/certificado-e-plataformas.md)**.

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
| **DANFE em PDF** | A4 e 80mm, NF-e e NFC-e, com suporte a logotipo; A4 no modelo da Reforma Tributária (NT 2026.010: CRT, totais e itens de IBS/CBS/IS) |
| **ESC/POS** | `EscPosBuilder` (layout livre) e `EscPosNFCeBuilder` (NFC-e pronto) |
| **Validação XSD** | Schemas oficiais da SEFAZ embutidos no binário |
| **Validação CNPJ/CPF** | Dígito verificador (inclui CNPJ alfanumérico) |
| **Extração de XML** | Leitura de campos de XML autorizado |

---

## Início rápido

```toml
[dependencies]
dfe = "0.8"
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
>
> **Linux:** instale `pkg-config`, `libssl-dev` e `libxml2-dev` (Debian/Ubuntu). Como o certificado
> funciona em cada sistema — CAPI no Windows, OpenSSL no Linux, provider `legacy` e cadeia
> ICP-Brasil — está em [Certificado digital e plataformas](docs/certificado-e-plataformas.md).
> macOS não foi testado.

---

## Ver o DANFE A4 sem certificado

A crate traz amostras do DANFE A4 da NF-e (modelo 55) no padrão da Reforma Tributária
(NT 2026.010, vigente a partir de 01/12/2026). Elas rodam no `cargo test` normal, **sem
certificado e sem acesso à SEFAZ**, e gravam os PDFs para você abrir e conferir:

```bash
git clone https://github.com/GustavoOta/dfe
cd dfe
cargo test --test danfe_a4_amostras -- --nocapture
```

Cada teste imprime o caminho do arquivo gerado. Os PDFs ficam em `target/tmp/danfe-a4/`; para
gravar em outra pasta, defina `DFE_DANFE_SAIDA`:

```bash
# Linux/macOS
DFE_DANFE_SAIDA=./amostras cargo test --test danfe_a4_amostras -- --nocapture
# Windows (PowerShell)
$env:DFE_DANFE_SAIDA = ".\amostras"; cargo test --test danfe_a4_amostras -- --nocapture
```

| Arquivo | O que mostra |
|---|---|
| `01-simples-sem-ibs-cbs.pdf` | Simples Nacional (CRT 1) sem IBS/CBS: bloco "TOTAL DO IBS / CBS / IS" em branco e itens sem sublinha |
| `02-regime-normal-ibs-cbs.pdf` | Regime normal (CRT 3) com IBS/CBS em todos os itens |
| `03-reducao-e-imposto-seletivo.pdf` | Redução de alíquota (`gRed`: sai a alíquota efetiva) e Imposto Seletivo |
| `04-monofasico.pdf` | Totais com `gMono`: aparece a linha da monofasia |
| `05-muitos-itens.pdf` | 60 itens, metade com IBS/CBS |
| `06-mei.pdf` | MEI (CRT 4) |

As notas são fictícias (dados e valores ilustrativos). Para gerar o DANFE de uma nota sua, use o
`DanfeBuilder` com o XML autorizado (`nfeProc`) — ver [DANFE](docs/danfe.md):

```rust
let base64_pdf = dfe::DanfeBuilder::new()
    .xml(xml_autorizado)   // conteúdo do nfeProc ou caminho de um arquivo .xml
    .paper_size("a4")
    .as_base64()           // ou .as_file("danfe.pdf")
    .build()
    .await?;
```

> **Limite conhecido:** NF-e com muitos itens não é quebrada em várias folhas A4 — a página
> cresce em altura (a amostra de 60 itens sai com ~485 mm).

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
dfe = { version = "0.8", default-features = false, features = ["danfe"] }
```

O núcleo — emissão, cancelamento, substituição, carta de correção, status, extração de XML, assinatura
e validação — está sempre disponível, independente das features.

---

## Documentação

| Seção | Descrição |
|---|---|
| [Instalação](docs/instalacao.md) | Plataformas suportadas, libxml2/vcpkg (linking estático no Windows) |
| [Certificado digital e plataformas](docs/certificado-e-plataformas.md) | CAPI no Windows × OpenSSL no Linux, provider `legacy` (PFX RC2/3DES) e por quê, cadeia ICP-Brasil para o TLS com a SEFAZ, macOS, testes |
| [Emissão NF-e / NFC-e](docs/emissao-nfe-nfce.md) | `NFeBuilder`, métodos, totais automáticos |
| [Cancelamento](docs/cancelamento.md) | `CancelarBuilder` |
| [Manifestação do Destinatário](docs/manifestacao.md) | Ciência, confirmação, desconhecimento, op. não realizada |
| [Distribuição de DF-e](docs/distribuicao.md) | Consulta por NSU e chave de acesso |
| [DANFE](docs/danfe.md) | Geração de PDF A4 e 80mm |
| [ESC/POS](docs/escpos.md) | `EscPosBuilder` e `EscPosNFCeBuilder` |
| [Status do Webservice](docs/status-webservice.md) | Disponibilidade por UF e modelo (NF-e 55 / NFC-e 65), códigos e limite de consulta |
| [Tratamento de Erros](docs/erros.md) | `DfeError` — variantes e quando ocorrem |
| [ICMS, PIS, COFINS](docs/icms-pis-cofins.md) | Todos os grupos de ICMS do leiaute (modalidades, partilha, repasse, monofásico, gCred/comb), PIS/COFINS de qualquer CST, IPI e validação CNPJ/CPF |
| [Testes](docs/testes.md) | Suites disponíveis, requisitos e testes de grupo contra o XSD |
| [Notas e Roadmap](docs/notas-roadmap.md) | Boas práticas e funcionalidades planejadas |

Referência completa dos *builders* (incluindo `SubstituicaoBuilder`, `CartaCorrecaoBuilder` e as
features) em [API.md](API.md).

---

## Licença

MIT — veja [LICENSE](LICENSE).
