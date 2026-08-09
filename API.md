# dfe — API pública (uso)

Exemplos de uso dos builders da crate `dfe`. Referenciado pelo `dfe/CLAUDE.md`. Para
arquitetura, gotchas técnicos e convenções internas, ver o `CLAUDE.md`.

Re-exports de conveniência em `lib.rs`:
```rust
use dfe::{NFeBuilder, CancelarBuilder, SubstituicaoBuilder, CartaCorrecaoBuilder, DanfeBuilder, NFeService, DfeError};
```

## Features de compilação

Opt-out: por padrão tudo está ligado, então adicionar a crate sem configurar features dá acesso a
toda a API (comportamento recomendado para o consumidor completo).

| Feature | Liga | Deps extras |
|---|---|---|
| `danfe` (default) | DANFE em PDF (`DanfeBuilder`) | `printpdf`, `image`, `barcoders`, `qrcodegen` |
| `escpos` (default) | Impressão ESC/POS (`EscPosBuilder`, `EscPosNFCeBuilder`) | `font8x8`, `image`, `barcoders`, `qrcodegen` |
| `distribuicao` (default) | Distribuição do AN + manifestação (`ManifestacaoBuilder`) | `flate2` |

O **core** (emissão, cancelamento, substituição, CC-e, status, extração de XML, tipos) está **sempre
disponível**, independente de features. Para um build enxuto, desligue o que não usar:

```toml
# Só emissão/cancelamento + DANFE (sem ESC/POS nem distribuição):
dfe = { version = "…", default-features = false, features = ["danfe"] }
```

---

## Emissão de NF-e / NFC-e — `NFeBuilder`

```rust
use dfe::NFeBuilder;
use dfe::tipos::{Icms, Pis, Cofins, Det, Emit, Ide, Pag, Total, Transp};
use dfe::tipos::emissao::Dest;

let itens = vec![
    Det {
        c_prod: "001".to_string(),
        x_prod: "PRODUTO".to_string(),   // sobrescrito em homologação (ver abaixo)
        ncm: "22030000".to_string(),
        cfop: 5102,
        u_com: "UN".to_string(), q_com: 1.0, v_un_com: 10.0, v_prod: 10.0,
        u_trib: "UN".to_string(), q_trib: 1.0, v_un_trib: 10.0,
        icms: Icms::icms00(0, 3, 10.0, 12.0, 1.20),  // ou struct literal
        pis: Pis::Aliq { cst: "01".to_string(), v_bc: 10.0, p_pis: 0.65, v_pis: 0.07 },
        cofins: Cofins::Aliq { cst: "01".to_string(), v_bc: 10.0, p_cofins: 3.0, v_cofins: 0.30 },
        ..Default::default()
    },
];

let resposta = NFeBuilder::new()
    .cert("caminho.pfx", "senha")
    .ide(Ide { c_uf: 35, mod_: 55, serie: 1, n_nf: 1, tp_amb: 2, ..Default::default() })
    .emitente(emit)
    .destinatario(dest)              // opcional
    .itens(itens)                    // Vec<Det> — totais calculados automaticamente
    .total(Total::default())         // informar apenas frete, seguro, ST, FCP, etc.
    .transporte(transp)
    .pagamento(pag)
    .informacoes_adicionais(inf_adic) // opcional
    .id_csc("000001")                // NFC-e apenas — ID do CSC
    .csc("CODIGO_CSC")               // NFC-e apenas — código CSC
    .desconto_rateio(valor)          // opcional — desconto rateado nos itens
    .emitir()
    .await?;

// resposta.protocolo.inf_prot.c_stat   → "100" = autorizada
// resposta.protocolo.inf_prot.x_motivo
// resposta.xml                          → XML do nfeProc autorizado
// resposta.send_xml                     → envelope SOAP enviado à SEFAZ (debug/auditoria)
// resposta.receive_xml                  → corpo cru da resposta da SEFAZ (debug/auditoria)
```

**NFC-e** (modelo 65): mesma API, com `mod_: 65`, `tp_imp: 4`, `.id_csc()` e `.csc()` obrigatórios. `dh_sai_ent` e QR Code tratados automaticamente.

