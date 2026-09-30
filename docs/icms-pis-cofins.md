# Tipos de ICMS, PIS e COFINS

## ICMS

### Caminho de código novo: `Icms::Parametros(IcmsParametros)`

O consumidor manda só o que o cadastro do produto guarda (CST/CSOSN, percentuais,
modalidades, valores unitários, códigos) e a crate escolhe o grupo do XSD e calcula os valores
sobre a base do item — vProd + frete + acréscimo − desconto, já rateados — e a quantidade
tributável (`qTrib`), que a pauta e o ad rem dos combustíveis usam.

```rust
use dfe::tipos::{Icms, IcmsParametros};

Icms::Parametros(IcmsParametros {
    cst: "20".into(), orig: 0,
    p_icms: Some(18.0), p_red_bc: Some(33.33),
    mot_des_icms: Some(9), ind_deduz_deson: Some(0),
    ..Default::default()
})
```

Campo que o grupo exige e veio vazio é `DfeError::Validacao` na montagem, com o nome do campo —
a nota não sai com grupo errado nem com valor inventado. Todo grupo é validado contra o XSD
oficial nos testes (`emissao::icms_calc::tests`).

#### Grupos e parâmetros

| CST/CSOSN | Grupo do XML | Exigidos | Opcionais |
|---|---|---|---|
| 00 | ICMS00 | `p_icms` | `mod_bc`, `p_fcp` |
| 10 | ICMS10 | `p_icms`, `p_icmsst` | `mod_bcst`, `p_mvast`, `p_red_bcst`, `p_fcp`, `p_fcpst`, `mot_des_icms_st` |
| 20 | ICMS20 | `p_icms`, `p_red_bc` | `p_fcp`, `mot_des_icms`, `ind_deduz_deson` |
| 30 | ICMS30 | `p_icmsst` | ST, `p_fcpst`, desoneração (`p_icms` = alíquota que seria devida) |
| 40/41/50 | ICMS40 | — | desoneração (`p_icms` + `mot_des_icms`) |
| 51 | ICMS51 | — (sem `p_icms`: só orig + CST) | `p_red_bc`, `c_benef_rbc`, `p_dif`, `p_fcp`, `p_fcp_dif` |
| 60 | ICMS60 | — | ST retido (`v_bcst_ret`, `p_st`, `v_icms_substituto`, `v_icmsst_ret`), `p_fcpst_ret`, efetivo (`p_red_bc_efet`, `p_icms_efet`) |
| 70 | ICMS70 | `p_icms`, `p_red_bc`, `p_icmsst` | ST, FCP, desoneração, `mot_des_icms_st` |
| 90 | ICMS90 | — (cada bloco entra com a sua alíquota) | próprio (`p_icms` + `p_red_bc`, `c_benef_rbc`, `p_dif`, `p_fcp`, `p_fcp_dif`), ST (`p_icmsst` + …), desoneração, `mot_des_icms_st` |
| 10/20/90 + `uf_st` | **ICMSPart** (partilha) | `p_icms`, `p_icmsst`, `p_bc_op` | `p_red_bc` (20/90), ST, FCP-ST, desoneração (motivos 9, 10, 11) |
| 41/60 + `v_bcst_dest` | **ICMSST** (repasse de ST retido) | `v_bcst_ret`, `v_icmsst_ret`, `v_bcst_dest`, `v_icmsst_dest` | `p_st`, `v_icms_substituto`, `p_fcpst_ret`, efetivo |
| 02 | ICMS02 (monofásico próprio) | `ad_rem_icms` | — |
| 15 | ICMS15 (monofásico + retenção) | `ad_rem_icms`, `ad_rem_icms_reten` | `p_red_ad_rem` + `mot_red_ad_rem` (1 ou 9; padrão 9) |
| 53 | ICMS53 (monofásico diferido) | — (sem ad rem: só orig + CST) | `ad_rem_icms`, `p_dif` |
| 61 | ICMS61 (monofásico cobrado antes) | `ad_rem_icms_ret` | — |
| CSOSN 101 | ICMSSN101 | `p_cred_sn` | — |
| CSOSN 102/103/300/400 | ICMSSN102 | — | — |
| CSOSN 201 | ICMSSN201 | `p_icmsst`, `p_cred_sn` | ST, `p_icms` (alíquota interna, deduzida do ST) |
| CSOSN 202/203 | ICMSSN202 | `p_icmsst` | ST, `p_icms` |
| CSOSN 500 | ICMSSN500 | — | ST retido, `p_fcpst_ret`, efetivo |
| CSOSN 900 | ICMSSN900 | — | próprio (`p_icms` + …), ST (`p_icmsst` + …), `p_cred_sn` |

Os grupos ICMSPart e ICMSST são escolhidos pelo **parâmetro**, não por um CST próprio: `uf_st`
preenchida num 10/20/90 vira ICMSPart; `v_bcst_dest` num 41/60 vira ICMSST. Os valores do
ICMSST vêm da operação (não do cadastro do produto).

#### Modalidades da base de cálculo

Todas as do leiaute são calculadas.

