# XSDs oficiais da DPS v1.01

Arquivos extraídos sem alterações dos pacotes oficiais consultados em 18/11/2026.
Cada pasta contém o schema da DPS e seus imports. As fixtures dos testes são
geradas em memória com dados sintéticos.

O XSD original de produção usa `^0{0,4}\d{1,5}$` para `TSSerieDPS`.
O libxml2 interpreta `^` e `$` como literais conforme a sintaxe XSD.
O teste de produção exige que esse seja o único erro. O de produção
restrita exige validação integral.

| Pasta | Pacote | SHA-256 do ZIP |
|---|---|---|
| `prod` | [Produção v1.01-20260209](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/documentacao-atual/nfse-esquemas_xsd-v1-01-20260209.zip) | `e7935cbd9470527c6cc32984c1b2263e614183bf0139ce2733eaaed2de9a8072` |
| `rest` | [Produção restrita v1.01-20261020](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/producao-restrita/esquemas_prod_restrita_20261020.zip) | `25fdcecc651a67a216496ab10d7566631e6ed10f721e37d62c7845dadb0ea267` |