### `Total` — campos informados pelo usuário

`v_bc`, `v_icms`, `v_prod`, `v_pis`, `v_cofins`, `v_nf`, `v_tot_trib`, `v_icms_deson` e `v_desc` são **calculados automaticamente** dos itens em `total_process`. Informar apenas:

| Campo | Descrição |
|---|---|
| `v_frete`, `v_seg`, `v_outro` | Despesas da NF (globais, não por item) |
| `v_ii`, `v_ipi`, `v_ipi_devol` | Impostos específicos |
| `v_bc_st`, `v_st` | ST (ICMS10/ICMS70 — não implementados ainda) |
| `v_fcp`, `v_fcpst`, `v_fcpst_ret` | Fundo de Combate à Pobreza |
| `v_fcpuf_dest`, `v_icms_uf_dest`, `v_icms_uf_remet` | Diferencial de alíquota UF destino |

Para uma venda simples sem frete/seguro: `Total::default()`.

---

## Enum `Icms` — variantes e construtores

| Variante | CST/CSOSN | Regime | Construtor |
|---|---|---|---|
| `Icms00 { orig, mod_bc, v_bc, p_icms, v_icms }` | 00 | Normal CRT=3 | `Icms::icms00(orig, mod_bc, v_bc, p_icms, v_icms)` |
| `Icms40 { orig, cst, v_icms_deson, mot_des_icms }` | 40/41/50 | Normal CRT=3 | `Icms::icms40(orig, cst)` |
| `Icms60 { orig, v_bcst_ret, p_st, v_icms_substituto, v_icmsst_ret }` | 60 | Normal CRT=3 | `Icms::icms60(orig)` |
| `Icms90 { orig }` | 90 | Normal CRT=3 | `Icms::icms90(orig)` |
| `Sn101 { orig, p_cred_sn, v_cred_icmssn }` | CSOSN 101 | Simples CRT=1 | `Icms::sn101(orig, p_cred_sn, v_cred_icmssn)` |
| `Sn102 { orig, csosn }` | CSOSN 102/103/300/400 | Simples CRT=1 | `Icms::sn102(orig, csosn)` |
| `Sn500 { orig, v_bcst_ret, v_icmsst_ret }` | CSOSN 500 | Simples CRT=1 | `Icms::sn500(orig)` |
| `Sn900 { orig, mod_bc, v_bc, … }` | CSOSN 900 | Simples CRT=1 | `Icms::sn900(orig)` |

Os construtores preenchem os campos obrigatórios e definem todos os `Option` como `None`. Para campos opcionais preenchidos (ex: ST em `Sn500`), usar o struct literal diretamente:
```rust
Icms::Sn500 { orig: 0, v_bcst_ret: Some(100.0), v_icmsst_ret: Some(12.0) }
```

## Enum `Pis` / `Cofins`

```rust
Pis::Aliq { cst, v_bc, p_pis, v_pis }            // CST 01/02 — alíquota
Pis::Outr                                          // CST 99 — outros (zeros automáticos)
Pis::Nt { cst }                                    // CST 04-09 — não tributado
Pis::Qtde { cst, q_bc_prod, v_aliq_prod, v_pis }  // CST 03 — por quantidade

Cofins::Aliq { cst, v_bc, p_cofins, v_cofins }
Cofins::Outr { cst }
Cofins::Nt { cst }
Cofins::Qtde { cst, q_bc_prod, v_aliq_prod, v_cofins }
```

---

## Cancelamento — `CancelarBuilder`

```rust
use dfe::CancelarBuilder;

let r = CancelarBuilder::new()
    .cert("caminho.pfx", "senha")
    .tp_amb(2)                   // 1 = Produção | 2 = Homologação
    .chave("35...")              // chave de acesso de 44 dígitos
    .protocolo("135...")         // protocolo de autorização
    .justificativa("Motivo do cancelamento aqui")  // mínimo 15 caracteres
    .mod_(55)                    // opcional — padrão 55; 65 para NFC-e
    .send()
    .await?;

// r.response.c_stat   → "135" = evento registrado
// r.response.x_motivo
// r.send_xml / r.receive_xml
```

