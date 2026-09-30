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
| `escpos` (default) | Impressão ESC/POS (`EscPosBuilder`, `EscPosNFCeBuilder`, `EscPosDanfeNFeBuilder`) | `font8x8`, `image`, `barcoders`, `qrcodegen` |
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
    .desconto_rateio(valor)          // opcional — desconto rateado nos itens (det/prod/vDesc)
    .outro_rateio(valor)             // opcional — acréscimo rateado nos itens (det/prod/vOutro)
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

`v_bc`, `v_icms`, `v_prod`, `v_pis`, `v_cofins`, `v_nf`, `v_tot_trib`, `v_icms_deson`, `v_desc`,
`v_bc_st`, `v_st`, `v_fcp`, `v_fcpst`, `v_fcpst_ret` e os totais do monofásico (`qBCMono`…
`vICMSMonoRet`, só com item 02/15/53/61) são **calculados automaticamente** dos itens em
`total_process`. Cada parcela é o valor **impresso** no item (`arred2`), então o total é
exatamente a soma do que está no XML (rejeições 531/532/602/603). Informar apenas:

| Campo | Descrição |
|---|---|
| `v_frete`, `v_seg`, `v_outro` | Despesas da NF (globais, não por item) |
| `v_ii`, `v_ipi`, `v_ipi_devol` | Impostos específicos |
| `v_bc_st`, `v_st` | ST fora dos itens — **somados** ao que vem dos itens |
| `v_fcp`, `v_fcpst`, `v_fcpst_ret` | FCP fora dos itens — **somados** aos dos itens |
| `v_fcpuf_dest`, `v_icms_uf_dest`, `v_icms_uf_remet` | Diferencial de alíquota UF destino |

`vNF = vProd + vFrete + vSeg − vDesc + vOutro + vII + vIPI − vIPIDevol + vST + vFCPST − vICMSDeson
+ vPIS(PISST) + vCOFINS(COFINSST) + vICMSMonoReten`

- `vICMSDeson` só dos itens com `ind_deduz_deson = 1` (NT 2023.004);
- PISST/COFINSST só com `indSomaPISST`/`indSomaCOFINSST = 1` (NT 2020.005);
- `vICMSMonoReten` do CST 15 (NT 2023.001). Todos pela regra 610.

Para uma venda simples sem frete/seguro: `Total::default()`.

### Ajustes automáticos na montagem

- `xNome` e `xFant` do emitente são cortados nos **60 caracteres** do XSD (conta caracteres,
  não bytes; espaço no ponto de corte é removido).
- `vUnCom`/`vUnTrib` saem com **2 a 10 casas** e `qCom`/`qTrib` com **3 a 4** (zeros à direita
  cortados até o mínimo): 602,6097 não vira "602.61" (rejeição 629 com quantidade maior).
