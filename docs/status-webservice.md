# Status do Webservice

Consulta se o serviço da SEFAZ está no ar (`consStatServ`, serviço `NFeStatusServico4`). É uma
consulta **não assinada**: só o mTLS com o certificado A1 da empresa, sem assinatura XML.

```rust
use dfe::NFeService;

let r = NFeService::new()
    .cert_path("./cert.pfx")
    .cert_pass("senha")
    .uf("SP")
    .environment(1)   // 1 = Produção | 2 = Homologação
    .modelo(65)       // 65 = NFC-e | 55 = NF-e (padrão)
    .send()
    .await?;

println!("cStat: {}",   r.c_stat);    // "107" = Serviço em operação
println!("xMotivo: {}", r.x_motivo);
println!("URL: {}",     r.url);
```

## Métodos do builder

| Método | Obrigatório | Descrição |
|---|---|---|
| `cert_path(&str)` | sim | Caminho do certificado A1 (`.pfx`). Precisa existir e ser arquivo. |
| `cert_pass(&str)` | sim | Senha do certificado. |
| `uf(&str)` | sim | Sigla da UF do emitente (`"SP"`, `"RJ"`…). Define a SEFAZ consultada e o `cUF` do XML. |
| `environment(u8)` | sim | `1` = Produção · `2` = Homologação. |
| `modelo(u32)` | não | `55` = NF-e (padrão) · `65` = NFC-e. Escolhe o webservice. |
| `build()` | — | Só valida os campos, sem enviar. Devolve `Result<Self, String>`. |
| `send()` | — | Valida, envia e devolve `Result<NFeServiceResponse, String>`. |

### Por que o modelo importa

NF-e e NFC-e têm **webservices de status separados**, e em várias UFs moram em hosts diferentes.
Em SP, por exemplo:

| Modelo | Produção |
|---|---|
| 55 (NF-e) | `https://nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx` |
| 65 (NFC-e) | `https://nfce.fazenda.sp.gov.br/ws/NFeStatusServico4.asmx` |

Um pode cair sem o outro. Quem decide contingência de NFC-e (`tpEmis=9`) tem que consultar o
status com `.modelo(65)` — olhar o da NF-e responde a pergunta errada.

Sem `.modelo(…)` a consulta vai para o webservice da NF-e (55), que era o único comportamento
antes do método existir; código antigo continua funcionando igual. As 27 UFs têm endpoint de
status para os dois modelos nos dois ambientes (`src/data/webservices.json`, travado pelo teste
`status::endpoint::tests::todas_as_ufs_tem_status_para_os_dois_modelos`).

O XML enviado é o mesmo para os dois modelos (`tpAmb`, `cUF`, `xServ=STATUS`); só muda a URL.

## Resposta — `NFeServiceResponse`

| Campo | Tipo | Conteúdo |
|---|---|---|
| `c_stat` | `String` | Código de status da SEFAZ. |
| `x_motivo` | `String` | Descrição devolvida pela SEFAZ. |
| `sent_xml` | `String` | Envelope SOAP enviado. |
| `received_xml` | `String` | Resposta SOAP crua. |
| `url` | `String` | Endpoint consultado — confirma qual webservice (modelo) respondeu. |

### Códigos mais comuns

| `c_stat` | Significado | Leitura prática |
|---|---|---|
| `107` | Serviço em operação | Online — pode emitir. |
| `108` | Serviço paralisado momentaneamente | Fora do ar — NFC-e pode ir para contingência. |
| `109` | Serviço paralisado sem previsão | Fora do ar — idem. |
| `656` | Consumo indevido | A SEFAZ bloqueou consultas do CNPJ (até 1 h). **Não** indica serviço fora. |
| `280`–`299` | Problemas de certificado | SEFAZ no ar, mas recusou o certificado (vencido, revogado, cadeia…). |

Só `107` significa "em operação". Qualquer outro código deve ser tratado como "não confirmado",
não automaticamente como "fora do ar" (o `656`, em especial, é bloqueio de quem consulta).

## Erros

`send()` devolve `Err(String)` — sem resposta da SEFAZ não há `c_stat`:

| Situação | Mensagem (resumo) |
|---|---|
| `cert_path` vazio / inexistente / não é arquivo | `O path do certificado e obrigatorio` · `Arquivo não encontrado: …` |
| `cert_pass` vazia | `A senha do certificado e obrigatoria` |
| `uf` vazia | `A UF e obrigatoria` |
| `environment` fora de 1/2 | `O ambiente deve ser 1 (producao) ou 2 (homologacao)` |
| `modelo` fora de 55/65 | `O modelo deve ser 55 (NF-e) ou 65 (NFC-e)` |
| UF sem endpoint | `Endpoint não encontrado: servico=NfeStatusServico …` |
| Rede, timeout, TLS, senha errada do `.pfx` | mensagem do transporte mTLS |

Para o consumidor, `Err` = "não conseguimos falar com a SEFAZ" (internet, certificado local ou
SEFAZ lenta) — diferente de um `c_stat` de paralisação, que é a SEFAZ dizendo que está parada.

## Consumo indevido — com que frequência consultar

A SEFAZ controla abuso também neste serviço:

- o Manual de Orientação do Contribuinte pede **intervalo mínimo de 3 minutos** entre consultas;
- laço de consulta rende `cStat 656` e **bloqueia o CNPJ por 1 hora** (desbloqueio automático);
- a recomendação é consultar **reativamente**, depois de uma falha de comunicação na emissão, e não
  antes de cada envio.

A crate não guarda estado nem limita a frequência — cada `send()` é uma consulta real. O controle é
do consumidor. Referência no ecossistema: o `gravis-pdv` consulta 30 s depois de abrir e depois a
cada **5 min** (os dois modelos), nunca repete um modelo antes de 3 min — seja qual for a origem da
consulta (monitor, clique do operador, emissão, cancelamento) — e mostra o resultado no badge
"SEFAZ" do rodapé (`src/sefaz/sefazStatusEstado.js`).

## Teste ao vivo

`tests/sefaz_live.rs::live_status_servico` consulta de verdade quando o `.env` está configurado
(ver `env.example`); sem ele, o teste é pulado.