---

## Cancelamento por substituição (NFC-e) — `SubstituicaoBuilder`

Evento `tpEvento` **110112** (NT 2018.004). **Só NFC-e (modelo 65)**, prazo 168h. Usado quando
duas NFC-e representam a mesma venda (ex.: uma normal + uma de contingência offline): cancela a
**duplicada** e mantém a **substituta** válida.

```rust
use dfe::SubstituicaoBuilder;

let r = SubstituicaoBuilder::new()
    .cert("caminho.pfx", "senha")
    .tp_amb(2)
    .chave("35...")             // NFC-e CANCELADA (44 dígitos, modelo 65)
    .protocolo("135...")        // protocolo da NFC-e cancelada
    .chave_substituta("35...")  // NFC-e que permanece VÁLIDA (chNFeRef)
    .ver_aplic("MeuPDV-1.0")    // nome/versão do software emissor (obrigatório)
    .justificativa("Falha na conexao com a internet no momento da venda")  // 15–255 caracteres
    .send()
    .await?;

// Retorna dfe::tipos::cancelar::Response (mesmo formato do cancelamento).
// r.response.c_stat / r.response.x_motivo / r.send_xml / r.receive_xml
```

⚠️ `chave`/`protocolo` = a NFC-e **cancelada**; `chave_substituta` = a que **fica válida**. Trocar
isso cancela a nota errada. O endpoint é resolvido pela UF real da chave (as 27 UFs).

---

## Carta de Correção (CC-e) — `CartaCorrecaoBuilder`

Evento `tpEvento` **110110**. Corrige dados **não** ligados a valores/impostos/destinatário. Vale
para NF-e e NFC-e. O `xCondUso` (texto legal fixo) é embutido; o `nSeqEvento` é **acumulativo**.

```rust
use dfe::CartaCorrecaoBuilder;

let r = CartaCorrecaoBuilder::new()
    .cert("caminho.pfx", "senha")
    .tp_amb(2)
    .chave("35...")                 // NF-e/NFC-e a corrigir (44 dígitos)
    .correcao("Onde se le X, leia-se Y no campo de observacoes")  // 15–1000 caracteres
    .n_seq_evento(1)                // opcional (padrão 1); incremente a cada nova CC-e da chave
    .send()
    .await?;

// Retorna dfe::tipos::cancelar::Response.
// r.response.c_stat / r.response.x_motivo / r.send_xml / r.receive_xml
```

`nSeqEvento` acumulativo: cada CC-e deve conter **todas** as correções anteriores (a última
substitui as demais). Quebras de linha no `correcao` são normalizadas para espaço (single-line).

---

## Manifestação do destinatário

```rust
use dfe::ManifestacaoBuilder;

// Um método terminal por tipo de evento; todos partem do mesmo builder.
let base = || ManifestacaoBuilder::new()
    .cert("caminho.pfx", "senha")
    .cnpj("11111111111111")
    .tp_amb(2)
    .chave("35...");

base().ciencia_operacao().await?;          // 210210
base().confirmacao_operacao().await?;      // 210200
base().desconhecimento_operacao().await?;  // 210220

base().operacao_nao_realizada("Motivo...").await?;  // 210240 (justificativa obrigatória)
```

Retorna `dfe::tipos::manifestacao::Response` (`.response` = `InfEvento`, `.send_xml`, `.receive_xml`).

Tipos de evento: `210210` Ciência · `210200` Confirmação · `210220` Desconhecimento · `210240` Operação Não Realizada (requer justificativa).

---

## Status do serviço SEFAZ

```rust
use dfe::NFeService;

let r = NFeService::new()
    .cert_path("caminho.pfx")
    .cert_pass("senha")
    .uf("SP")
    .environment(2)
    .send()
    .await?;
// r.c_stat, r.x_motivo, r.url
```

---

## Consulta de situação da NF-e/NFC-e — `ConsultaSituacaoBuilder`

`consSitNFe` — consulta **não assinada** (só mTLS, como o status do serviço) que devolve a
situação atual de uma nota pela chave de acesso: se está autorizada, se não consta na base da
SEFAZ, e eventos já registrados contra ela (ex.: cancelamento).

