# dfe

Crate Rust para integração com os webservices da SEFAZ brasileira. É uma **biblioteca** (`lib`) — não tem binário próprio. Consumida principalmente pelo `dfe-api` (`../dfe-api`).

## O que esta crate faz

- Emite NF-e e NFC-e (autorização via SOAP para SEFAZ estadual)
- Cancela NF-e e NFC-e (evento 110111)
- Manifestação do destinatário (ciência, confirmação, desconhecimento, operação não realizada)
- Distribui documentos fiscais (consulta ao Ambiente Nacional — AN)
- Gera DANFE em PDF (modelo 55 em 80mm; outros tamanhos em desenvolvimento)
- Valida XML contra XSD oficial da SEFAZ
- Assina XML digitalmente (XML-DSig, RSA-SHA1)
- Lê e usa certificados digitais A1 (.pfx / PKCS#12)

## Stack

| Responsabilidade | Crate |
|---|---|
| Criptografia / Assinatura digital | `openssl` (vendored) |
| HTTP client (MTLS) | `reqwest` + `native-tls` |
| XML parsing/serialização | `quick-xml`, `serde-xml-rs` |
| Validação XSD | `libxml` (wrapper C libxml2) |
| Geração de PDF | `printpdf` |
| Código de barras (Code128) | `barcoders` |
| Hash SHA1 / SHA256 | `sha1`, `sha2` |
| Decimal preciso | `rust_decimal` |
| Compressão GZIP | `flate2` |
| Async runtime | `tokio` (full) |

## Estrutura dos módulos

```
src/
├── lib.rs                    ← exporta: pub mod nfe, distribuicao, pdf; pub use pdf::*
├── nfe/
│   ├── mod.rs                ← service_status(), crypt(), decrypt()
│   ├── autorizacao/          ← emit() — emissão de NF-e / NFC-e
│   ├── cancelar/             ← nfe_cancelar()
│   ├── manifestacao/         ← nfe_ciencia_operacao(), nfe_confirmacao_operacao(), ...
│   ├── mod_status_service/   ← status do webservice SEFAZ
│   ├── common/
│   │   ├── cert.rs           ← leitura de .pfx, assinatura digital, extração de chave
│   │   ├── chave_acesso.rs   ← geração e validação de chave de acesso (44 dígitos)
│   │   ├── validation.rs     ← validação XML contra XSD (auto-download dos arquivos XSD)
│   │   ├── ws.rs             ← mapa de URLs dos webservices SEFAZ por UF/ambiente/modelo
│   │   ├── cleaner.rs        ← limpeza de strings XML (simula C14N)
│   │   ├── config_file.rs    ← criptografia AES-256-CBC da senha do certificado
│   │   ├── dates.rs          ← helpers de data/hora
│   │   └── extract.rs        ← extração de dados de structs
│   ├── connection/
│   │   └── mod.rs            ← WebService: cria cliente reqwest com TLS mútuo + envia SOAP
│   ├── types/                ← structs de dados (NFe, Det, Total, NFeCancelar, config...)
│   ├── xml_extractor/        ← trait XmlExtractorSignature: XML → structs Rust
│   ├── xml_rules/            ← validações de regras de negócio (UF, CFOP, estado dest.)
│   └── autorizacao/flag/     ← flag de autorização (homologação / produção)
├── distribuicao/
│   ├── mod.rs                ← builders fluentes (Distribuicao, DistribuicaoNSU, ...)
│   ├── service.rs            ← comunicação SOAP com Ambiente Nacional (AN)
│   └── test.rs               ← testes de integração
└── pdf/
    ├── mod.rs / builder.rs   ← DanfeBuilder (builder fluente)
    ├── actions/
    │   └── pdf_builder_80mm.rs ← implementação DANFE 80mm (modelo 55)
    └── validations/          ← validações de entrada
```

## API pública

### Emissão de NF-e

```rust
use dfe::nfe::autorizacao::emit;
use dfe::nfe::types::autorizacao4::NFe;

let resposta = emit(nfe).await?;
// resposta.protocolo: TagInfProt
// resposta.xml: String (XML autorizado)
```

### Cancelamento

```rust
use dfe::nfe::cancelar::nfe_cancelar;
use dfe::nfe::types::cancelar::NFeCancelar;

let resposta = nfe_cancelar(params).await?;
// resposta.response: InfEvento
// resposta.send_xml / receive_xml: String
```

### Manifestação do destinatário

```rust
use dfe::nfe::manifestacao::{
    nfe_ciencia_operacao,
    nfe_confirmacao_operacao,
    nfe_desconhecimento_operacao,
    nfe_operacao_nao_realizada,
};
```

Tipos de evento SEFAZ:
- `210210` — Ciência da Operação
- `210200` — Confirmação da Operação
- `210220` — Desconhecimento da Operação
- `210240` — Operação Não Realizada (requer justificativa)

### Distribuição (Ambiente Nacional)

Builder fluente:

```rust
use dfe::distribuicao::Distribuicao;

let resposta = Distribuicao::new()
    .cert_path("caminho.pfx")
    .cert_pass("senha")
    .cnpj("11111111111111")
    .uf(35)
    .ambiente(2)
    .send()
    .await?;
```

Builders disponíveis: `Distribuicao`, `DistribuicaoNSU`, `DistribuicaoChaveAcesso`,
`CienciaOperacao`, `ConfirmacaoOperacao`, `DesconhecimentoOperacao`, `OperacaoNaoRealizada`.

A resposta pode conter `docZip` — conteúdo codificado em base64 + GZIP. O serviço decodifica automaticamente.

### DANFE (PDF)

```rust
use dfe::pdf::DanfeBuilder;

// Salvar em arquivo
let caminho = DanfeBuilder::new()
    .xml(xml_str)
    .paper_size("80mm")
    .as_file("nota.pdf")
    .build()
    .await?;

// Retornar como base64
let b64 = DanfeBuilder::new()
    .xml(xml_str)
    .paper_size("80mm")
    .build()
    .await?;
```

Tamanhos implementados: `"80mm"` (modelo 55). A4 e 54mm ainda não implementados.

---

## Detalhes técnicos críticos

### Certificado digital (.pfx)

O certificado A1 (PKCS#12) é usado em dois contextos independentes:

1. **TLS mútuo (MTLS)** — A SEFAZ exige autenticação por certificado na camada TLS. O cliente reqwest é criado com `Identity::from_pkcs12_der`.
2. **Assinatura XML** — A chave privada assina o `SignedInfo`; o certificado X.509 vai no `KeyInfo` do XML.

Nunca cachear nem reutilizar o objeto `Cert` entre requisições — lê do arquivo a cada operação.

### Assinatura digital XML (XML-DSig)

Padrão exigido pela SEFAZ: **XML-DSig com RSA-SHA1**.

Fluxo para cancelamento/manifestação:
```
infEvento (XML)
  → SHA1 → DigestValue (base64)
  → SignedInfo (XML com DigestValue)
  → "limpa" espaços (simula C14N REC-xml-c14n-20010315)
  → RSA-SHA1 com chave privada → SignatureValue (base64)
  → Signature = SignedInfo + SignatureValue + X509Certificate
```

**Importante:** A canonicalização C14N não usa uma biblioteca dedicada — é simulada via `cleaner.rs` que remove espaços entre tags e quebras de linha. Qualquer alteração nessa limpeza pode invalidar assinaturas.

### Comunicação SEFAZ (SOAP 1.2)

Todos os webservices usam SOAP 1.2 (não REST, não SOAP 1.1).

Header obrigatório: `Content-Type: application/soap+xml; charset=utf-8`

Cabeçalho SEFAZ no SOAP (`nfeCabecMsg`): `cUFAutor` (código IBGE da UF) + `versaoDados`.

As URLs dos webservices ficam em `nfe/common/ws.rs`. São hardcoded por UF + ambiente + modelo (55/65) + SVC (contingência). Ao adicionar suporte a nova UF, editar esse arquivo.

### Validação XSD

A validação usa `libxml` (wrapper da libxml2 em C). Os arquivos XSD são baixados automaticamente de `https://raw.githubusercontent.com/GustavoOta/dfe/main/dfe/shema/...` na primeira execução e salvos em `./dfe/shema/PL_010b_NT2025_002_v1.21/`.

Se o diretório não existir ou os arquivos estiverem ausentes, a validação tenta baixá-los. Em ambientes sem acesso à internet, os arquivos devem ser pré-instalados.

XSD principal: `nfe_v4.00.xsd` (referencia os demais).

### Chave de acesso

Gerada em `nfe/common/chave_acesso.rs`. Composição (43 dígitos + 1 DV = 44):

```
cUF(2) + AAMM(4) + CNPJ(14) + mod(2) + serie(3) + nNF(9) + tpEmis(1) + cNF(8) + cDV(1)
```

DV: módulo 11 com pesos cíclicos `[2..9]`. Resultado 0 ou 1 → DV = 0.

### Error handling

**Padrão adotado:** `Result<T, String>` — usar em todo código novo.

Código legado ainda usa `anyhow::Error` e `XMLExtractorError`; ambos serão removidos gradualmente. Ao tocar em funções que retornam esses tipos, migrar para `Result<T, String>`:

```rust
// legado — não usar em código novo
fn foo() -> Result<T, anyhow::Error> { ... }

// padrão atual
fn foo() -> Result<T, String> {
    algo().map_err(|e| e.to_string())?;
    Ok(valor)
}
```

Ao consumir funções legadas com `anyhow::Error` de dentro de código novo, converter na chamada:
```rust
funcao_legada().map_err(|e| e.to_string())?;
```

---

## Builder pattern — onde implementar

**Esta crate é o lugar correto para builders.** Ao implementar ou refatorar qualquer construção de XML, SOAP, PDF ou comunicação SEFAZ, o builder deve ficar aqui — não no `dfe-api` nem em outros consumidores.

Regra: se a lógica envolve estrutura de documento fiscal, assinatura, protocolo SEFAZ ou geração de output (PDF, XML), pertence ao `dfe`. O consumidor (`dfe-api`) apenas transforma o payload da API nas structs desta crate e chama o builder.

```
dfe-api             →       dfe
transforma JSON     →       constrói / valida / assina / envia
```

Padrão de builder fluente adotado (ver `distribuicao/mod.rs` como referência):

```rust
pub struct MeuBuilder {
    cert_path: Option<String>,
    campo: Option<String>,
}

impl MeuBuilder {
    pub fn new() -> Self { Self { cert_path: None, campo: None } }
    pub fn cert_path(mut self, v: &str) -> Self { self.cert_path = Some(v.to_string()); self }
    pub fn campo(mut self, v: &str) -> Self { self.campo = Some(v.to_string()); self }
    pub async fn send(self) -> Result<Resposta, String> { ... }
}
```

## Convenções internas

- `shemas/` — pasta de structs mapeadas (mesmo padrão do gravisServer; grafia intencional do projeto)
- `tp_amb: u8` — `1` = Produção, `2` = Homologação
- `mod_: u32` — `55` = NF-e, `65` = NFC-e
- Campos do XML SEFAZ mantêm a nomenclatura original em snake_case: `c_stat`, `x_motivo`, `dh_recbto`, `n_prot`

## Comandos

```bash
cargo build              # compilar
cargo test               # rodar testes unitários
cargo doc --open         # gerar e abrir documentação
```

Não há `build-release.bat` neste crate — o release é feito pelo consumidor (`dfe-api`).