| Campo | Valor | Base | Parâmetro exigido |
|---|---|---|---|
| `mod_bc` | 3 (padrão) | valor da operação | — |
| | 0 | base × (1 + MVA) | `p_mva_proprio` |
| | 1 (pauta), 2 (preço tabelado máx.) | `qTrib` × valor | `v_pauta` |
| `mod_bcst` | 4 (padrão) | base × (1 + `p_mvast`) | — (`p_mvast` opcional) |
| | 6 | valor da operação | — |
| | 0, 1, 2, 3, 5 (preço tabelado/listas/pauta) | `qTrib` × valor | `v_pauta_st` |

`p_red_bc` e `p_red_bcst` são aplicados depois da modalidade.

#### Como os valores são calculados

- **ICMS-ST** deduz o ICMS próprio (e o FCP próprio do FCP-ST). No Simples (201/202/203) deduz
  `base × p_icms` quando o cadastro informa a alíquota interna.
- **Desoneração** (`mot_des_icms`): vICMSDeson = base × `p_icms` − ICMS destacado. O motivo é
  conferido contra a lista que o XSD aceita no CST.
- **ST desonerado** (`mot_des_icms_st`, motivos 3/9/12): vICMSSTDeson = ICMS-ST sem a redução
  da BC-ST − ICMS-ST destacado.
- **Diferimento** (51/90): vICMSOp = ICMS da operação; vICMSDif = vICMSOp × `p_dif`;
  vICMS = vICMSOp − vICMSDif. **FCP diferido** (`p_fcp_dif`): vFCPDif = vFCP × %, vFCPEfet = o resto.
- **Efetivo** (60/500/ICMSST): vBCEfet = base × (1 − `p_red_bc_efet`); vICMSEfet = vBCEfet ×
  `p_icms_efet`. Sai quando `p_icms_efet` vem; redução 0 sai `0.0000` (o XSD aceita).
- **FCP-ST retido** (`p_fcpst_ret`): sobre o vBCSTRet (ou a base do item, sem ele).
- **Monofásico**: vICMSMono = `qTrib` × ad rem; no 53, vICMSMono = operação − diferido.
- `c_benef_rbc` só sai junto de `p_red_bc`.

### Dados do `<prod>`: `c_benef`, `cred_presumido`, `comb`

| Campo de `Det` | XML | Regra |
|---|---|---|
| `c_benef: Option<String>` | `prod/cBenef` (entre CEST e CFOP) | 8 ou 10 caracteres ou `SEM CBENEF`; vazio é omitido |
| `cred_presumido: Vec<CredPresumido>` | `prod/gCred` (até 4, depois do cBenef) | vCredPresumido = base do item × `p`; mais de 4 é erro |
| `comb: Option<Comb>` | `prod/comb` (fim do `<prod>`) | **obrigatório** nos CST 02/15/53/61 (sem código ANP é erro de montagem) |

```rust
use dfe::tipos::{Comb, CredPresumido, Encerrante};

Det {
    cred_presumido: vec![CredPresumido { codigo: "SP000002".into(), p: 3.0 }],
    comb: Some(Comb {
        c_prod_anp: "320102001".into(),
        desc_anp: "GASOLINA C COMUM".into(),
        uf_cons: "SP".into(),
        p_bio: Some(14.0),
        cide: Some((250.0, 0.10)),          // (qBCProd, vAliqProd) — vCIDE calculado
        encerrante: Some(Encerrante { n_bico: "1".into(), n_bomba: None, n_tanque: "3".into(), v_enc_ini: 1000.0, v_enc_fin: 1250.0 }),
        ..Default::default()
    }),
    ..Default::default()
}
```

### Variantes antigas (valor pronto)

Exigem que o chamador faça a conta. Continuam aceitas; código novo usa `Icms::Parametros`.

| Variante | CST/CSOSN | Construtor |
|---|---|---|
| `Icms00` | 00 | `Icms::icms00(orig, mod_bc, v_bc, p_icms, v_icms)` |
| `Icms10` | 10 | `Icms::icms10(orig, mod_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, v_bcst, p_icmsst, v_icmsst)` |
| `Icms20` | 20 | `Icms::icms20(orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms)` |
| `Icms30` | 30 | `Icms::icms30(orig, mod_bcst, p_mvast, v_bcst, p_icmsst, v_icmsst)` |
| `Icms40` | 40/41/50 | `Icms::icms40(orig, cst)` |
| `Icms51` | 51 | `Icms::icms51(orig)` + campos via struct literal |
| `Icms60` | 60 | `Icms::icms60(orig)` |
| `Icms70` | 70 | `Icms::icms70(orig, mod_bc, v_bc, p_icms, v_icms, mod_bcst, p_mvast, v_bcst, p_icmsst, v_icmsst)` |
| `Icms90` | 90 | `Icms::icms90(orig)` + campos opcionais via struct literal |
| `Sn101` | CSOSN 101 | `Icms::sn101(orig, p_cred_sn, v_cred_icmssn)` |
| `Sn102` | CSOSN 102/103/300/400 | `Icms::sn102(orig, csosn)` |
| `Sn500` | CSOSN 500 | `Icms::sn500(orig)` |
| `Sn900` | CSOSN 900 | `Icms::sn900(orig)` + campos opcionais via struct literal |

