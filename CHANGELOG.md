# Changelog

Registro de alterações da crate `dfe`. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).
Fases do refactor de arquitetura em `planning/ARQUITETURA_REFACTOR.md`.

## [Unreleased]

### A8 — Feature-gating / modularização de dependências

#### Changed (build — opt-out; **consumidores atuais não mudam**)
- `danfe`, `escpos` e `distribuicao` (este agrupa distribuição do AN **e** manifestação do
  destinatário) agora são **features de compilação**. O `default` liga as três, então quem consome a
  crate como hoje (`dfe = { path = … }`) recebe **exatamente a mesma API e comportamento** — zero
  quebra, respeitando o protocolo de não quebrar os apps.
- As dependências de **renderização** (`printpdf`, `image`, `barcoders`, `qrcodegen`, `font8x8`) e o
  `flate2` viraram `optional = true`. Desligar as features correspondentes **remove** essas deps do
  build:

  | Feature desligada | Dependências dropadas |
  |---|---|
  | `danfe` | `printpdf` (+ reduz `image`/`barcoders`/`qrcodegen`) |
  | `escpos` | `font8x8` (+ reduz `image`/`barcoders`/`qrcodegen`) |
  | `danfe` **e** `escpos` | toda a stack de render (`printpdf`, `image`, `barcoders`, `qrcodegen`, `font8x8`) |
  | `distribuicao` | `flate2` |

- **Core sempre ligado** (sem feature): emissão, cancelamento, substituição, CC-e, status,
  `xml_extractor`, `tipos`, `error` e todo o `interno` (assinatura, transporte, cert, roteamento,
  validação XSD).
- Consumidor minimalista pode optar por só o que usa, ex.:
  `dfe = { version = "…", default-features = false, features = ["danfe"] }`.
- `#[cfg(feature = …)]` aplicado aos `pub mod`/`pub use` gated no `lib.rs`, ao helper interno
  `interno::cnpj_cpf::format_cnpj_cpf` (só usado pela render) e aos testes correspondentes das
  camadas 2/3. **API pública e comportamento inalterados sob `default`.**
- **Verificado:** build e testes verdes sob `default`, `--no-default-features` e cada feature isolada;
  `--features sefaz-live` compila.

### A7 — Quebra de god files + convenção de módulo §2.1 (interno)

#### Changed (interno — sem mudança de API pública nem de comportamento)
- `emissao/mod.rs` (~600 linhas, misturava builder + montagem XML + assinatura + envelope + envio +
  parse) foi dividido na espinha §2.1: `mod.rs` (só o `NFeBuilder` + re-exports), `types.rs`
  (structs), `xml.rs` (`build_signed_xml` + `qrcode_hash`), `service.rs` (`emit_nfe`), `parser.rs`
  (`xml_result`/`extract_xml_tag` + testes). `NFeBuilder`/`Response`/`InfProt`/`TagInfProt`
  continuam exportados nos mesmos caminhos.
- `emissao/det_process/entity.rs` (~1100 linhas de structs de imposto) virou a pasta `entity/`:
  `icms.rs`, `pis_cofins.rs`, `ibscbs.rs`, `ipi.rs` (por família) + `mod.rs` (`ProdProcess`,
  `ImpostoProcess`/`to_xml`, `DetProcess` e os helpers de serialização), re-exportando tudo por
  glob — os caminhos `det_process::entity::*` seguem válidos.
- **Movimentação mecânica**, verificada pelo compilador (todas as resoluções de `serialize_with`
  cruzam módulos corretamente) e pela suíte verde; nenhuma alteração no XML gerado.

### A6 — Roteamento multi-UF (nacional)

#### Changed (comportamento — deixa de ser SP-only)
- A crate roteava **todos** os serviços SOAP para SP (`"SP"` fixo em emissão, cancelamento,
  substituição, CC-e; `webservices.json` só com SP). Agora a **UF real** é derivada do documento
  (2 primeiros dígitos da chave = `cUF`, ou `ide.cUF` na emissão) e resolvida para o endpoint
  correto. Status já recebia a UF do chamador.
