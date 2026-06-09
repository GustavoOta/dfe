# ESC/POS — Impressoras Térmicas

## EscPosBuilder — layout personalizado

```rust
use dfe::EscPosBuilder;

let bytes = EscPosBuilder::new()
    .paper_width(80)               // 80 mm ou 58 mm
    .align_center()
    .bold(true)
    .text("EMPRESA LTDA\n")
    .bold(false)
    .align_left()
    .text("CNPJ: 11.222.333/0001-81\n")
    .divider()
    .text(format!("{:<20} {:>10}\n", "PRODUTO EXEMPLO", "R$  50,00"))
    .divider()
    .align_right()
    .bold(true)
    .text("TOTAL  R$  50,00\n")
    .bold(false)
    .cut()
    .build();                      // → Vec<u8>

// Enviar para impressora
std::fs::write("\\\\.\\COM3", &bytes)?;   // Windows — porta serial
std::fs::write("/dev/usb/lp0", &bytes)?; // Linux — USB
```

| Método | Descrição |
|---|---|
| `.paper_width(mm)` | Largura do papel: `80` ou `58`. Padrão: `80` |
| `.printer_dpi(dpi)` | DPI nativo da impressora (`203` padrão, `300` alta resolução) |
| `.printable_dots(dots)` | Largura imprimível em dots (alternativa ao DPI) |
| `.columns(n)` | Sobrescreve o número de colunas de texto. Padrão: 48 (80 mm) · 32 (58 mm). EPSON: 48 · Bematech MP-4200 TH: 42 |
| `.align_left()` / `.align_center()` / `.align_right()` | Alinhamento |
| `.bold(bool)` | Negrito (`ESC E`) |
| `.underline(bool)` | Sublinhado (`ESC -`) |
| `.font_size(u8)` | Tamanho da fonte (1 = normal, 2 = duplo) |
| `.font_b(bool)` | Fonte B — menor e condensada (`ESC M`) |
| `.line_spacing(dots)` | Espaçamento entre linhas em dots (`ESC 3`) |
| `.text(str)` | Insere texto (codificado em CP850) |
| `.divider()` | Linha separadora proporcional à largura do papel |
| `.barcode_128(str)` | Code 128 como imagem raster (GS v 0) |
| `.qr_code(str, size)` | QR Code nativo ESC/POS (GS ( k) |
| `.image(bytes)` | Imagem PNG/JPEG rasterizada → bitmap 1-bit (GS v 0) |
| `.feed(n)` | Avança `n` linhas (`ESC d`) |
| `.cut()` | Corte total (`GS V 0`) |
| `.partial_cut()` | Corte parcial (`GS V 1`) |
| `.open_drawer(pin)` | Abre gaveta de dinheiro — `2` ou `5` |
| `.build()` | Retorna `Vec<u8>` |

---

## EscPosNFCeBuilder — impressão de NFC-e (modelo 65)

Gera o cupom completo da NFC-e a partir do XML autorizado pela SEFAZ.
Aceita o XML como caminho de arquivo (`.xml`) ou string direta.

### QR Code centralizado (padrão)

```rust
use dfe::EscPosNFCeBuilder;

let bytes = EscPosNFCeBuilder::new()
    .xml("caminho/nota.xml")   // ou string XML diretamente
    .paper_width(80)           // padrão: 80 mm
    .build()?;                 // → Result<Vec<u8>, DfeError>

std::fs::write("\\\\.\\COM3", &bytes)?;
```

**Layout:**
```
         AMBIENTE DE HOMOLOGAÇÃO         ← apenas tp_amb = 2
           SEM VALOR FISCAL
------------------------------------------------
      Documento Auxiliar da NFC-e
------------------------------------------------
          NOME DO EMITENTE
  CNPJ: XX.XXX.XXX/XXXX-XX  IE: XXXXXXXX  SP
        Rua Exemplo, 100 - Centro
               Cidade/SP
------------------------------------------------
#  DESCRICAO                 Qtd  UN  VlUnit  Total
------------------------------------------------
1  Produto Exemplo
   1,000  UN  R$ 10,00             R$ 10,00
------------------------------------------------
Qtd. Itens: 1
TOTAL                                R$ 10,00
------------------------------------------------
FORMA DE PAGAMENTO                      VALOR
PIX                                  R$ 10,00
Troco:                                R$  0,00
------------------------------------------------
Consulte pela Chave de Acesso em
www.nfce.fazenda.sp.gov.br/consulta
3524 0612 3456 7890 0001 6500 1000 0000 1...
[===== CODE 128 =====]
          [QR CODE CENTRALIZADO]
------------------------------------------------
PROTOCOLO DE AUTORIZAÇÃO
135XXXXXX - 04/06/2026 14:30:00
NF-e No 000000001  Serie 1  04/06/2026 14:30:00
------------------------------------------------
CONSUMIDOR NÃO IDENTIFICADO
```

### QR Code lateral (`.qr_side()`)

Imprime QR Code à esquerda e informações de protocolo, série/número,
data de emissão e cliente à direita — tudo como imagem raster.
Útil para impressoras com papel mais estreito ou para economizar papel.

```rust
use dfe::EscPosNFCeBuilder;

// Layout lateral — impressora padrão 203 DPI
let bytes = EscPosNFCeBuilder::new()
    .xml(xml_string)
    .paper_width(80)
    .qr_side()
    .build()?;

// Layout lateral — impressora 300 DPI (ajusta escala automaticamente)
let bytes = EscPosNFCeBuilder::new()
    .xml(xml_string)
    .paper_width(80)
    .printer_dpi(300)
    .qr_side()
    .build()?;

std::fs::write("\\\\.\\COM3", &bytes)?;
```

**Layout (`.qr_side()`):**
```
------------------------------------------------
Consulte pela Chave de Acesso em
www.nfce.fazenda.sp.gov.br/consulta
3524 0612 3456 7890 0001 6500 1000 0000 1...
[===== CODE 128 =====]
┌─────────────┐  Protocolo:
│             │  135XXXXXX
│  QR CODE   │  NFC-e Serie/Num:
│  (~33 mm)  │  1 / 000000001
│             │  Data emissao:
│             │  04/06/2026
└─────────────┘  14:30:00
                 Cliente
                 JOSE DA SILVA
```

### Impressão direta no Windows (`.print()`)

Em Windows, o método `.print()` envia os bytes diretamente à impressora como job RAW,
sem diálogo — equivalente ao `WritePrinter` da WinAPI.

```rust
use dfe::EscPosNFCeBuilder;

EscPosNFCeBuilder::new()
    .xml("caminho/nota.xml")
    .paper_width(80)
    .printer_name("EPSON TM-T20 Receipt")  // nome exato do Painel de Controle
    .print()?;                              // disponível apenas em Windows

// Com QR lateral e DPI personalizado
EscPosNFCeBuilder::new()
    .xml(xml_string)
    .paper_width(80)
    .printer_dpi(203)
    .qr_side()
    .printer_name("POS-80C")
    .print()?;
```

> O nome da impressora deve coincidir exatamente com o exibido em
> **Painel de Controle → Dispositivos e Impressoras**.

### Impressoras testadas

| Impressora | Papel | Colunas | Observações |
|---|---|---|---|
| EPSON TM-T20X | 80 mm | 48 (padrão) | Funciona sem configuração adicional |
| Bematech MP-4200 TH | 80 mm | 42 | Requer `.columns(42)` |

### Compatibilidade de colunas por modelo de impressora

Impressoras de 80 mm podem diferir no número de colunas imprimíveis.
Use `.columns(n)` para ajustar quando o cupom tiver texto truncado ou ultrapassar a margem.

| Impressora | Colunas (fonte A, 80 mm) |
|---|---|
| EPSON TM-T20X e similares | 48 (padrão) |
| Bematech MP-4200 TH e similares | 42 |

```rust
// Bematech MP-4200 TH — 42 colunas
EscPosNFCeBuilder::new()
    .xml("nota.xml")
    .paper_width(80)
    .columns(42)
    .printer_name("MP-4200 TH")
    .print()?;

// EPSON — padrão (48 colunas, .columns() desnecessário)
EscPosNFCeBuilder::new()
    .xml("nota.xml")
    .paper_width(80)
    .printer_name("EPSON TM-T20 Receipt")
    .print()?;
```

### Referência dos métodos

| Método | Descrição |
|---|---|
| `.xml(src)` | XML do `nfeProc` autorizado — caminho de arquivo ou string |
| `.paper_width(mm)` | Largura do papel: `80` ou `58`. Padrão: `80` |
| `.printer_dpi(dpi)` | DPI da impressora para cálculo de escala do QR (`203` padrão, `300` alta resolução) |
| `.printable_dots(dots)` | Largura imprimível em dots nativos (alternativa ao DPI) |
| `.columns(n)` | Sobrescreve o número de colunas de texto. Padrão: 48 (80 mm) · 32 (58 mm). EPSON: 48 · Bematech MP-4200 TH: 42 |
| `.qr_side()` | QR Code à esquerda com informações à direita (imagem raster) |
| `.printer_name(name)` | Nome da impressora Windows para `.print()` |
| `.build()` | Gera e retorna `Vec<u8>` |
| `.print()` | Gera e envia diretamente à impressora (somente Windows) |

### Conteúdo do cupom

| Seção | Origem no XML |
|---|---|
| Homologação | `ide/tpAmb = 2` |
| Emitente | `emit/xFant` ou `emit/xNome`, CNPJ, IE, endereço |
| Itens | `det/prod` — descrição, qtd, unidade, valor unit., total |
| Totais | `total/ICMSTot` — subtotal, desconto, TOTAL |
| Pagamentos | `pag/detPag` — forma e valor; troco de `pag/vTroco` |
| Chave de acesso | `infNFe/@Id` (sem prefixo `NFe`) |
| Code 128 | Chave de acesso em barras |
| QR Code | `infNFeSupl/qrCode` |
| URL consulta | `infNFeSupl/urlChave` |
| Protocolo | `protNFe/infProt/nProt` e `dhRecbto` |
| Cliente | `dest/xNome` (quando informado) |
| Tributos | `total/ICMSTot/vTotTrib` |
| Obs. complementares | `infAdic/infCpl` |

### Erros comuns

| Erro | Causa |
|---|---|
| `Configuracao("XML não informado")` | `.xml()` não foi chamado |
| `Configuracao("EscPosNFCeBuilder espera modelo 65...")` | XML é NF-e (modelo 55), não NFC-e |
| `Configuracao("printer_name não informado")` | `.print()` sem `.printer_name()` |
| `Configuracao("Falha ao abrir impressora '...'")` | Nome da impressora incorreto ou impressora offline |
