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
| **Validação XSD** | Schemas SEFAZ embutidos no binário — sem arquivos externos |
| **Status do webservice** | Consulta de disponibilidade por UF e ambiente |

---

## Instalação

```toml
[dependencies]
dfe = "0.5.7"
```

> **OpenSSL:** No Windows, a feature `vendored` é usada automaticamente — nenhuma instalação manual necessária.  
> **Schemas XSD** (`PL_010b_NT2025_002_v1.21`) estão embutidos no binário via `include_bytes!`.

---

## Tratamento de erros

Todas as funções públicas retornam `Result<T, DfeError>`.

```rust
use dfe::DfeError;

match resultado {
    Ok(r)                        => { /* ... */ }
    Err(DfeError::Certificado(m)) => eprintln!("Problema no .pfx: {}", m),
    Err(DfeError::Validacao(m))   => eprintln!("Dado inválido: {}", m),
    Err(DfeError::Webservice(m))  => eprintln!("Falha SEFAZ: {}", m),
    Err(e)                        => eprintln!("Erro: {}", e),
}
```

| Variante | Quando ocorre |
|---|---|
| `Certificado` | Falha ao abrir o `.pfx` ou senha incorreta |
| `Xml` | Erro de parsing ou serialização XML |
| `Assinatura` | Falha na assinatura digital RSA-SHA1 |
| `Webservice` | Erro HTTP ou resposta inesperada da SEFAZ |
| `Validacao` | Campo obrigatório ausente ou fora das regras XSD |
| `Configuracao` | Falha ao ler configuração ou credenciais |
| `Io` | Erro de leitura/escrita em disco |

---

## NFeBuilder — Emissão de NF-e / NFC-e

```rust
use dfe::NFeBuilder;
use dfe::tipos::{Det, Emit, Icms, Ide, InfAdic, Pag, Pis, Cofins, Total, Transp};
use dfe::tipos::emissao::Dest;

let resposta = NFeBuilder::new()
    .cert("./cert.pfx", "senha_do_pfx")
    .ide(Ide {
        c_uf: 35,           // cUF da UF emitente (SP = 35)
        mod_: 55,           // 55 = NF-e | 65 = NFC-e
        serie: 1,
        n_nf: 100,
        tp_amb: 2,          // 1 = Produção | 2 = Homologação
        tp_nf: 1,           // 0 = Entrada | 1 = Saída
        nat_op: "VENDA DE MERCADORIA".to_string(),
        ..Default::default()
    })
    .emitente(Emit {
        cnpj: Some("11111111111111".to_string()),
        x_nome: Some("EMPRESA LTDA".to_string()),
        ie: Some("111111111111".to_string()),
        crt: 1,             // 1 = Simples Nacional | 3 = Regime Normal
        ..Default::default()
    })
    .destinatario(Dest {   // opcional para NFC-e
        cnpj: Some("22222222222222".to_string()),
        x_nome: Some("CLIENTE LTDA".to_string()),
        ..Default::default()
    })
    .itens(vec![Det {
        c_prod: "001".to_string(),
        x_prod: "PRODUTO EXEMPLO".to_string(),
        ncm: "22030000".to_string(),
        cfop: 5102,
        u_com: "UN".to_string(),
        q_com: 2.0,
        v_un_com: 50.0,
        v_prod: 100.0,
        u_trib: "UN".to_string(),
        q_trib: 2.0,
        v_un_trib: 50.0,
        icms: Icms::sn102(0, "400".to_string()),      // CSOSN 400
        pis: Pis::Nt { cst: "07".to_string() },
        cofins: Cofins::Nt { cst: "07".to_string() },
        ..Default::default()
    }])
    .total(Total::default())         // totais calculados automaticamente dos itens
    .transporte(Transp { mod_frete: Some("9".to_string()), ..Default::default() })
    .pagamento(Pag { ..Default::default() })
    .informacoes_adicionais(InfAdic {
        inf_cpl: Some("Pedido 123".to_string()),
        ..Default::default()
    })
    // NFC-e: obrigatório também .id_csc() e .csc()
    .emitir()
    .await?;

println!("cStat:    {}", resposta.protocolo.inf_prot.c_stat);
println!("xMotivo:  {}", resposta.protocolo.inf_prot.x_motivo);
println!("Protocolo: {}", resposta.protocolo.inf_prot.n_prot.unwrap_or_default());
// resposta.xml → XML autorizado (nfeProc) completo
```

