# Status do Webservice

```rust
use dfe::NFeService;

let r = NFeService::new()
    .cert_path("./cert.pfx")
    .cert_pass("senha")
    .uf("SP")
    .environment(2)   // 1 = Produção | 2 = Homologação
    .send()
    .await?;

println!("cStat: {}",   r.c_stat);    // "107" = Serviço em operação
println!("xMotivo: {}", r.x_motivo);
println!("URL: {}",     r.url);
```
