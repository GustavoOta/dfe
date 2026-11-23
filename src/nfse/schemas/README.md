# XSDs oficiais da DPS v1.01

Embutidos na crate (`src/nfse/xsd.rs`, `include_bytes!`) e usados para validar a DPS depois de
assinada. Cada pasta contém o schema da DPS e seus imports, extraídos dos pacotes oficiais
consultados em 18/11/2026.

| Pasta | Pacote | SHA-256 do ZIP |
|---|---|---|
| `prod` | [Produção v1.01-20260209](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/documentacao-atual/nfse-esquemas_xsd-v1-01-20260209.zip) | `e7935cbd9470527c6cc32984c1b2263e614183bf0139ce2733eaaed2de9a8072` |
| `rest` | [Produção restrita v1.01-20261020](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/producao-restrita/esquemas_prod_restrita_20261020.zip) | `25fdcecc651a67a216496ab10d7566631e6ed10f721e37d62c7845dadb0ea267` |

## Única alteração: `TSSerieDPS` no pacote de produção

O original de produção usa `^0{0,4}\d{1,5}$` em `TSSerieDPS` (`prod/tiposSimples_v1.01.xsd`).
Na sintaxe de expressão regular do XSD, `^` e `$` **não são âncoras**: o padrão já casa com o
valor inteiro, e o libxml2 lê os dois como caracteres literais. Com o original, toda DPS de
produção era reprovada na série. A linha passou a `0{0,4}\d{1,5}`, que é o que o padrão quer
dizer. Nenhuma outra linha mudou; `rest` está intacto.

| Arquivo (LF) | SHA-256 |
|---|---|
| `prod/tiposSimples_v1.01.xsd` original | `a2769845a367fdee495eb5a6ea6343639d36e0e5474ff65eb7b79a763fba9341` |
| `prod/tiposSimples_v1.01.xsd` corrigido | `70f16e2b638ab64658a7e05605274bc715d140aaf31e9b16f7bc1fba95f385fc` |

Travado por `nfse::xsd::tests::xsd_de_producao_tem_a_serie_sem_ancoras_literais`. Pacote novo
de produção: conferir se a SEFIN corrigiu o padrão antes de reaplicar a correção.