### Métodos do NFeBuilder

| Método | Obrigatório | Descrição |
|---|:---:|---|
| `.cert(path, pass)` | ✅ | Caminho e senha do certificado `.pfx` |
| `.ide(Ide)` | ✅ | Identificação do documento |
| `.emitente(Emit)` | ✅ | Dados do emitente |
| `.itens(Vec<Det>)` | ✅ | Lista de itens; totais calculados automaticamente |
| `.total(Total)` | ✅ | Informar apenas frete, seguro, ST, FCP — demais campos auto-calculados |
| `.transporte(Transp)` | ✅ | Modalidade de frete e dados do transportador |
| `.pagamento(Pag)` | ✅ | Forma de pagamento |
| `.destinatario(Dest)` | — | Obrigatório para NF-e mod 55 |
| `.informacoes_adicionais(InfAdic)` | — | Informações complementares e ao fisco |
| `.id_csc(str)` | — | ID do CSC — **obrigatório NFC-e** |
| `.csc(str)` | — | Valor do CSC — **obrigatório NFC-e** |
| `.desconto_rateio(Decimal)` | — | Desconto global rateado proporcionalmente nos itens |
| `.emitir()` | — | Valida, assina e transmite para a SEFAZ |

### Totais automáticos

Os campos `v_bc`, `v_icms`, `v_prod`, `v_pis`, `v_cofins`, `v_desc` e `v_nf` são **calculados automaticamente** dos itens. No `Total` informe apenas despesas extras:

| Campo | Quando usar |
|---|---|
| `v_frete`, `v_seg`, `v_outro` | Frete, seguro e outras despesas |
| `v_ii`, `v_ipi`, `v_ipi_devol` | Impostos específicos |
| `v_bc_st`, `v_st` | Substituição Tributária |
| `v_fcp`, `v_fcpst`, `v_fcpst_ret` | Fundo de Combate à Pobreza |

Para uma venda simples sem extras: `Total::default()`.

### Tipos de ICMS suportados

| Variante | Regime | Construtor |
|---|---|---|
| `Icms00` | CST 00 — Tributada integralmente (CRT 3) | `Icms::icms00(orig, mod_bc, v_bc, p_icms, v_icms)` |
| `Icms40` | CST 40/41/50 — Isenta/NT/Suspensa (CRT 3) | `Icms::icms40(orig, cst)` |
| `Icms60` | CST 60 — ST cobrada anteriormente (CRT 3) | `Icms::icms60(orig)` |
| `Icms90` | CST 90 — Outros (CRT 3) | `Icms::icms90(orig)` |
| `Sn101` | CSOSN 101 — Com crédito (CRT 1) | `Icms::sn101(orig, p_cred_sn, v_cred_icmssn)` |
| `Sn102` | CSOSN 102/103/300/400 — Sem crédito (CRT 1) | `Icms::sn102(orig, csosn)` |
| `Sn500` | CSOSN 500 — ST anterior (CRT 1) | `Icms::sn500(orig)` |
| `Sn900` | CSOSN 900 — Outros (CRT 1) | `Icms::sn900(orig)` |

### Resposta da emissão

```rust
pub struct Response {
    pub protocolo: TagInfProt,  // n_prot, c_stat, x_motivo, dh_recbto
    pub xml: String,            // XML nfeProc autorizado — salvar em disco
}
```

---

## CancelarBuilder — Cancelamento

```rust
use dfe::CancelarBuilder;

let r = CancelarBuilder::new()
    .cert("./cert.pfx", "senha")
    .tp_amb(2)                          // 1 = Produção | 2 = Homologação
    .chave("35241211111111111111550010000000361491395167")
    .protocolo("135190000000000")
    .justificativa("Nota emitida com erro de valor")  // mín. 15 chars
    .mod_(55)                           // opcional — padrão 55; 65 para NFC-e
    .send()
    .await?;

println!("cStat: {}",   r.response.c_stat);    // "135" = evento registrado
println!("xMotivo: {}", r.response.x_motivo);
// r.send_xml    → XML enviado
// r.receive_xml → XML de resposta
```

