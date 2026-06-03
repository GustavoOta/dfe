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
