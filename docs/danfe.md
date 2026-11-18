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

## Formatos implementados

| Tamanho | Modelo 55 (NF-e) | Modelo 65 (NFC-e) |
|---|:---:|:---:|
| `"a4"` | ✅ (suporta `.logo()`) | ❌ |
| `"80mm"` | ✅ | ✅ (suporta `.qr_side()`) |
| `"54mm"` | ❌ | ❌ |

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

> **Limite conhecido:** nota com muitos itens não é quebrada em várias folhas A4 — a página
> cresce em altura (a amostra de 60 itens sai com ~485 mm).

## Colunas de ICMS dos itens (A4)

A base, a alíquota e o valor do ICMS de cada item vêm do grupo que o item tiver no XML —
ICMS00, 10, 20, 51, 70, 90, ICMSPart, ICMSST, SN101/201/202 etc. (`xml_extractor::ICMS::grupo`).
Até 30-09-2026 só o ICMS00 era lido e os demais CST saíam com as colunas em branco. Grupo sem
esses campos (40, 60, monofásico, SN102) sai em branco, como manda o leiaute.