| Método | Obrigatório | Descrição |
|---|:---:|---|
| `.cert(path, pass)` | ✅ | Certificado `.pfx` |
| `.tp_amb(u8)` | ✅ | Ambiente (1 = Produção, 2 = Homologação) |
| `.chave(str)` | ✅ | Chave de acesso de 44 dígitos |
| `.protocolo(str)` | ✅ | Protocolo de autorização da NF-e |
| `.justificativa(str)` | ✅ | Mínimo 15 caracteres |
| `.mod_(u32)` | — | Modelo do documento (padrão: 55) |

---

## Manifestação do Destinatário

Eventos enviados ao **Ambiente Nacional (AN)**. O `Manifestacao` é compartilhado entre os quatro eventos.

```rust
use dfe::manifestacao::{
    ciencia_operacao,
    confirmacao_operacao,
    desconhecimento_operacao,
    operacao_nao_realizada,
};
use dfe::tipos::manifestacao::{Manifestacao, OperacaoNaoRealizada};

let params = Manifestacao {
    cert_path:  "./cert.pfx".to_string(),
    cert_pass:  "senha".to_string(),
    cnpj:       "11111111111111".to_string(),
    tp_amb:     2,
    mod_:       None,   // None = 55; Some(65) para NFC-e
    chave:      "35241211111111111111550010000000361491395167".to_string(),
};

// 210210 — Ciência da Operação
let r = ciencia_operacao(params.clone()).await?;

// 210200 — Confirmação da Operação
let r = confirmacao_operacao(params.clone()).await?;

// 210220 — Desconhecimento da Operação
let r = desconhecimento_operacao(params.clone()).await?;

// 210240 — Operação Não Realizada (requer justificativa mín. 15 chars)
let r = operacao_nao_realizada(OperacaoNaoRealizada {
    justificativa: "Mercadoria não recebida".to_string(),
    ..params.into()
}).await?;

println!("cStat: {}",   r.response.c_stat);
println!("xMotivo: {}", r.response.x_motivo);
```

---

## Distribuição de DF-e

Consulta documentos fiscais de interesse do CNPJ no Ambiente Nacional.

```rust
use dfe::distribuicao::{
    Distribuicao,               // últimos documentos a partir do NSU atual
    DistribuicaoNSU,            // a partir de um NSU específico
    DistribuicaoChaveAcesso,    // por chave de acesso
};

// Últimos documentos
let r = Distribuicao::new()
    .cert_path("./cert.pfx")
    .cert_pass("senha")
    .cnpj("11111111111111")
    .uf(35)       // código IBGE da UF (SP = 35)
    .ambiente(2)
    .send()
    .await?;

println!("ultNSU: {} | maxNSU: {}", r.ult_nsu, r.max_nsu);

if let Some(docs) = r.lote_dist_dfe_int {
    for doc in docs {
        println!("NSU: {} | Schema: {}", doc.nsu, doc.schema);
        if let Some(nfe) = doc.content {
            println!("  chNFe: {} | vNF: {}", nfe.ch_nfe, nfe.v_nf);
        }
    }
}

// A partir de um NSU específico
let r = DistribuicaoNSU::new()
    .cert_path("./cert.pfx").cert_pass("senha")
    .cnpj("11111111111111").uf(35).ambiente(2)
    .nsu("000000000000100")
    .send().await?;

// Por chave de acesso
let r = DistribuicaoChaveAcesso::new()
    .cert_path("./cert.pfx").cert_pass("senha")
    .cnpj("11111111111111").uf(35).ambiente(2)
    .chave_acesso("35241211111111111111550010000000361491395167")
    .send().await?;
```

---

## NFeService — Status do Webservice

```rust
use dfe::NFeService;

let r = NFeService::new()
    .cert_path("./cert.pfx")
    .cert_pass("senha")
    .uf("SP")
    .environment(2)   // 1 = Produção | 2 = Homologação
    .send()
    .await?;

println!("cStat: {}",   r.c_stat);    // "107" = Serviço em operação
println!("xMotivo: {}", r.x_motivo);
println!("URL: {}",     r.url);
```

---

## DanfeBuilder — Geração de DANFE

Gera o DANFE em PDF a partir do XML autorizado (`nfeProc`). O modelo do documento (55 ou 65) é detectado automaticamente do campo `<mod>` no XML.

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

### Métodos do DanfeBuilder