- `webservices.json` populado com as **27 UFs** (NF-e mod 55 e NFC-e mod 65, ambientes produção/
  homologação), incluindo os estados atendidos por **autorizadores compartilhados** (SVAN/SVRS) —
  a delegação foi resolvida para a URL concreta de cada UF. Fonte: endpoints oficiais da SEFAZ.
  As linhas de **SP** (validadas em produção) foram **preservadas** sem alteração.
- Novo `interno::uf` com o mapa **código IBGE ↔ sigla** (num único lugar); `status` passou a usá-lo
  (antes duplicava a tabela sigla→código).
- **API pública inalterada** — os builders continuam iguais; muda só o endpoint resolvido.
- **Limitações remanescentes:** a URL do **QR Code da NFC-e** (`infNFeSupl`) ainda é de SP —
  precisa de um dataset de consulta-QR por UF (follow-up). Contingência **SVC** (svn=true) só existe
  para SP (pendência de contingência, fase futura). UF fora das 27 → erro claro (`DfeError::Validacao`).

### A5 — Parsing de eventos por serde (interno)

#### Changed (interno — sem mudança de API pública)
- `interno::evento::parse_ret_evento::<T>` deixou de **recortar o `<infEvento>` por regex**
  (`(?s)<infEvento.*?</infEvento>`, frágil e dependente de ordem) e passa a **desserializar
  tipado** o `retEnvEvento` via `quick_xml::de` (`RetEnvEvento<T> → RetEvento<T> → infEvento`).
- O único passo textual restante é isolar o subtree **`retEnvEvento`** (elemento de **nome estável**)
  do envelope SOAP, porque o **wrapper varia** entre implantações da SEFAZ
  (`nfeRecepcaoEventoResult`/`nfeResultMsg`/…) — desserializar o envelope inteiro pelo nome do
  wrapper seria frágil. Vale para cancelamento, manifestação, substituição e CC-e (um só ponto).
- **Lote rejeitado** (sem `<retEvento>`): o erro agora carrega o `cStat`/`xMotivo` de **nível de
  lote** (antes era um genérico "Erro ao capturar infEvento").
- **Primeiros testes offline** desse parse (antes não havia nenhum): resposta de cancelamento OK,
  de manifestação OK (com `CNPJDest`/`nProt`), lote rejeitado e SOAP Fault.

### S2 — Carta de Correção Eletrônica (evento 110110)

#### Added (API pública — **aditivo, não-breaking**)
- Novo `dfe::CartaCorrecaoBuilder` para o evento `tpEvento` **110110** (MOC / `CCe_v1.00.xsd`):
  corrige dados não ligados a valores/impostos/destinatário. Vale para NF-e e NFC-e. Setters
  `cert`, `tp_amb`, `chave`, `correcao` (`xCorrecao`), `n_seq_evento` (padrão 1);
  `send() -> Result<dfe::tipos::cancelar::Response>`.
- **`nSeqEvento` acumulativo:** o builder aceita `n_seq_evento` (≥ 1, default 1) — diferente do
  cancelamento/substituição (fixos em 1). O `Id`/`reference_uri` usam esse número.
- `xCondUso` é **constante embutida** (texto do Convênio S/N 15/12/1970, art. 7º §1º-A) — nunca
  informado pelo chamador. `descEvento` fixo "Carta de Correcao".
