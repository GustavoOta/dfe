# dfe

**Documentos Fiscais Eletrônicos Brasileiros** — crate Rust para integração com os webservices da SEFAZ.

[![Crates.io](https://img.shields.io/crates/v/dfe)](https://crates.io/crates/dfe)
[![Docs.rs](https://docs.rs/dfe/badge.svg)](https://docs.rs/dfe)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Sponsor GustavoOta](https://img.shields.io/badge/sponsor-GustavoOta-%23EA4AAA?style=flat&logo=github)](https://github.com/sponsors/GustavoOta)

---

## Funcionalidades

| Funcionalidade | Descrição |
|---|---|
| **Emissão NF-e / NFC-e** | Autorização via SOAP para SEFAZ estadual (modelos 55 e 65) |
| **Cancelamento** | Evento 110111 para NF-e e NFC-e |
| **Manifestação do destinatário** | Ciência, confirmação, desconhecimento e operação não realizada |
| **Distribuição de DF-e** | Consulta ao Ambiente Nacional por NSU ou chave de acesso |
| **DANFE** | Geração de PDF em A4 e 80mm (NF-e e NFC-e) com suporte a logotipo |
| **ESC/POS** | `EscPosBuilder` (layout livre) + `EscPosNFCeBuilder` (NFC-e pronto) |
| **Validação XSD** | Schemas SEFAZ embutidos no binário — sem arquivos externos |
| **Validação CNPJ/CPF** | Verificação de dígito verificador |
| **Status do webservice** | Consulta de disponibilidade por UF e ambiente |

---

## Documentação

| Seção | Descrição |
|---|---|
| [Instalação](docs/instalacao.md) | Dependências de build (OpenSSL, libxml2/vcpkg), linking estático no Windows |
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

---

## Início rápido

```toml
[dependencies]
dfe = "0.5.8"
```

> **Windows:** antes de compilar, leia [docs/instalacao.md](docs/instalacao.md) — são necessários Strawberry Perl (OpenSSL) e vcpkg (libxml2 estático).

---

## Licença

MIT — veja [LICENSE](LICENSE).