| Método | Obrigatório | Descrição |
|---|:---:|---|
| `.xml(src)` | ✅ | Caminho `.xml` ou string do `nfeProc` |
| `.paper_size(str)` | — | `"a4"` (padrão), `"80mm"` ou `"54mm"` |
| `.as_file(path)` | ✅¹ | Salva o PDF em disco; retorna o caminho |
| `.as_base64()` | ✅¹ | Retorna o PDF como string base64 |
| `.logo(src)` | — | Logotipo do emitente — apenas A4 |
| `.qr_side()` | — | Layout QR lateral — apenas NFC-e 80mm |

¹ Use `.as_file()` **ou** `.as_base64()` — nunca os dois.

### Formatos implementados

| Tamanho | Modelo 55 (NF-e) | Modelo 65 (NFC-e) |
|---|:---:|:---:|
| `"a4"` | ✅ (suporta `.logo()`) | ❌ |
| `"80mm"` | ✅ | ✅ (suporta `.qr_side()`) |
| `"54mm"` | ❌ | ❌ |

### Logotipo do emitente — `.logo(src)`

O logo é renderizado no topo da coluna do emitente, centralizado horizontalmente, com altura máxima de 18mm. A proporção é sempre mantida; a imagem nunca é ampliada além do tamanho original.

| Formato de entrada | Exemplo |
|---|---|
| Caminho de arquivo | `"./logo.png"` / `"./logo.jpg"` |
| Base64 puro | `"iVBORw0KGgo..."` |
| Data URI | `"data:image/png;base64,iVBORw0KGgo..."` |

---

## Testes

```bash
cargo test
```

Os testes estão em `src/bin/tests/` e cobrem:

| Suite | Arquivo | Descrição |
|---|---|---|
| DANFE NF-e A4 | `test_danfe_nfe_a4.rs` | Geração A4 modelo 55 (base64, arquivo, erro modelo 65) |
| DANFE NFC-e 80mm | `test_danfe_nfce.rs` | Geração 80mm modelo 65 (multi-pagamento, CPF, qr_side, erros) |
| XML Extractor | `test_xml_extractor.rs` | Parsing de `nfeProc` a partir de string e arquivo |
| Integração | `mod.rs` | Status SEFAZ, emissão NF-e/NFC-e, cancelamento, DANFE¹ |

¹ Os testes de integração (`test_emit_nfe`, `test_emit_nfce`, `test_cancelar_nfe`, `test_service_status`) requerem certificado `.pfx` válido e conectividade com a SEFAZ em homologação.

Os testes de DANFE e XML não requerem certificado nem conexão.

---

## Notas importantes

- Sempre teste em **homologação** (`tp_amb: 2`) antes de produção.
- O certificado `.pfx` é lido do disco a cada operação — nunca cacheado em memória.
- Os webservices cobertos são da **SEFAZ/SP** e do **Ambiente Nacional**. Para outras UFs, contribua adicionando URLs em `interno/ws.rs`.
- Em `tp_amb = 2`, o campo `x_prod` do **primeiro item** é substituído automaticamente por `"NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL"` (exigência SEFAZ).

---

## Roadmap

### Regras tributárias — implementações pendentes

Atualmente o `NFeBuilder` suporta os principais grupos de ICMS, PIS e COFINS. Os itens abaixo estão planejados para as próximas versões:

| Item | Status | Descrição |
|---|:---:|---|
| `ICMS10` — ST parcial | 🔜 | ICMS próprio + Substituição Tributária com MVA |
| `ICMS20` — Redução de BC | 🔜 | Base de cálculo reduzida (CRT 3) |
| `ICMS30` — Isenta + ST | 🔜 | Isenta/NT para o emitente com ST para o destinatário |
| `ICMS51` — Diferimento | 🔜 | Diferimento total ou parcial |
| `ICMS70` — BC reduzida + ST | 🔜 | Redução de BC combinada com Substituição Tributária |
| `ICMS90` completo | 🔜 | Suporte a todos os campos opcionais (FCP, DIFAL) |
| `Sn900` completo | 🔜 | CSOSN 900 com todos os campos opcionais |
| **DIFAL / ICMS interestadual** | 🔜 | `v_icms_uf_dest`, `v_icms_uf_remet`, `v_fcpuf_dest` |
| **IPI** por item | 🔜 | Campos `vIPI`, `pIPI`, `cEnq`, `cSelo` no `Det` |
| **PIS/COFINS ST** | 🔜 | CST 05 — ST retida na cadeia |
| **IBS / CBS** | 🔜 | Reforma tributária — campos opcionais já mapeados |
| Validação de CNPJ/CPF | 🔜 | Verificação de dígito verificador antes da transmissão |
| Contingência (EPEC / FS-DA) | 🔜 | Emissão offline com posterior transmissão |