### Totais do ICMS (`ICMSTot`)

vBC/vICMS/vBCST/vST/vFCP/vFCPST/vICMSDeson somam os itens (ICMSPart incluído). `vFCPSTRet`
soma os itens (+ o informado em `Total`). Os totais do monofásico (`qBCMono`, `vICMSMono`,
`qBCMonoReten`, `vICMSMonoReten`, `qBCMonoRet`, `vICMSMonoRet`) só saem quando algum item é
02/15/53/61. O vNF soma `vICMSMonoReten` (NT 2023.001, regra 610) — fórmula completa em
[emissao-nfe-nfce.md](emissao-nfe-nfce.md#totais-automáticos).

## IPI por item

```rust
use dfe::tipos::Ipi;

// CST 50 — saída tributada por alíquota ad valorem
ipi: Some(Ipi::tributado("999", v_bc, p_ipi, v_ipi))

// CST 53 — saída não tributada
ipi: Some(Ipi::nao_tributado("999", "53"))
```

## PIS / COFINS

### Caminho de código novo: `Pis::por_parametros` / `Cofins::por_parametros`

A crate escolhe o grupo do XSD pelo CST e calcula o valor — qualquer um dos 32 CST do leiaute.

```rust
use dfe::tipos::{Cofins, Pis, PisCofinsParametros};

let p = PisCofinsParametros {
    cst: "49".into(),
    v_bc: 82.0,              // vProd, já sem o ICMS quando o produto o exclui
    aliquota: Some(1.65),    // %
    q_bc_prod: 3.5,          // qTrib
    v_aliq_prod: None,       // R$ por unidade
};
let pis = Pis::por_parametros(&p);
let cofins = Cofins::por_parametros(&PisCofinsParametros { aliquota: Some(7.6), ..p });
```

| CST | Grupo | Valor |
|---|---|---|
| 01, 02 | PISAliq / COFINSAliq | vBC × alíquota |
| 03 | PISQtde / COFINSQtde | qBCProd × vAliqProd |
| 04–09 | PISNT / COFINSNT | sem valor (alíquota do cadastro é ignorada) |
| 49–56, 60–67, 70–75, 98, 99 | PISOutr / COFINSOutr | com `v_aliq_prod` → por quantidade; com `aliquota` → por alíquota; sem nada → zerado |

Sem valor, use `Pis::nao_tributado(cst)` / `Cofins::nao_tributado(cst)`. **O grupo sai pelo CST,
não pela variante**: `Pis::Nt { cst: "49" }` vai para o PISOutr com CST 49, e
`Cofins::Outr { cst: "07" }` vai para o COFINSNT. Antes, CST 04–09 em `Outr` era rejeição de
schema ("CST '07' is not an element of the set {'49', …}") e o `Pis::Outr` saía sempre com CST 99.
A lista dos CST sem valor está em `dfe::tipos::CST_PIS_COFINS_NT`.

### Variantes

```rust
Pis::Aliq { cst, v_bc, p_pis, v_pis }                   // 01/02
Pis::Qtde { cst, q_bc_prod, v_aliq_prod, v_pis }        // 03
Pis::Nt { cst }                                         // sem valor (grupo pelo CST)
Pis::OutrAliq { cst, v_bc, p_pis, v_pis }               // 49–99 por alíquota
Pis::OutrQtde { cst, q_bc_prod, v_aliq_prod, v_pis }    // 49–99 por quantidade
Pis::Outr                                               // 99 zerado
Pis::St { v_bc, p_pis, q_bc_prod, v_aliq_prod, v_pis, ind_soma }  // CST 05 do substituto
```

`Cofins` tem as mesmas variantes (`Cofins::Outr { cst }` leva o CST).

**CST 05 do substituto (`Pis::St` / `Cofins::St`)**: sai `PISNT` com CST 05 dentro de `<PIS>` e o
grupo `PISST` **ao lado**, fora dele (ordem do XSD: PIS, PISST, COFINS, COFINSST). Informe
`v_bc + p_pis` **ou** `q_bc_prod + v_aliq_prod`. `ind_soma` é o `indSomaPISST`: com 1, o vPIS do
ST entra no vNF (NT 2020.005, regra 610). Antes o PISST saía dentro de `<PIS>` (XML inválido).

### Totais

`ICMSTot/vPIS` e `vCOFINS` somam os grupos Aliq, Qtde e **Outr**. O PISST/COFINSST não entra
neles — só no vNF, e só com `ind_soma = 1`.

## Validação de CNPJ / CPF

```rust
use dfe::{validate_cnpj, validate_cpf};

assert!(validate_cnpj("11.222.333/0001-81"));
assert!(validate_cnpj("11222333000181"));

assert!(validate_cpf("529.982.247-25"));
assert!(validate_cpf("52998224725"));
```
