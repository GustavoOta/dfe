# Testes

```bash
cargo test
```

Além dos testes de unidade dentro de `src/`, os testes de integração ficam em `tests/`:

| Suite | Arquivo | Descrição |
|---|---|---|
| DANFE NFC-e 80mm | `tests/danfe.rs` | Geração 80mm modelo 65 (multi-pagamento, CPF, qr_side, erros) |
| DANFE NF-e A4 | `tests/danfe_a4.rs` | Geração A4 modelo 55 (base64, arquivo, erro modelo 65) |
| ESC/POS | `tests/escpos.rs` | Bytes do cupom NFC-e |
| XML Extractor | `tests/xml_extractor.rs` | Parsing de `nfeProc` (snapshots `insta` em `tests/snapshots/`) |
| Homologação travada | `tests/homologacao_travada.rs` | Nenhum teste escolhe o ambiente por conta própria (ver abaixo) |
| SEFAZ ao vivo | `tests/sefaz_live.rs` | Status, emissão, cancelamento, distribuição, manifestação¹ |

¹ Só com a feature `sefaz-live` e um `.env` com o certificado (ver abaixo).

## Certificado e plataformas

Os testes de certificado rodam no `cargo test` padrão, no Windows e no Linux, com os **PFX
sintéticos** de `tests/fixtures/cert/` (moderno AES-256 e legado RC2-40 + 3DES, gerados por
`gerar.sh`; autoassinados, CNPJ fictício, senha pública). Os valores esperados foram medidos no
CAPI do Windows, e o Linux tem de devolver exatamente o mesmo:

| Módulo | Cobre |
|---|---|
| `interno::cert::tests` | `CertInfo` (titular, emissor, datas), certificado extraído (SHA-1 do DER inteiro), assinatura RSA-SHA1 byte a byte, senha errada, arquivo inexistente, PFX corrompido |
| `interno::cert::tests_mtls` | a identidade do mTLS abre os dois PFX (no Linux, depende do provider `legacy`) |
| `emissao::xml::qrcode_v2_online_tests` | QR Code da NFC-e: ID do CSC `000001` sai `1` e passa no padrão do XSD; mesmo hash com e sem zeros; ID inválido é erro |
| `interno::connection::tests` (só Linux) | a cadeia ICP-Brasil embutida carrega inteira, tem as ACs dos servidores da SEFAZ e nenhum certificado vencido |

Como e por quê: [Certificado digital e plataformas](certificado-e-plataformas.md).

## Camada `sefaz-live` — sempre homologação

```bash
cp env.example .env     # caminho, senha, CNPJ e UF do certificado — o .env nunca é versionado
cargo test --features sefaz-live
```

Sem `.env`, cada teste ao vivo é ignorado (não falha). **O ambiente não é configurável:** todo
teste que envia algo à SEFAZ usa a constante `tests/common::HOMOLOGACAO` (= 2), e
`tests/homologacao_travada.rs`, que roda no `cargo test` padrão, reprova qualquer
`.ambiente(…)`, `.tp_amb(…)`, `.environment(…)` ou `tp_amb:` fora dela.

Com o certificado e os dados do emitente no `.env` (IE, CRT, endereço, código do município,
série — ver `env.example`):

| Teste | O que faz | Exige |
|---|---|---|
| `live_status_servico` | Status do serviço (mTLS) | 107 |
| `live_ciclo_nfe_55_homologacao` | Emite uma NF-e, envia CC-e e cancela | 100, 135, 135 |
| `live_ciclo_nfce_65_homologacao` | Emite uma NFC-e (QR Code pelo CSC de homologação: `DFE_TEST_ID_CSC`/`DFE_TEST_CSC`) e cancela | 100, 135 |
| `live_inutilizacao_homologacao` | Inutiliza um número nunca usado, modelos 55 e 65 | 102 — ou 998 nas UFs que dispensaram a inutilização (SP, AC, BA) |
| `live_certificado_real` (`--lib`) | Dados, certificado e hash da assinatura, sem enviar nada — compare a linha `REF` entre Windows e Linux | sem erro |

O número da nota sai do relógio, na série `DFE_TEST_SERIE` (padrão 99): nunca repete e não colide
com as notas de homologação de outros sistemas que usem o mesmo CNPJ. Rodar sequencial
(`--test-threads=1`) e não repetir a distribuição em menos de uma hora — o Ambiente Nacional
responde `656` (consumo indevido) e bloqueia o CNPJ por 1 h.

Sem certificado e sem enviar documento, dá para conferir só o TLS com todos os servidores da
SEFAZ da tabela de webservices:

```bash
cargo test --features sefaz-live --lib live_tls -- --nocapture
```

## Testes de unidade contra o XSD

A montagem dos grupos fiscais é testada contra o **leiaute oficial embutido**
(`interno::validation::validar_fragmento`, só em `cfg(test)`): o teste serializa só o grupo
(`<ICMS>`, `<PIS>`, `<prod>`…) e valida contra um schema mínimo recortado do
`leiauteNFe_v4.00.xsd`, com todos os tipos simples globais do leiaute incluídos.

| Módulo | Cobre |
|---|---|
| `emissao::icms_calc::tests` | os 25 códigos de ICMS (com só os obrigatórios e com todos os opcionais), as 7 modalidades de ST em 8 grupos, pauta/MVA própria, ST desonerado, cBenefRBC, FCP diferido, efetivo, FCP-ST retido, ICMSPart, ICMSST, monofásico 02/15/53/61, motivos de desoneração, obrigatório ausente = erro |
| `emissao::det::tests_icms_tudo` | totais do monofásico e vNF com vICMSMonoReten, vFCPSTRet e ICMSPart nos totais, `<prod>` com gCred e comb no XSD, CST monofásico sem ANP = erro |
| `emissao::det::tests_pis_cofins_grupo` | os 32 CST de PIS/COFINS × 3 modalidades no XSD; variante errada ainda sai no grupo certo; PISST/COFINSST como grupos irmãos |
| `emissao::det::tests_pis_cofins_total` | vPIS/vCOFINS com Outr; PISST no vNF só com indSoma = 1 |
| `xml_extractor::structs::tests_icms_grupo` | DANFE lê o ICMS de qualquer grupo |
