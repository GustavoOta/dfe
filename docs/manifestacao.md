# Manifestação do Destinatário

Eventos enviados ao **Ambiente Nacional (AN)**. O `Manifestacao` é compartilhado entre os quatro eventos.

```rust
use dfe::manifestacao::{
    ciencia_operacao,
    confirmacao_operacao,
    desconhecimento_operacao,
    operacao_nao_realizada,
};
use dfe::tipos::manifestacao::{Manifestacao, OperacaoNaoRealizada};

let params = Manifestacao {
    cert_path:  "./cert.pfx".to_string(),
    cert_pass:  "senha".to_string(),
    cnpj:       "11111111111111".to_string(),
    tp_amb:     2,
    mod_:       None,   // None = 55; Some(65) para NFC-e
    chave:      "35241211111111111111550010000000361491395167".to_string(),
};

// 210210 — Ciência da Operação
let r = ciencia_operacao(params.clone()).await?;

// 210200 — Confirmação da Operação
let r = confirmacao_operacao(params.clone()).await?;

// 210220 — Desconhecimento da Operação
let r = desconhecimento_operacao(params.clone()).await?;

// 210240 — Operação Não Realizada (requer justificativa mín. 15 chars)
let r = operacao_nao_realizada(OperacaoNaoRealizada {
    justificativa: "Mercadoria não recebida".to_string(),
    ..params.into()
}).await?;

println!("cStat: {}",   r.response.c_stat);
println!("xMotivo: {}", r.response.x_motivo);
```
