# ESC/POS — Impressoras Térmicas

## EscPosBuilder — layout personalizado

```rust
use dfe::EscPosBuilder;

let bytes = EscPosBuilder::new()
    .paper_width(80)               // 80mm ou 58mm
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
std::fs::write("\\\\.\\COM3", &bytes)?;   // Windows
std::fs::write("/dev/usb/lp0", &bytes)?; // Linux
```

| Método | Descrição |
|---|---|
| `.paper_width(mm)` | Largura do papel: `80` ou `58`. Padrão: `80` |
| `.align_left()` / `.align_center()` / `.align_right()` | Alinhamento |
| `.bold(bool)` | Negrito |
| `.underline(bool)` | Sublinhado |
| `.font_size(u8)` | Tamanho da fonte (1 = normal, 2 = duplo) |
| `.text(str)` | Insere texto |
| `.divider()` | Linha separadora proporcional à largura do papel |
| `.barcode_128(str)` | Code 128 nativo ESC/POS |
| `.qr_code(str, size)` | QR Code nativo ESC/POS |
| `.image(bytes)` | Imagem rasterizada PNG/JPEG → bitmap 1-bit (GS v 0) |
| `.feed(n)` | Avança `n` linhas |
| `.cut()` | Corte total |
| `.partial_cut()` | Corte parcial |
| `.build()` | Retorna `Vec<u8>` |

---

## EscPosNFCeBuilder — impressão de NFC-e

```rust
use dfe::EscPosNFCeBuilder;

// QR Code centralizado (padrão)
let bytes = EscPosNFCeBuilder::new()
    .xml("caminho/nota.xml")   // ou string XML diretamente
    .paper_width(80)           // padrão: 80mm; 58mm também suportado
    .build()?;                 // → Result<Vec<u8>, DfeError>

// QR Code à esquerda (compacto)
let bytes = EscPosNFCeBuilder::new()
    .xml(xml_string)
    .qr_side()
    .build()?;

std::fs::write("\\\\.\\COM3", &bytes)?;
```

**Layout do cupom:**
```
         NOME DO EMITENTE
  CNPJ: XX.XXX.XXX/XXXX-XX  IE: XXXXXXXX  SP
  Rua Exemplo, 100 - Centro / Cidade/SP
------------------------------------------------
DOCUMENTO AUXILIAR DA NFC-E
------------------------------------------------
 #  DESCRICAO
    QTD UN  R$ VL.UNIT           R$ TOTAL
------------------------------------------------
Qtd. Itens: N
                              TOTAL  R$ XXX,XX
------------------------------------------------
FORMA DE PAGAMENTO                       VALOR
Dinheiro                             R$ XXX,XX
------------------------------------------------
Consulte pela Chave de Acesso em
www.nfce.fazenda.sp.gov.br/consulta
CHAVE DE ACESSO
3524 0600 0000 0000 ...
[CODE 128]
[QR CODE]
------------------------------------------------
PROTOCOLO DE AUTORIZACAO
135XXXXXX - DD/MM/YYYY HH:MM:SS
NF-e No 000000001  Serie 1  DD/MM/YYYY HH:MM:SS
------------------------------------------------
Valor Aproximado dos Tributos R$ X,XX (Fonte: IBPT)
```
