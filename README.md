# dfe

**Documentos Fiscais Eletrônicos Brasileiros** — crate Rust para integração com os webservices da SEFAZ.

[![Crates.io](https://img.shields.io/crates/v/dfe)](https://crates.io/crates/dfe)
[![Docs.rs](https://docs.rs/dfe/badge.svg)](https://docs.rs/dfe)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Sponsor GustavoOta](https://img.shields.io/badge/sponsor-GustavoOta-%23EA4AAA?style=flat&logo=github)](https://github.com/sponsors/GustavoOta)

---

## Índice

- [Instalação](#instalação)
- [Funcionalidades](#funcionalidades)
- [Emissão NF-e / NFC-e](#emissão-nf-e--nfc-e)
- [Cancelamento](#cancelamento)
- [Manifestação do Destinatário](#manifestação-do-destinatário)
- [Distribuição de DF-e](#distribuição-de-df-e)
- [DANFE — Geração de PDF](#danfe--geração-de-pdf)
- [ESC/POS — Impressoras Térmicas](#escpos--impressoras-térmicas)
- [Status do Webservice](#status-do-webservice)
- [Tratamento de Erros](#tratamento-de-erros)
- [Tipos de ICMS suportados](#tipos-de-icms-suportados)
- [Validação de CNPJ / CPF](#validação-de-cnpj--cpf)
- [Testes](#testes)
- [Notas importantes](#notas-importantes)
- [Roadmap](#roadmap)
- [Licença](#licença)

---

## Instalação

```toml
[dependencies]
dfe = "0.5.7"
```

> **Schemas XSD** (`PL_010b_NT2025_002_v1.21`) estão embutidos no binário via `include_bytes!`.

### OpenSSL — compilação a partir do código-fonte (`vendored`)

Esta crate depende do `openssl` com `features = ["vendored"]`. Isso significa que o OpenSSL **não precisa estar instalado no sistema** — ele é baixado e compilado automaticamente durante o `cargo build`. A desvantagem é que a primeira compilação leva alguns minutos a mais.

Porém, o sistema de build do OpenSSL é escrito em Perl, então **o Perl deve estar instalado e acessível no `PATH`** antes de compilar. Sem ele, o build falhará com um erro parecido com:

```
Could not find perl
```

#### Windows

Instale o [Strawberry Perl](https://strawberryperl.com) (recomendado — inclui compilador C e ferramentas Unix necessárias):

1. Baixe o instalador `.msi` em <https://strawberryperl.com>
2. Execute o instalador (o PATH é atualizado automaticamente)
3. Abra um novo terminal e confirme: `perl -v`
4. Execute `cargo build` normalmente

> O Strawberry Perl já vem com `dmake` e `gcc`, o que evita dependência do MSVC ou do Visual Studio Build Tools.

#### Linux

O Perl geralmente já está disponível. Verifique:

```bash
perl -v
```

Se não estiver instalado:

```bash
# Debian/Ubuntu
sudo apt-get install perl

# Fedora/RHEL
sudo dnf install perl
```

#### macOS

O Perl já vem pré-instalado com o macOS. Nenhuma ação necessária.

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

## Emissão NF-e / NFC-e

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
        x_nome: "EMPRESA LTDA".to_string(),
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
        icms: Icms::sn102(0, "400"),      // CSOSN 400
        pis: Pis::Nt { cst: "07".to_string() },
        cofins: Cofins::Nt { cst: "07".to_string() },
        ..Default::default()
    }])
    .total(Total::default())         // totais calculados automaticamente dos itens
    .transporte(Transp { mod_frete: 9, ..Default::default() })
    .pagamento(Pag { ..Default::default() })
    .informacoes_adicionais(InfAdic {
        inf_cpl: Some("Pedido 123".to_string()),
        ..Default::default()
    })
    // NFC-e: obrigatório também .id_csc() e .csc()
    .emitir()
    .await?;

println!("cStat:     {}", resposta.protocolo.inf_prot.c_stat);
println!("xMotivo:   {}", resposta.protocolo.inf_prot.x_motivo);
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
| `v_bc_st`, `v_st` | ST global (itens com ICMS10/30/70 auto-somam) |
| `v_fcp`, `v_fcpst`, `v_fcpst_ret` | Fundo de Combate à Pobreza |
| `v_fcpuf_dest`, `v_icms_uf_dest`, `v_icms_uf_remet` | DIFAL |

Para uma venda simples sem extras: `Total::default()`.

---

## Cancelamento

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

## DANFE — Geração de PDF

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

---

## ESC/POS — Impressoras Térmicas

### EscPosBuilder — layout personalizado

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

### EscPosNFCeBuilder — impressão de NFC-e

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

---

## Status do Webservice

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

## Tratamento de Erros

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

## Tipos de ICMS suportados

| Variante | CST/CSOSN | Regime | Construtor |
|---|---|---|---|
| `Icms00` | 00 | Normal CRT=3 | `Icms::icms00(orig, mod_bc, v_bc, p_icms, v_icms)` |
| `Icms10` | 10 | Normal CRT=3 | `Icms::icms10(orig, mod_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, v_bcst, p_icmsst, v_icmsst)` |
| `Icms20` | 20 | Normal CRT=3 | `Icms::icms20(orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms)` |
| `Icms30` | 30 | Normal CRT=3 | `Icms::icms30(orig, mod_bcst, p_mvast, v_bcst, p_icmsst, v_icmsst)` |
| `Icms40` | 40/41/50 | Normal CRT=3 | `Icms::icms40(orig, cst)` |
| `Icms51` | 51 | Normal CRT=3 | `Icms::icms51(orig)` + campos via struct literal |
| `Icms60` | 60 | Normal CRT=3 | `Icms::icms60(orig)` |
| `Icms70` | 70 | Normal CRT=3 | `Icms::icms70(orig, mod_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, v_bcst, p_icmsst, v_icmsst)` |
| `Icms90` | 90 | Normal CRT=3 | `Icms::icms90(orig)` + campos opcionais via struct literal |
| `Sn101` | CSOSN 101 | Simples CRT=1 | `Icms::sn101(orig, p_cred_sn, v_cred_icmssn)` |
| `Sn102` | CSOSN 102/103/300/400 | Simples CRT=1 | `Icms::sn102(orig, csosn)` |
| `Sn500` | CSOSN 500 | Simples CRT=1 | `Icms::sn500(orig)` |
| `Sn900` | CSOSN 900 | Simples CRT=1 | `Icms::sn900(orig)` + campos opcionais via struct literal |

### IPI por item

```rust
use dfe::tipos::Ipi;

// CST 50 — saída tributada por alíquota ad valorem
ipi: Some(Ipi::tributado("999", v_bc, p_ipi, v_ipi))

// CST 53 — saída não tributada
ipi: Some(Ipi::nao_tributado("999", "53"))
```

### PIS / COFINS

```rust
use dfe::tipos::{Pis, Cofins};

// CST 01/02 — alíquota
Pis::Aliq { cst, v_bc, p_pis, v_pis }

// CST 03 — por quantidade
Pis::Qtde { cst, q_bc_prod, v_aliq_prod, v_pis }

// CST 04-09 — não tributado
Pis::Nt { cst }

// CST 05 — substituição tributária
Pis::St { v_bc: Some(100.0), p_pis: Some(0.65), q_bc_prod: None, v_aliq_prod: None, v_pis: 0.65 }

// CST 99 — outros (zeros automáticos)
Pis::Outr
```

---

## Validação de CNPJ / CPF

```rust
use dfe::{validate_cnpj, validate_cpf};

assert!(validate_cnpj("11.222.333/0001-81"));
assert!(validate_cnpj("11222333000181"));

assert!(validate_cpf("529.982.247-25"));
assert!(validate_cpf("52998224725"));
```

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

¹ Os testes de integração requerem certificado `.pfx` válido e conectividade com a SEFAZ em homologação.

---

## Notas importantes

- Sempre teste em **homologação** (`tp_amb: 2`) antes de produção.
- O certificado `.pfx` é lido do disco a cada operação — nunca cacheado em memória.
- Em `tp_amb = 2`, o campo `x_prod` do **primeiro item** é substituído automaticamente por `"NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL"` (exigência SEFAZ).
- Os webservices cobertos são da **SEFAZ/SP** e do **Ambiente Nacional**. Para outras UFs, contribua adicionando URLs em `interno/ws.rs`.

---

## Roadmap

| Item | Status |
|---|:---:|
| `ICMS10/20/30/51/70` | ✅ |
| `ICMS90` / `Sn900` completos | ✅ |
| **DIFAL** (`v_icms_uf_dest`, `v_icms_uf_remet`) | ✅ |
| **IPI** por item (`Det.ipi`) | ✅ |
| **PIS/COFINS ST** (CST 05) | ✅ |
| **IBS / CBS** (reforma tributária) | ✅ |
| Validação de CNPJ/CPF | ✅ |
| ESC/POS `EscPosBuilder` | ✅ |
| ESC/POS `EscPosNFCeBuilder` | ✅ |
| Contingência (EPEC / FS-DA) | 🔜 |

---

## Licença

MIT — veja [LICENSE](LICENSE).