```rust
use dfe::ConsultaSituacaoBuilder;

let r = ConsultaSituacaoBuilder::new()
    .cert("caminho.pfx", "senha")
    .tp_amb(2)
    .chave("35...") // 44 dígitos
    .send()
    .await?;

r.response.c_stat;       // status da CONSULTA (ex.: "217" = não consta na base)
r.response.prot_nfe;     // Option<ProtNFe> — presente só se a nota consta (autorizada/cancelada)
r.response.proc_evento_nfe; // Vec<ProcEventoNFe> — eventos já registrados (ex.: cancelamento 110111)
```

Se `prot_nfe` estiver presente, `prot_nfe.inf_prot.c_stat` é o status da **nota** (`100` =
autorizada, `101` = cancelada) e `.n_prot`/`.dh_recbto` trazem o protocolo/data de autorização.

Uso previsto: recuperação de emissão órfã (app fechou/timeout antes da resposta chegar) —
consultar a chave 1x, respeitando o rate-limit da SEFAZ (10 consultas/hora por chave, NT
2014.002), para decidir se a nota foi autorizada antes de reenviar ou reemitir.

---

## Distribuição (Ambiente Nacional)

```rust
use dfe::distribuicao::Distribuicao;

let r = Distribuicao::new()
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

A resposta pode conter `docZip` — base64 + GZIP, decodificado automaticamente.

---

## DANFE (PDF)

```rust
use dfe::DanfeBuilder;

// NF-e A4 — salvar em arquivo
let caminho = DanfeBuilder::new()
    .xml(xml_str)          // string XML do nfeProc ou caminho de arquivo (termina em ".xml")
    .paper_size("a4")      // padrão quando omitido
    .as_file("nota.pdf")
    .build()
    .await?;               // Ok(String) = caminho do arquivo gerado

// NF-e A4 — com logotipo do emitente
let b64 = DanfeBuilder::new()
    .xml(xml_str)
    .paper_size("a4")
    .logo("caminho/logo.png")   // caminho .png/.jpg, base64 puro ou data URI
    .as_base64()
    .build()
    .await?;

// NF-e 80mm — retornar como base64
let b64 = DanfeBuilder::new()
    .xml(xml_str)
    .paper_size("80mm")
    .as_base64()
    .build()
    .await?;               // Ok(String) = string base64 do PDF

// NFC-e 80mm — QR Code centralizado (padrão)
let b64 = DanfeBuilder::new()
    .xml(xml_str)
    .paper_size("80mm")    // modelo 65 detectado automaticamente pelo XML
    .as_base64()
    .build()
    .await?;

// NFC-e 80mm — QR Code lateral (à esquerda, ~33mm; chave e protocolo à direita)
let b64 = DanfeBuilder::new()
    .xml(xml_str)
    .paper_size("80mm")
    .qr_side()             // layout lateral — só faz efeito em NFC-e 80mm
    .as_base64()
    .build()
    .await?;
```

Retorno: `Result<String, String>` — `Ok` contém o caminho do arquivo (`.as_file`) ou a string base64 (`.as_base64`).

| Formato | Modelo 55 (NF-e) | Modelo 65 (NFC-e) |
|---|---|---|
| `"a4"` | ✅ (suporta `.logo()`) | ❌ não implementado |
| `"80mm"` | ✅ | ✅ (suporta `.qr_side()`) |
| `"54mm"` | ❌ não implementado | ❌ não implementado |

O modelo é detectado automaticamente do campo `<mod>` no XML — não é necessário informar.

### Logotipo do emitente — `.logo(src)` (apenas A4)

| Formato de entrada | Exemplo |
|---|---|
| Caminho de arquivo | `"logo.png"` / `"logo.jpg"` |
| Base64 puro | `"iVBORw0KGgo..."` |
| Data URI | `"data:image/png;base64,iVBORw0KGgo..."` |

O logo é renderizado no topo da coluna do emitente, centralizado horizontalmente, com altura máxima de 18mm. A proporção original é sempre mantida; a imagem nunca é ampliada além do tamanho original. Formatos suportados: PNG e JPEG.
