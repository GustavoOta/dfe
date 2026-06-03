# Cancelamento

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

## Métodos do CancelarBuilder

| Método | Obrigatório | Descrição |
|---|:---:|---|
| `.cert(path, pass)` | ✅ | Certificado `.pfx` |
| `.tp_amb(u8)` | ✅ | Ambiente (1 = Produção, 2 = Homologação) |
| `.chave(str)` | ✅ | Chave de acesso de 44 dígitos |
| `.protocolo(str)` | ✅ | Protocolo de autorização da NF-e |
| `.justificativa(str)` | ✅ | Mínimo 15 caracteres |
| `.mod_(u32)` | — | Modelo do documento (padrão: 55) |