- Todo número do XML passa por `dfe::fmt_dec` (arredondamento meio para cima — ver
  [Arredondamento](#arredondamento--dfearred2)).

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

### `Icms::Parametros(IcmsParametros)` — caminho de código novo

O consumidor manda só o que o cadastro do produto guarda e a crate escolhe o grupo do
XSD e calcula os valores sobre a base do item (vProd + frete + acréscimo − desconto, já
rateados) e a quantidade tributável (`qTrib`). Cobre **todos os grupos de ICMS do leiaute**:
CST 00, 02, 10, 15, 20, 30, 40, 41, 50, 51, 53, 60, 61, 70, 90, ICMSPart, ICMSST e CSOSN 101,
102, 103, 201, 202, 203, 300, 400, 500, 900. Tabela de grupos × parâmetros, modalidades e
fórmulas: [`docs/icms-pis-cofins.md`](docs/icms-pis-cofins.md#icms).

```rust
Icms::Parametros(IcmsParametros {
    cst: "20".into(), orig: 0,
    p_icms: Some(18.0), p_red_bc: Some(33.33),
    mot_des_icms: Some(9), ind_deduz_deson: Some(0),
    ..Default::default()
})
```

- Campo que o grupo exige e veio vazio (ex.: CST 20 sem `p_red_bc`) é `DfeError::Validacao`
  na montagem, com o nome do campo — a nota não sai com o grupo errado.
- Todas as modalidades são calculadas: `mod_bc` 3 (valor da operação), 0 (MVA própria,
  `p_mva_proprio`), 1/2 (pauta / preço tabelado: `qTrib × v_pauta`); `mod_bcst` 4 (MVA), 6
  (valor da operação), 0/1/2/3/5 (`qTrib × v_pauta_st`). Modalidade sem o seu parâmetro é erro.
- `mot_des_icms` e `mot_des_icms_st` são conferidos contra a lista que o XSD aceita no grupo.
- ICMS-ST deduz o ICMS próprio; no Simples (201/202/203) deduz `base × p_icms` quando
  o cadastro informa a alíquota interna.
- CST 60 / CSOSN 500: os campos de ST retido são repassados como vieram; FCP-ST retido
  (`p_fcpst_ret`) e ICMS efetivo (`p_red_bc_efet`, `p_icms_efet`) a crate calcula.
- Grupos por parâmetro: `uf_st` num 10/20/90 → **ICMSPart**; `v_bcst_dest` num 41/60 →
  **ICMSST** (repasse; valores da operação).
- Monofásico (02/15/53/61): `qTrib × ad rem`; exige `Det::comb` com o código ANP.
- Totais: vFCP, vFCPST, vFCPSTRet, vBCST e vST saem da soma dos itens (ICMSPart incluído);
  fórmula do vNF em [`Total`](#total--campos-informados-pelo-usuário).

### `Det::c_benef` — código de benefício fiscal

`Some("SP070001")` sai como `<cBenef>` no `<prod>` (entre `CEST` e `CFOP`). O XSD aceita
8 ou 10 caracteres ou `SEM CBENEF`. Vazio/espaços é omitido. Algumas UFs exigem o código
para CST de benefício (20, 30, 40, 41, 50, 51, 70, 90) — sem ele, rejeição 930.

#### Campos de `IcmsParametros` (todos `Option`, `#[serde(default)]`)

| Campo | Uso |
|---|---|
| `mod_bc`, `p_red_bc`, `p_icms`, `p_fcp`, `p_dif` | ICMS próprio, redução, FCP, diferimento (51/90) |
| `p_mva_proprio`, `v_pauta` | `mod_bc` 0 e 1/2 |
| `mod_bcst`, `p_mvast`, `p_red_bcst`, `p_icmsst`, `p_fcpst`, `v_pauta_st` | ICMS-ST |
| `mot_des_icms`, `ind_deduz_deson` | desoneração |
| `mot_des_icms_st` | ST desonerado (10/70/90: 3, 9, 12) |
| `c_benef_rbc`, `p_fcp_dif` | cBenef da redução e FCP diferido (51/90) |
| `v_bcst_ret`, `p_st`, `v_icms_substituto`, `v_icmsst_ret` | ST retido (60/500/ICMSST) |
| `p_fcpst_ret`, `p_red_bc_efet`, `p_icms_efet` | FCP-ST retido e ICMS efetivo (60/500/ICMSST) |
| `uf_st`, `p_bc_op` | ICMSPart |
| `v_bcst_dest`, `v_icmsst_dest` | ICMSST (repasse) |
| `ad_rem_icms`, `ad_rem_icms_reten`, `p_red_ad_rem`, `mot_red_ad_rem`, `ad_rem_icms_ret` | monofásico 02/15/53/61 |
| `p_cred_sn` | crédito do Simples (101/201/900) |

### `Det::cred_presumido` e `Det::comb`

- `cred_presumido: Vec<CredPresumido { codigo, p }>` → `prod/gCred` (até 4; vCredPresumido =
  base do item × `p`).
- `comb: Option<Comb>` → `prod/comb` (código e descrição ANP, UF de consumo, % biodiesel,
  CIDE, encerrante, origem). Obrigatório nos CST monofásicos.

Exemplo em [`docs/icms-pis-cofins.md`](docs/icms-pis-cofins.md#dados-do-prod-c_benef-cred_presumido-comb).

## Enum `Pis` / `Cofins`

```rust
Pis::Aliq { cst, v_bc, p_pis, v_pis }            // CST 01/02 — alíquota
Pis::por_parametros(&PisCofinsParametros { cst, v_bc, aliquota, q_bc_prod, v_aliq_prod })
                                                   // QUALQUER CST: 01/02 Aliq, 03 Qtde, 04-09 NT,
                                                   // 49-99 Outr por % ou por R$/un. (ou zerado) — preferir
Pis::OutrAliq { cst, v_bc, p_pis, v_pis }          // 49-99 por alíquota
Pis::OutrQtde { cst, q_bc_prod, v_aliq_prod, v_pis } // 49-99 por quantidade
Pis::St { v_bc, p_pis, q_bc_prod, v_aliq_prod, v_pis, ind_soma } // CST 05 do substituto: PISNT 05 + PISST irmão
Pis::nao_tributado(cst)                            // sem valor: 04-09 → PISNT, 49-99 → PISOutr (preferir)
Pis::Outr                                          // CST 99 — outros (zeros automáticos)
Pis::Nt { cst }                                    // sem valor; o grupo sai pelo CST (04-09 NT, 49-99 Outr)
Pis::Qtde { cst, q_bc_prod, v_aliq_prod, v_pis }  // CST 03 — por quantidade

Cofins::Aliq { cst, v_bc, p_cofins, v_cofins }
Cofins::por_parametros(&p)                         // mesma regra do PIS
Cofins::nao_tributado(cst)                         // sem valor: 04-09 → COFINSNT, 49-99 → COFINSOutr (preferir)
Cofins::Outr { cst }                               // Outr e Nt: o grupo sai pelo CST, não pela variante
Cofins::Nt { cst }
Cofins::Qtde { cst, q_bc_prod, v_aliq_prod, v_cofins }
```

`q_bc_prod` sai com 4 casas (TDec_1204v): 3,5 kg de 0,1234 não vira 0.123.

## IBS/CBS — `Det::ibs_cbs: Option<IbsCbs>`

```rust
// IbsCbs não tem Default: todos os campos são informados.
IbsCbs {
    cst: "000".into(), class_trib: "000001".into(),
    p_ibs_uf: dec!(0.10), p_ibs_mun: dec!(0.00), p_cbs: dec!(0.90),
    v_bc: Decimal::ZERO, v_ibs_uf: Decimal::ZERO,     // ignorados com calcular = true
    v_ibs_mun: Decimal::ZERO, v_cbs: Decimal::ZERO,
    calcular: true,                     // a crate calcula vBC e valores
    p_red_ibs: None, p_red_cbs: None,   // pRedIBS / pRedCBS do cClassTrib
}
```

- **`calcular: true`** — a crate ignora os `v_*` recebidos e calcula
  `vBC = vProd + vFrete + vSeg + vOutro − vDesc − vPIS − vCOFINS − vICMS − vFCP` do item (já
  rateados, como impressos; NT 2025.002-RTC, rejeição 1115) e `vIBSUF`/`vIBSMun`/`vCBS = vBC ×
  alíquota`. **`false`** — valores prontos do chamador, como antes.
- **`p_red_ibs` / `p_red_cbs`** — monta `gRed` (`pRedAliq`, `pAliqEfet`) em `gIBSUF`, `gIBSMun`
  e `gCBS`, com `pAliqEfet = p × (1 − pRedAliq/100)` (4 casas) e valor `vBC × pAliqEfet`.
  Só vai ao XML quando `IbsCbs::cst_exige_reducao(cst)` (CST **200** e **515**, `ind_gRed = 1`):
  nesses é obrigatório (1033/1074); em outro CST a redução é ignorada (gRed lá é a 1032).
  Redução de 100% sai com `pAliqEfet 0.0000` e valores zerados.
- `IBSCBSTot` só é enviado se algum item tiver IBS/CBS (1118). `vCredPres` e
  `vCredPresCondSus` saem `0.00` (obrigatórios no XSD; vazio reprovava no pattern).

## Arredondamento — `dfe::arred2`

Regra única do ecossistema: **meio para cima, afastando do zero, sobre o valor decimal** (o
`round(..., 2, PHP_ROUND_HALF_UP)` do NFePHP). O f64 passa por 15 algarismos significativos
antes de virar `Decimal`, então `1.005` dá `1.01` — `format!("{:.2}")` e `(v*100).round()/100`
dão `1.00`. O JS tem o espelho `arredondamento.js` (PDV e retaguarda).

```rust
use dfe::{arred, arred2, arred_decimal, fmt_dec, fmt_dec_ate};

arred2(1.005)              // 1.01
arred(0.12345, 4)          // 0.1235
fmt_dec(0.125, 2)          // "0.13"   (texto do XML, casas fixas)
fmt_dec_ate(602.6097, 2, 10) // "602.6097" (mín. 2, máx. 10 casas)
arred_decimal(d, 2)        // Decimal
```

Regra de uso: campo derivado sai dos campos **já arredondados** que vão no XML (vBC do
IBS/CBS = vProd − vICMS impresso…), e todo total é a soma dos itens arredondados. O rateio de
desconto/acréscimo desempata para cima e o último item absorve a diferença.

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
    .modelo(65) // 65 = NFC-e · 55 = NF-e (padrão)
    .send()
    .await?;
// r.c_stat, r.x_motivo, r.url
```

`modelo` escolhe o webservice: em várias UFs (SP, por exemplo) a NFC-e roda em outro host e um pode
cair sem o outro. Sem `.modelo(…)` consulta o da NF-e (55) — o comportamento de antes. Modelo fora
de 55/65 é recusado na validação.

A SEFAZ controla consumo indevido também neste serviço: o manual pede **no mínimo 3 minutos** entre
consultas, e laço de consulta rende `cStat 656` com o CNPJ bloqueado por 1 hora. Quem monitora em
segundo plano deve espaçar (o PDV usa 5 min) e preferir consultar depois de uma falha de comunicação.

Guia completo (métodos, campos da resposta, tabela de `cStat`, erros): [`docs/status-webservice.md`](docs/status-webservice.md).

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

## Inutilização de numeração — `InutilizacaoBuilder`

`inutNFe` — declara à SEFAZ que uma faixa de números **não foi e não será usada**. É o desfecho
de um número "pulado": quando a emissão falha na comunicação e a venda é concluída com o número
seguinte, aquele número fica em aberto até ser inutilizado (ou até a nota aparecer autorizada na
consulta de situação, caso em que o caminho é cancelar, não inutilizar).

```rust
use dfe::InutilizacaoBuilder;

let r = InutilizacaoBuilder::new()
    .cert("caminho.pfx", "senha")
    .tp_amb(2)
    .uf("SP")
    .cnpj("11.222.333/0001-81") // aceita máscara e CNPJ alfanumérico
    .mod_(65)                   // 55 = NF-e | 65 = NFC-e
    .serie(1)
    .faixa(325, 325)            // um número só = faixa(n, n)
    .justificativa("Numeracao pulada por falha de comunicacao na emissao") // 15 a 255
    .send()
    .await?;

r.response.c_stat;   // "102" = inutilização homologada (único sucesso)
r.response.n_prot;   // Option<String> — protocolo, só quando homologada
```

`.ano("26")` é opcional (padrão: ano corrente, 2 dígitos). Rejeições comuns: a faixa já foi usada
por uma nota autorizada, ou já existe pedido de inutilização para ela.

---

## Emissão em duas etapas — `NFeBuilder::assinar` + `NFeAssinada::transmitir`

`emitir()` faz as duas metades de uma vez. Separá-las serve para **saber a chave de acesso antes
do envio**: se a transmissão falhar por rede, não se sabe se a SEFAZ recebeu a nota, e com a
chave em mãos dá para consultar a situação depois em vez de reemitir às cegas com o mesmo número.

```rust
let assinada = NFeBuilder::new() /* … */ .assinar().await?;
assinada.chave; // 44 dígitos, lidos do Id do XML assinado
assinada.xml;   // <NFe> assinada, exatamente o que vai no envelope

match assinada.transmitir().await {
    Ok(resp) => { /* cStat da SEFAZ — inclusive rejeição fiscal */ }
    Err(e)   => { /* rede/timeout: a nota PODE ter sido autorizada — consulte a chave */ }
}
```

`gerar_xml()` continua existindo e é atalho para `assinar().await?.xml` (usado pela contingência
off-line da NFC-e, que assina e imprime sem transmitir).

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

## DANFE Simplificado Tipo 2 da NF-e (ESC/POS) — `EscPosDanfeNFeBuilder`

DANFE 80 mm da **NF-e modelo 55** em impressora térmica, no leiaute do Ajuste SINIEF 13/2026 e
da **NT 2026.003** (nove divisões, na ordem da norma). Recusa modelo 65 — NFC-e é o
`EscPosNFCeBuilder`.

```rust
use dfe::EscPosDanfeNFeBuilder;

let bytes = EscPosDanfeNFeBuilder::new()
    .xml(xml_nfe_proc)   // string do nfeProc autorizado, ou caminho terminado em ".xml"
    .paper_width(80)     // 80 ou 58; abaixo de 56 mm o build() recusa (mínimo da NT)
    .columns(42)         // opcional: colunas do modelo da impressora
    .build()?;           // Vec<u8> pronto para job RAW
```

- **Fidelidade ao XML:** a NT proíbe imprimir o que não está no XML. O **QR Code** (Divisão V)
  só sai com `infNFeSupl/qrCode`, e o bloco **IBS/CBS/IS** (Divisão III-A) só com
  `IBSCBSTot`/`ISTot`. Sem `urlChave`, a consulta aponta para o portal nacional da NF-e. O cupom
  não leva o crédito "Gerado por dfe".
- **Avisos:** homologação (`tpAmb=2`) → "SEM VALOR FISCAL" na Divisão VIII; contingência em que
  o DANFE sai antes da autorização (`tpEmis` 2, 4, 5 ou 9) → aviso em dois locais. SVC (6/7) não
  leva aviso.
- **Limitação atual:** a emissão (`NFeBuilder`) só gera `infNFeSupl` para o modelo 65, então a
  NF-e 55 emitida pela crate ainda sai sem QR.