---

### ESC/POS — Suporte a impressoras térmicas

Implementação planejada de um builder fluente para geração de comandos **ESC/POS** (Epson Standard Code for Printers), padrão universal de impressoras térmicas de cupom.

#### API planejada — layout personalizado

```rust
use dfe::escpos::EscPosBuilder;

// Layout customizável com comandos ESC/POS
let bytes = EscPosBuilder::new()
    .paper_width(80)               // 80mm ou 58mm
    .align_center()
    .bold(true)
    .text("EMPRESA LTDA\n")
    .bold(false)
    .align_left()
    .text("CNPJ: 11.222.333/0001-81\n")
    .divider()                     // linha separadora "--------------------------------"
    .text(format!("{:<20} {:>10}\n", "PRODUTO EXEMPLO", "R$  50,00"))
    .divider()
    .align_right()
    .bold(true)
    .text("TOTAL  R$  50,00\n")
    .bold(false)
    .cut()                         // corte do papel
    .build();                      // → Vec<u8> para enviar à porta serial/USB

// Enviar para impressora (porta serial ou USB)
std::fs::write("/dev/usb/lp0", &bytes)?;          // Linux
std::fs::write("\\\\.\\COM3", &bytes)?;            // Windows
```

#### Comandos planejados

| Método | Descrição |
|---|---|
| `.paper_width(mm)` | Define largura do papel (80mm ou 58mm) |
| `.align_left()` / `.align_center()` / `.align_right()` | Alinhamento do texto |
| `.bold(bool)` | Negrito |
| `.underline(bool)` | Sublinhado |
| `.font_size(u8)` | Tamanho da fonte (1 = normal, 2 = duplo) |
| `.text(str)` | Insere texto |
| `.divider()` | Linha separadora proporcional à largura do papel |
| `.barcode_128(str)` | Code 128 nativo ESC/POS |
| `.qr_code(str, size)` | QR Code nativo ESC/POS |
| `.image(bytes)` | Imagem rasterizada (logo) |
| `.feed(n)` | Avança `n` linhas |
| `.cut()` | Corte total do papel |
| `.partial_cut()` | Corte parcial |
| `.build()` | Retorna `Vec<u8>` com os comandos ESC/POS |

---

### ESC/POS — Impressão de NFC-e (modelo 65)

Impressão direta de cupom fiscal NFC-e em impressoras térmicas, sem geração de PDF.

#### 80mm — planejado

```rust
use dfe::escpos::NfceEscPos;

let bytes = NfceEscPos::new()
    .xml("<nfeProc>...</nfeProc>")   // XML autorizado
    .paper_width(80)
    .logo("./logo.png")              // opcional
    .build()
    .await?;                         // → Vec<u8> prontos para envio

std::fs::write("\\\\.\\COM3", &bytes)?;
```

#### 58mm — planejado

```rust
let bytes = NfceEscPos::new()
    .xml("<nfeProc>...</nfeProc>")
    .paper_width(58)
    .build()
    .await?;
```

#### Layout planejado do cupom NFC-e ESC/POS

```
         NOME DO EMITENTE
  CNPJ: XX.XXX.XXX/XXXX-XX  IE: XXXXXXXX
  End.: Rua Exemplo, 100 - Centro - SP
─────────────────────────────────────────
CUPOM FISCAL ELETRÔNICO - SAT/NFC-e
─────────────────────────────────────────
 #  DESCRIÇÃO            QTD   VL UNIT  TOTAL
 1  PRODUTO EXEMPLO      2,00    50,00 100,00
─────────────────────────────────────────
                    TOTAL R$      100,00
DINHEIRO                          100,00
─────────────────────────────────────────
CHAVE: 3524 0509 4914 3500 ...
[BARCODE 128]
[QR CODE]
─────────────────────────────────────────
Protocolo: 135262060475222 27/05/2026
Consulte em portalnfce.fazenda.sp.gov.br
```

| Formato | Status |
|---|:---:|
| NFC-e ESC/POS 80mm | 🔜 planejado |
| NFC-e ESC/POS 58mm | 🔜 planejado |
| Layout customizável (`EscPosBuilder`) | 🔜 planejado |

---

## Licença

MIT — veja [LICENSE](LICENSE).
