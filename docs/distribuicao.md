# Distribuição de DF-e

Consulta documentos fiscais de interesse do CNPJ no Ambiente Nacional.

```rust
use dfe::distribuicao::{
    Distribuicao,               // últimos documentos a partir do NSU atual
    DistribuicaoNSU,            // a partir de um NSU específico
    DistribuicaoChaveAcesso,    // por chave de acesso
};

// Últimos documentos
let r = Distribuicao::new()
    .cert_path("./cert.pfx")
    .cert_pass("senha")
    .cnpj("11111111111111")
    .uf(35)       // código IBGE da UF (SP = 35)
    .ambiente(2)
    .send()
    .await?;

println!("ultNSU: {} | maxNSU: {}", r.ult_nsu, r.max_nsu);

if let Some(docs) = r.lote_dist_dfe_int {
    for doc in docs {
        println!("NSU: {} | Schema: {}", doc.nsu, doc.schema);
        if let Some(nfe) = doc.content {
            println!("  chNFe: {} | vNF: {}", nfe.ch_nfe, nfe.v_nf);
        }
    }
}

// A partir de um NSU específico
let r = DistribuicaoNSU::new()
    .cert_path("./cert.pfx").cert_pass("senha")
    .cnpj("11111111111111").uf(35).ambiente(2)
    .nsu("000000000000100")
    .send().await?;

// Por chave de acesso
let r = DistribuicaoChaveAcesso::new()
    .cert_path("./cert.pfx").cert_pass("senha")
    .cnpj("11111111111111").uf(35).ambiente(2)
    .chave_acesso("35241211111111111111550010000000361491395167")
    .send().await?;
```