- Validações fail-fast: `xCorrecao` 15–1000, chave 44 dígitos, `nSeqEvento` ≥ 1. Quebras de linha
  no `correcao` são **normalizadas para espaço** (single-line) — a "C14N por string"
  (`clear_xml_string`) não preserva `\n`, então normalizar evita juntar palavras e mantém a
  assinatura consistente. Roteia pelo modelo da chave (SP-only, pendência #5 / A6).
- Golden `insta` do `<infEvento>` (110110, com `nSeqEvento` 1 e 3) travando a ordem do `detEvento`
  (`descEvento`/`xCorrecao`/`xCondUso`) e o texto fixo de `xCondUso`.

### S1 — Cancelamento por substituição de NFC-e (evento 110112)

#### Added (API pública — **aditivo, não-breaking**)
- Novo `dfe::SubstituicaoBuilder` para o evento `tpEvento` **110112** (NT 2018.004): cancela uma
  NFC-e duplicada mantendo a substituta válida. **Só modelo 65**; setters `cert`, `tp_amb`,
  `chave` (cancelada), `protocolo`, `chave_substituta` (`chNFeRef`), `ver_aplic`, `justificativa`;
  `send() -> Result<dfe::tipos::cancelar::Response>` (o `retEvento` tem a mesma forma do
  cancelamento comum).
- Validações da NT antes de qualquer I/O (fail-fast): modelo 65 (derivado da chave), `verAplic`
  não-vazio, `xJust` 15–255, chave com 44 dígitos. `descEvento`/`tpAutor`(=1)/`nSeqEvento`(=1)
  fixos internamente; `cOrgaoAutor` = UF da chave. Roteamento SP-only como o restante da crate.
- Golden `insta` do `<infEvento>` do 110112 (`substituicao::tests::golden_inf_evento_110112`)
  travando a ordem/conteúdo do `detEvento` conforme `eventoCancSubst_v1.00.xsd`.

#### Changed (interno — sem mudança de API pública)
- `interno::evento` ganhou `env_evento_xml(inf_evento, signature)` (envelope SOAP `envEvento`) e
  `parse_ret_evento::<T>(response)` (extração+desserialização do `retEvento`), **antes duplicados
  byte-a-byte** em `cancelar` e `manifestacao`. Ambos passam a delegar; `SubstituicaoBuilder` nasce
  sobre essa espinha única (`interno::evento` + `interno::assinatura` + `interno::transporte`).
- **Byte-identidade provada:** o golden do envelope migrou de `cancelar` para
  `interno::evento::tests::golden_env_evento_xml` com conteúdo idêntico; os golden de `infEvento`/
  assinatura de `cancelar`/`manifestacao` permanecem inalterados. Nenhuma mudança no XML enviado.
- A extração por regex do `retEvento` fica agora num **único lugar** (`parse_ret_evento`) — o alvo
  da futura fase A5 (desserialização tipada) passa a ser um ponto só, não quatro.

### A4b — `ManifestacaoBuilder` (BREAKING) + fim do log em CWD da manifestação

#### Changed (API pública — **BREAKING**)
- A manifestação do destinatário agora é um **builder**, `dfe::ManifestacaoBuilder`, alinhado ao
  padrão de `CancelarBuilder`/`NFeBuilder` (§2.1). Substitui as funções livres
  `manifestacao::{nfe_ciencia_operacao, nfe_confirmacao_operacao, nfe_desconhecimento_operacao,
  nfe_operacao_nao_realizada}` e as structs `tipos::manifestacao::{Manifestacao,
  OperacaoNaoRealizada}` (**removidas**).
- Novo uso — um método terminal por tipo de evento:
  ```rust
  use dfe::ManifestacaoBuilder;
  ManifestacaoBuilder::new()
      .cert(path, pass).cnpj(cnpj).tp_amb(2).chave(chave)
      .ciencia_operacao().await?;          // ou confirmacao_operacao / desconhecimento_operacao
  // Operação não Realizada (justificativa obrigatória):
  ManifestacaoBuilder::new().cert(..).cnpj(..).tp_amb(..).chave(..)
      .operacao_nao_realizada("Motivo...").await?;
  ```
- `tipos::manifestacao::{Response, InfEvento}` **permanecem** (o retorno é o mesmo `Response`).
- O campo `mod_` (modelo 55/65) some da entrada: era **morto** — a manifestação sempre usa o
  endpoint `RecepcaoEvento` do Ambiente Nacional, independente do modelo.
- **Como adaptar (consumidores):** trocar a construção da struct + chamada da função livre pela
  cadeia do builder. Feito no `gravisServer` (único consumidor) no mesmo passo.

#### Changed (comportamento — fim do side-effect de CWD, §1.1/§1.9)
- A manifestação **não escreve mais** em `./distribuicao-logs/{requests,responses,errors}/…` no
  diretório de trabalho (era corrida de dados sob o Actix multi-thread e poluía o CWD, igual ao
  que a A1 já resolvera em emissão/cancelamento). Os XMLs de envio e resposta continuam
  disponíveis em `Response.send_xml` / `Response.receive_xml`.
- Transporte migrado para `interno::transporte::MtlsTransport` (antes montava `Cert` + client +
  `post` inline; a migração fora adiada na A3a justamente por causa desse log). O `<infEvento>`
  assinado permanece **byte-idêntico** — travado pelos golden `manifestacao::tests`.

### A4a — infEvento genérico (interno)

#### Changed (interno — sem mudança de API pública)
- Novo `interno::evento` com `inf_evento_xml(InfEvento)` — montagem **genérica** do `<infEvento>`
  de eventos (parte comum + `detEvento` variável via `desc_evento` + `det_campos`). Antes
  duplicada em `cancelar` e `manifestacao`.
- `cancelar` (evento 110111) e `manifestacao` (210200/210210/210220/210240) delegam a ele.
  Diferenças (cOrgao UF vs 91; CNPJ da chave vs payload; detEvento) viram parâmetros.
- **Byte-identidade provada** por golden `insta` (`cancelar::tests::golden_inf_evento_xml` e
  `manifestacao::tests::golden_inf_evento_{sem,com}_just`, capturados da impl anterior). Zero
  mudança no `infEvento` (nó cujo digest é assinado).
- **Desbloqueia** eventos novos (substituição, CC-e): basta montar `det_campos` e chamar o genérico.

### A3b — assinatura XML-DSig unificada (interno)

#### Changed (interno — sem mudança de API pública)
- Novo `interno::assinatura` com `signed_info_xml(reference_uri, digest)` e
  `signature_xml(signed_info, signature_value, x509)` — **uma** implementação da montagem do
  `<SignedInfo>`/`<Signature>` (RSA-SHA1), antes **triplicada**: inline na emissão (string) e
  duplicada em `cancelar`/`manifestacao` (quick-xml Writer).
- Emissão, cancelamento e manifestação passam a delegar a esse módulo. A diferença entre eles é
  só o `reference_uri` (`#NFe{chave}` na emissão; `#ID{tpEvento}{chave}{nSeq}` nos eventos).
- **Saída byte-a-byte idêntica** à anterior — provado por golden `insta`: os snapshots de
  `cancelar` (capturados da impl Writer) e de `manifestacao` continuam passando contra a nova
  impl string; novos golden em `interno::assinatura` cobrem ambas as formas de `reference_uri`.
  Nenhuma alteração no XML assinado enviado à SEFAZ.

### A3a — transporte SOAP unificado + seam de testes (interno)

#### Changed (interno — sem mudança de API pública)
- Novo `interno::transporte` com o trait `SoapTransport` (injetável — permite transporte falso
  nos testes offline) e a implementação real `MtlsTransport` (mTLS + POST SOAP + leitura do
  corpo). Emissão, cancelamento e consulta de status passam a usá-lo, eliminando a duplicação
  de `Cert::from_pfx` + client + `post(...)` espalhada nesses módulos.
- Removido o método `WebService::send` (agora coberto por `MtlsTransport`); `WebService::client`
  permanece como construtor do cliente mTLS.
- Efeito observável: em falha HTTP (status ≠ 2xx) o erro agora é `DfeError::Webservice` uniforme
  (antes cada módulo tratava — ou ignorava — o status de forma diferente). O caminho de sucesso
  é idêntico (mesmo corpo de resposta, mesmos headers `Content-Type`/`Content-Length`).
- **`manifestacao` e `distribuicao` ainda não migraram** (acoplados ao logging de auditoria §1.9
  e, no caso da distribuição, ao tratamento de gzip) — convergem numa fase posterior.

### A2 — erros sem panic no fluxo de request

**Contexto:** numa lib consumida por um servidor HTTP multi-thread (ex.: Actix), um
`panic!`/`unwrap` derruba o worker. Todo caminho de operação deve retornar `Result<_, DfeError>`.

#### Changed (interno — sem mudança de API pública)
- `interno::chave_acesso::gerar_chave_acesso` passou de `-> ChaveAcesso` para
  `-> Result<ChaveAcesso>`: o erro de cálculo do DV (ex.: caractere inválido) agora propaga
  em vez de `panic!`. (`interno` é privado → não afeta consumidores.)
- `cancelar` e `manifestacao`: `regex::new(...).unwrap()` e `str::from_utf8(...).unwrap()` na
  resposta da SEFAZ viraram `map_err`/`?` — resposta malformada agora vira `DfeError::Xml`,
  não crash.
- `interno::cleaner` (fluxo de assinatura) e `interno::ws` (parse do `webservices.json`
  embutido): fallback defensivo — nunca panicam no runtime; a validade do recurso embutido
  segue coberta por teste.
- `emissao::det`: `Option::unwrap` guardado por `is_some()` trocado por `unwrap_or_else`
  (limpeza, sem mudança de comportamento).

> Nota: as ocorrências de `unwrap`/`panic!` restantes na crate estão **todas em código de
> teste** (`#[cfg(test)]`/`#[test]`), fora do binário de produção.

### A1 — fim dos side-effects de CWD (bug de concorrência)

**Contexto:** a crate é consumida por um servidor HTTP multi-thread (ex.: Actix). Ela gravava
arquivos de nome fixo no diretório de trabalho **durante** a operação (`nfe_request.xml`,
`nfe_request_envelope.xml`, `nfe_response.xml`, `cancelar.xml`, `cancelar_response.xml`,
`inf_evento.xml`, `flag_autorizacao.env`) — duas operações concorrentes sobrescreviam o
mesmo arquivo (corrida de dados) e poluíam o CWD do servidor.

#### Added
- `dfe::EmissaoResponse` (`emissao::Response`) ganhou dois campos públicos, **aditivos** e
  anotados com `#[serde(default)]`:
  - `send_xml: String` — envelope SOAP efetivamente enviado à SEFAZ.
  - `receive_xml: String` — corpo cru da resposta da SEFAZ.
  Substituem os antigos dumps de debug em disco; úteis para auditoria/diagnóstico.

#### Changed
- Emissão (`emissao`) e cancelamento (`cancelar`) **não escrevem mais** arquivos de debug
  no CWD. Os XMLs continuam disponíveis: emissão via `xml`/`send_xml`/`receive_xml`;
  cancelamento via os já existentes `send_xml`/`receive_xml` da sua resposta.
- `FlagAutorizacao::start()` deixou de ler/gravar `flag_autorizacao.env` no CWD. O mecanismo
  era vestigial (nada no fluxo produzia estado ≠ `Ready`); agora retorna `Ready` em memória,
  preservando o comportamento observável (a emissão só prossegue com `Ready`).

#### Impacto nos consumidores
- **Nenhum breaking change.** O consumidor só lê a `Response`; os campos novos são aditivos.
  Some apenas o lixo `*.xml`/`flag_autorizacao.env` antes gerado no CWD do processo consumidor.
- **Fora de escopo (adiado):** o logging de auditoria em subpasta de `manifestacao` e
  `distribuicao` (§1.9) continua como está — migra para logger opcional (`interno::obs`) numa
  fase posterior.

### Interno — fundação de testes (Fase 0, sem impacto no produto)
- Estrutura de testes em camadas (`tests/` + fixtures + golden `insta`; camada `sefaz-live`
  gated por feature). Blindagem de segredos (`.gitignore`, `env.example`, `.env`).
  Dev-deps `insta`/`dotenvy` (não entram no binário do consumidor). Removido o alvo binário
  de teste `src/bin`.
- Camada 4 (0.4): golden `insta` inline das funções puras de montagem do XML de evento
  (`cancelar`) e da limpeza C14N (`interno::cleaner`) — rede de segurança para a A3b. Feature
  `filters` do `insta` (redação de campos voláteis). Sem impacto no produto.
