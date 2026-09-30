# Testes

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
