# Emissão NF-e / NFC-e

```rust
use dfe::NFeBuilder;
use dfe::tipos::{Det, Emit, Icms, Ide, InfAdic, Pag, Pis, Cofins, Total, Transp};
use dfe::tipos::emissao::Dest;

let resposta = NFeBuilder::new()
    .cert("./cert.pfx", "senha_do_pfx")
    .ide(Ide {
        c_uf: 35,           // cUF da UF emitente (SP = 35)
        mod_: 55,           // 55 = NF-e | 65 = NFC-e
        serie: 1,
        n_nf: 100,
        tp_amb: 2,          // 1 = Produção | 2 = Homologação
        tp_nf: 1,           // 0 = Entrada | 1 = Saída
        nat_op: "VENDA DE MERCADORIA".to_string(),
        ..Default::default()
    })
    .emitente(Emit {
        cnpj: Some("11111111111111".to_string()),
        x_nome: "EMPRESA LTDA".to_string(),
        ie: Some("111111111111".to_string()),
        crt: 1,             // 1 = Simples Nacional | 3 = Regime Normal
        ..Default::default()
    })
    .destinatario(Dest {   // opcional para NFC-e
        cnpj: Some("22222222222222".to_string()),
        x_nome: Some("CLIENTE LTDA".to_string()),
        ..Default::default()
    })
    .itens(vec![Det {
        c_prod: "001".to_string(),
        x_prod: "PRODUTO EXEMPLO".to_string(),
        ncm: "22030000".to_string(),
        cfop: 5102,
        u_com: "UN".to_string(),
        q_com: 2.0,
        v_un_com: 50.0,
        v_prod: 100.0,
        icms: Icms::sn102(0, "400"),      // CSOSN 400
        pis: Pis::Nt { cst: "07".to_string() },
        cofins: Cofins::Nt { cst: "07".to_string() },
        ..Default::default()
    }])
    .total(Total::default())         // totais calculados automaticamente dos itens
    .transporte(Transp { mod_frete: 9, ..Default::default() })
    .pagamento(Pag { ..Default::default() })
    .informacoes_adicionais(InfAdic {
        inf_cpl: Some("Pedido 123".to_string()),
        ..Default::default()
    })
    // NFC-e: obrigatório também .id_csc() e .csc()
    .emitir()
    .await?;

println!("cStat:     {}", resposta.protocolo.inf_prot.c_stat);
println!("xMotivo:   {}", resposta.protocolo.inf_prot.x_motivo);
println!("Protocolo: {}", resposta.protocolo.inf_prot.n_prot.unwrap_or_default());
// resposta.xml → XML autorizado (nfeProc) completo
```

## Métodos do NFeBuilder

| Método | Obrigatório | Descrição |
|---|:---:|---|
| `.cert(path, pass)` | ✅ | Caminho e senha do certificado `.pfx` |
| `.ide(Ide)` | ✅ | Identificação do documento |
| `.emitente(Emit)` | ✅ | Dados do emitente |
| `.itens(Vec<Det>)` | ✅ | Lista de itens; totais calculados automaticamente |
| `.total(Total)` | ✅ | Informar apenas frete, seguro, ST, FCP — demais campos auto-calculados |
| `.transporte(Transp)` | ✅ | Modalidade de frete e dados do transportador |
| `.pagamento(Pag)` | ✅ | Forma de pagamento |
| `.destinatario(Dest)` | — | Obrigatório para NF-e mod 55 |
| `.informacoes_adicionais(InfAdic)` | — | Informações complementares e ao fisco |
| `.id_csc(str)` | — | ID do CSC — **obrigatório NFC-e** |
| `.csc(str)` | — | Valor do CSC — **obrigatório NFC-e** |
| `.desconto_rateio(Decimal)` | — | Desconto global rateado proporcionalmente nos itens |
| `.emitir()` | — | Valida, assina e transmite para a SEFAZ |

## Totais automáticos

Os campos `v_bc`, `v_icms`, `v_prod`, `v_pis`, `v_cofins`, `v_desc` e `v_nf` são **calculados automaticamente** dos itens. No `Total` informe apenas despesas extras:

| Campo | Quando usar |
|---|---|
| `v_frete`, `v_seg`, `v_outro` | Frete, seguro e outras despesas |
| `v_ii`, `v_ipi`, `v_ipi_devol` | Impostos específicos |
| `v_bc_st`, `v_st` | ST global (itens com ICMS10/30/70 auto-somam) |
| `v_fcp`, `v_fcpst`, `v_fcpst_ret` | Fundo de Combate à Pobreza |
| `v_fcpuf_dest`, `v_icms_uf_dest`, `v_icms_uf_remet` | DIFAL |

Para uma venda simples sem extras: `Total::default()`.

## Tipos de ICMS por item

Veja a referência completa em [icms-pis-cofins.md](icms-pis-cofins.md).
