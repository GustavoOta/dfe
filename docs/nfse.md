# NFS-e Nacional (feature `nfse`)

Emissão de NFS-e pela **SEFIN Nacional** (Emissor Público Nacional): a crate monta a DPS v1.01,
assina, confere no XSD oficial, transmite e consulta. Exemplo completo de uso em
[`API.md`](../API.md#nfs-e-nacional--dfenfse-feature-nfse).

```toml
dfe = { version = "…", features = ["nfse"] }
```

Escopo atual: prestador **MEI** ou **Simples Nacional**, serviço nacional **sem retenção** de ISSQN.
Ainda não: grupo IBS/CBS, retenções, regime normal, eventos (cancelamento/substituição) e DANFSe.

---

## Ambientes

| | Nome oficial | Endereço da API (SEFIN) | `tpAmb` | Valor fiscal |
|---|---|---|---|---|
| Testes | **Produção restrita** (é a "homologação" da NFS-e) | `https://sefin.producaorestrita.nfse.gov.br/SefinNacional` | 2 | nenhum |
| Real | Produção | `https://sefin.nfse.gov.br/SefinNacional` | 1 | sim |

`Ambiente::ProducaoRestrita` / `Ambiente::Producao`. O cliente recusa, sem enviar nada, DPS de um
ambiente transmitida por cliente de outro.

A comunicação é REST/JSON com **TLS mútuo**: o certificado A1 (ICP-Brasil, com "Autenticação do
Cliente") vai no TLS e também assina a DPS. Sem certificado, o servidor responde **403 do IIS**
(página HTML), não um erro da API.

---

## Antes de emitir: o município precisa estar no Emissor Nacional — em cada ambiente

A adesão do município é **separada por ambiente**. É comum o município emitir em produção e **não**
estar ativo, ou não usar o Emissor Nacional, na produção restrita. Nesse caso a SEFIN rejeita a DPS
na restrita e não há contorno pelo sistema.

**Como conferir** (só leitura, com o A1 no TLS) — fica no **ADN**, não na SEFIN:

```text
GET https://adn.producaorestrita.nfse.gov.br/parametrizacao/{códigoIBGE}/convenio   (restrita)
GET https://adn.nfse.gov.br/parametrizacao/{códigoIBGE}/convenio                    (produção)
```

```json
{"parametrosConvenio":{"aderenteAmbienteNacional":1,"aderenteEmissorNacional":1,
 "situacaoEmissaoPadraoContribuintesRFB":1,"aderenteMAN":0,"permiteAproveitametoDeCreditos":true},
 "mensagem":"Parâmetros do convênio recuperados com sucesso."}
```

- `aderenteEmissorNacional = 0`: o município não usa o Emissor Nacional naquele ambiente.
- HTTP 404 com *"O convênio do município … ainda não está ativo no Sistema Nacional da NFS-e"*: sem
  convênio naquele ambiente.
- O painel de monitoramento de adesões do gov.br/nfse mostra o mesmo.

### Exceção do MEI (Anexo I oficial)

Pela aba de regras da DPS do Anexo I (produção v1.01-20260209 e produção restrita
v1.01-20261020), as regras de convênio do município emissor **não se aplicam a MEI**
(`opSimpNac = 2`):

| Regra | Exige | Exceção |
|---|---|---|
| E0016 | competência ≥ data de ativação do convênio | MEI |
| E0037 | município emissor no cadastro de convênio | MEI |
| E0038 | convênio "ATIVO" | MEI |
| E0039 | município parametrizado para os emissores públicos nacionais | MEI; na planilha da restrita, também ME/EPP com `regApTribSN = 1` e competência ≥ 01/11/2026 |
| E0041 | MEI emite pelo município do endereço no cadastro CNPJ | — |

Ou seja: **checagem de convênio não deve barrar MEI**. Para os demais regimes, sem adesão do
município naquele ambiente não há emissão.

### Sem município ativo na produção restrita

1. Pedir à prefeitura que ative o convênio também no ambiente de testes.
2. Testar com uma empresa de outro município ativo na restrita.
3. Emitir uma NFS-e **real** de valor simbólico e cancelar — é documento fiscal de verdade: conferir
   antes o **prazo de cancelamento** do município, descrever como teste de integração, emitir sem
   tomador (ou com tomador de acordo), cancelar em seguida e avisar o contador. A crate ainda não
   tem o evento de cancelamento: cancelar pelo portal do Emissor Nacional.

---

## Problema conhecido: produção restrita respondendo 403 a todo pedido

Em 23-11-2026, a API da produção restrita (`sefin.producaorestrita.nfse.gov.br`) respondeu
**HTTP 403 do IIS** ("You do not have permission to view this directory or page using the
credentials that you supplied") a **todo** pedido, com certificado válido. A crate classifica essa
resposta do `transmitir` como `ErroEnvio::Indeterminado` (não é JSON da API) e o `consultar_dps`
devolve `DfeError::Webservice` com o HTTP 403.

**Diagnóstico** — feito assim, e repetível por quem cair no mesmo caso:

| Teste | Produção restrita | Produção |
|---|---|---|
| `HEAD /SefinNacional/dps/{id}` sem certificado | 403 | 403 |
| `HEAD /SefinNacional/dps/{id}` **com** A1 | **403** | **404** (DPS inexistente — resposta da API) |
| `GET /SefinNacional/docs/index` com A1 | **403** (também em `/API/SefinNacional/docs`) | 200 |
| Servidor pede o certificado no TLS? | **não** (nem em TLS 1.2 com renegociação) | sim |
| Mesmo teste no navegador com o A1 | 403 | — |
| ADN da restrita (`adn.producaorestrita…`) com o A1 | responde normalmente | — |

Ferramentas: o próprio cliente da crate, `openssl s_client` (`-cert`/`-key` em PEM extraído do
PFX, testando `-tls1_2` e o padrão) e o navegador com o certificado instalado. Com os mesmos rede,
máquina e certificado, a **produção** pediu o certificado e atendeu — então rede, certificado e o
endereço `/SefinNacional` estão certos, e o 403 é do servidor da produção restrita, antes da API
(nenhuma regra de negócio chega a rodar: uma rejeição de convênio viria em JSON com `E0039` etc.).

```bash
# Extrair o PEM do PFX (apague o arquivo depois do teste)
openssl pkcs12 -in certificado.pfx -nodes -out cert.pem          # pede a senha
H=sefin.producaorestrita.nfse.gov.br                               # ou sefin.nfse.gov.br
printf "HEAD /SefinNacional/dps/DPS%042d HTTP/1.1\r\nHost: $H\r\nConnection: close\r\n\r\n" 1 \
  | openssl s_client -quiet -connect $H:443 -servername $H -cert cert.pem -key cert.pem
rm cert.pem
```

Leitura do resultado: **403 sem pedir certificado** na restrita e **404 com certificado** na
produção = o caso descrito aqui. Não há o que ajustar do lado do cliente; o caminho é o
**Fale Conosco** do [gov.br/nfse](https://www.gov.br/nfse), informando endereço, horário e que a
produção responde com o mesmo certificado.

**Resposta real da produção para DPS inexistente** (travada em teste da crate):

```json
{ "tipoAmbiente": 0, "dataHoraProcessamento": "2026-11-23T18:21:21.6638409-03:00",
  "erro": { "codigo": "E2404", "descricao": "Não foi gerada uma NFS-e com o identificador de DPS informado" } }
```

HTTP 404 → `ConsultaDps::NaoEncontrada`. A SEFIN responde em camelCase e com `erro` no singular; o
Swagger do ADN publica PascalCase (`Erros`, `Alertas`). A crate lê as duas formas.

---

## Testes

- Offline (`cargo test --features nfse`): assinatura com PFX sintético (XML assinado travado em
  snapshot e conferido à parte com lxml e `openssl dgst -verify`), XSD dos dois ambientes e o
  cliente REST com transporte simulado.
- Ciclo real opt-in, **sempre na produção restrita** (ambiente fixo no código):
  `cargo test --features "sefaz-live nfse" --test nfse_live -- --nocapture`, com
  `DFE_TEST_NFSE_*` no `.env` (ver `env.example`). Serviço de R$ 1,00.

## Referências

- [Manual Integrado do SN NFS-e](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/manualintegradosnnfse_v1-00-02-producao.pdf) — §6.1.4, padrão de assinatura
- [APIs — Produção Restrita e Produção](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/apis-prod-restrita-e-producao)
- Anexo I — [produção](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/documentacao-atual/anexo_i-sefin_adn-dps_nfse-snnfse-v1-01-20260209.xlsx) · [produção restrita](https://www.gov.br/nfse/pt-br/biblioteca/documentacao-tecnica/producao-restrita/anexo_i-sefin_adn-dps_nfse-snnfse_prodrest-v1-01-20261020.xlsx)
- [Fórum ACBr — E0037 em produção × homologação](https://www.projetoacbr.com.br/forum/topic/92249-rejei%C3%A7%C3%A3o-e0037-o-c%C3%B3digo-do-munic%C3%ADpio-emissor-informado-na-dps-%C3%A9-inexistente-no-cadastro-de-conv%C3%AAnio-municipal-do-sistema-nacional-como-saber-se-o-munic%C3%ADpio-aderiu-ao-padr%C3%A3o-nacional-tanto-em-produ%C3%A7%C3%A3o-quanto-homologa%C3%A7%C3%A3o)
