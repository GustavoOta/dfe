# Tratamento de Erros

Todas as funções públicas retornam `Result<T, DfeError>`.

```rust
use dfe::DfeError;

match resultado {
    Ok(r)                        => { /* ... */ }
    Err(DfeError::Certificado(m)) => eprintln!("Problema no .pfx: {}", m),
    Err(DfeError::Validacao(m))   => eprintln!("Dado inválido: {}", m),
    Err(DfeError::Webservice(m))  => eprintln!("Falha SEFAZ: {}", m),
    Err(e)                        => eprintln!("Erro: {}", e),
}
```

| Variante | Quando ocorre |
|---|---|
| `Certificado` | Falha ao abrir o `.pfx` ou senha incorreta |
| `Xml` | Erro de parsing ou serialização XML |
| `Assinatura` | Falha na assinatura digital RSA-SHA1 |
| `Webservice` | Erro HTTP ou resposta inesperada da SEFAZ |
| `Validacao` | Campo obrigatório ausente ou fora das regras XSD |
| `Configuracao` | Falha ao ler configuração ou credenciais |
| `Io` | Erro de leitura/escrita em disco |
