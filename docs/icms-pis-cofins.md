# Tipos de ICMS, PIS e COFINS

## ICMS

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

## IPI por item

```rust
use dfe::tipos::Ipi;

// CST 50 — saída tributada por alíquota ad valorem
ipi: Some(Ipi::tributado("999", v_bc, p_ipi, v_ipi))

// CST 53 — saída não tributada
ipi: Some(Ipi::nao_tributado("999", "53"))
```

## PIS / COFINS

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

## Validação de CNPJ / CPF

```rust
use dfe::{validate_cnpj, validate_cpf};

assert!(validate_cnpj("11.222.333/0001-81"));
assert!(validate_cnpj("11222333000181"));

assert!(validate_cpf("529.982.247-25"));
assert!(validate_cpf("52998224725"));
```
