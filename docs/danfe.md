# DANFE — Geração de PDF

Gera o DANFE em PDF a partir do XML autorizado (`nfeProc`). O modelo (55 ou 65) é detectado automaticamente do campo `<mod>` no XML.

```rust
use dfe::DanfeBuilder;

// NF-e A4 — salvar como arquivo
let caminho = DanfeBuilder::new()
    .xml("./nota_autorizada.xml")   // caminho .xml ou string XML diretamente
    .paper_size("a4")               // padrão quando omitido
    .as_file("./danfe.pdf")
    .build()
    .await?;

println!("PDF salvo em: {}", caminho);

// NF-e A4 — com logotipo do emitente, retornar base64
let b64 = DanfeBuilder::new()
    .xml("<nfeProc>...</nfeProc>")
    .paper_size("a4")
    .logo("./logo.png")   // .png/.jpg, base64 puro ou data URI
    .as_base64()
    .build()
    .await?;

// NF-e 80mm
let b64 = DanfeBuilder::new()
    .xml("<nfeProc>...</nfeProc>")
    .paper_size("80mm")
    .as_base64()
    .build()
    .await?;

// NFC-e 80mm — QR Code lateral
let b64 = DanfeBuilder::new()
    .xml("<nfeProc>...</nfeProc>")
    .paper_size("80mm")
    .qr_side()   // QR Code à esquerda (~33mm); chave e protocolo à direita
    .as_base64()
    .build()
    .await?;
```

## Métodos do DanfeBuilder

| Método | Obrigatório | Descrição |
|---|:---:|---|
| `.xml(src)` | ✅ | Caminho `.xml` ou string do `nfeProc` |
| `.paper_size(str)` | — | `"a4"` (padrão), `"80mm"` ou `"54mm"` |
| `.as_file(path)` | ✅¹ | Salva o PDF em disco; retorna o caminho |
| `.as_base64()` | ✅¹ | Retorna o PDF como string base64 |
| `.logo(src)` | — | Logotipo do emitente — apenas A4 |
| `.qr_side()` | — | Layout QR lateral — apenas NFC-e 80mm |

¹ Use `.as_file()` **ou** `.as_base64()` — nunca os dois.

`build()` é `async` e devolve `Result<String, String>`: o caminho do arquivo ou o base64 do PDF;
no erro, a mensagem. Numa função que devolve `DfeError`, converta com `.map_err(DfeError::Validacao)`
ou equivalente.

## Formatos implementados

| Tamanho | Modelo 55 (NF-e) | Modelo 65 (NFC-e) |
|---|:---:|:---:|
| `"a4"` | ✅ (suporta `.logo()`) | ❌ |
| `"80mm"` | ✅ | ✅ (suporta `.qr_side()`) |
| `"54mm"` | ❌ | ❌ |

## Paginação da NF-e em A4

O `DanfeBuilder` gera um único PDF com folhas de 210 × 297 mm, numeradas no cabeçalho
como `FOLHA 1/1` ou `FOLHA 1/2`, conforme o total. Cada folha repete o cabeçalho
fiscal, incluindo a chave de acesso e o código de barras.

A quebra depende da altura disponível. A sublinha de IBS/CBS/IS acompanha o respectivo
item e a tabela conserva as mesmas colunas nas folhas seguintes. As informações
complementares (`infCpl`) continuam em outra folha quando ultrapassam o espaço
disponível. O quadro reservado ao Fisco fica na primeira folha.

## Modelo da Reforma Tributária no A4 (NT 2026.010)

A partir de 01/12/2026 o DANFE A4 traz o **CRT** no quadro do emitente, o bloco **"TOTAL DO
IBS / CBS / IS"** (com a linha da monofasia só quando o XML tem `gMono`) e, em cada item com
IBS/CBS ou IS, uma **sublinha** com cClassTrib, base, alíquota (a efetiva quando há `gRed`) e
valor. Campo ausente no XML sai em branco — nota sem IBS/CBS fica com o bloco vazio e sem
sublinhas.

### Ver o resultado sem certificado

```bash
cargo test --test danfe_a4_amostras -- --nocapture
```

Gera seis PDFs de NF-e fictícias em `target/tmp/danfe-a4/` (ou na pasta de `DFE_DANFE_SAIDA`) e
imprime o caminho de cada um: Simples sem IBS/CBS, regime normal com IBS/CBS, redução de alíquota
com Imposto Seletivo, monofasia, 60 itens e MEI. Não precisa de certificado nem de rede.

Para gravar em outra pasta:

```bash
# Linux/macOS
DFE_DANFE_SAIDA=./amostras cargo test --test danfe_a4_amostras -- --nocapture
# Windows (PowerShell)
$env:DFE_DANFE_SAIDA = "./amostras"; cargo test --test danfe_a4_amostras -- --nocapture
```

| Arquivo | O que mostra |
|---|---|
| `01-simples-sem-ibs-cbs.pdf` | Simples Nacional (CRT 1) sem IBS/CBS: bloco "TOTAL DO IBS / CBS / IS" em branco e itens sem sublinha |
| `02-regime-normal-ibs-cbs.pdf` | Regime normal (CRT 3) com IBS/CBS em todos os itens |
| `03-reducao-e-imposto-seletivo.pdf` | Redução de alíquota (`gRed`: sai a alíquota efetiva) e Imposto Seletivo |
| `04-monofasico.pdf` | Totais com `gMono`: aparece a linha da monofasia |
| `05-muitos-itens.pdf` | Duas folhas A4 com 60 itens, metade com IBS/CBS |
| `06-mei.pdf` | MEI (CRT 4) |

As notas são fictícias (dados e valores ilustrativos).

## Colunas de ICMS dos itens (A4)

A base, a alíquota e o valor do ICMS de cada item vêm do grupo que o item tiver no XML —
ICMS00, 10, 20, 51, 70, 90, ICMSPart, ICMSST, SN101/201/202 etc. (`xml_extractor::ICMS::grupo`).
Até 30-09-2026 só o ICMS00 era lido e os demais CST saíam com as colunas em branco. Grupo sem
esses campos (40, 60, monofásico, SN102) sai em branco, como manda o leiaute.
