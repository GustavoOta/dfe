# Changelog

Registro de alterações da crate `dfe`. Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/).
Fases do refactor de arquitetura em `planning/ARQUITETURA_REFACTOR.md`.

## [Unreleased]

## [0.8.2] — 2026-11-23

### Changed
- **README reescrito como vitrine** (NF-e, NFC-e e NFS-e Nacional): curto, com o detalhe nas
  páginas de `docs/` (perfis de *features* em `instalacao.md`, amostras em `danfe.md`, regras do
  builder em `nfse.md`). O exemplo do DANFE no README não compilava (`xml()` recebe `&str` e
  `build()` devolve `Result<String, String>`); os exemplos em Rust do README agora são **doctests**
  (`ReadmeDoctests`, `cargo test --features nfse`). A página do docs.rs passa a documentar a
  feature `nfse` e lista inutilização e consulta de situação.
- **Validação XSD no Pacote de Liberação PL_010h v1.03** (13/11/2026), no lugar do PL_010d
  (`src/schemas_pl010h/`). O pacote traz as NT 2025.002 v1.50, 2026.002/003 (`tpImp=6`),
  2026.005 (IBS/CBS do Simples: `cAtivSN`, `tpRegimeIBSCBS`, `gTribSN`), 2026.006 (split,
  `gPgtoVinc`), 2026.007 (IE do emitente opcional) e 2026.008 (`vProdLiq`). Os campos novos
  são opcionais: a nota de hoje, sem eles, passa igual. Validado em homologação na SEFAZ-SP
  (18-11-2026): NF-e 55, NFC-e 65 e NFC-e com destinatário CNPJ autorizadas (100). A crate
  ainda **não emite** nenhum campo novo. O PL_010d fica em `src/schemas_nt2026_004/` para
  rollback. Os XSD extraídos vão para `dfe_schemas_nfe_PL_010h` no temp, uma pasta por pacote.
  - `leiauteNFe`: só campos opcionais novos (`cIndOp`, `tpRegimeIBSCBS`, `ISUFEmit`,
    `vUnComLiq`/`vProdLiq`, `cAtivSN`, `vProdLiqTot`, `gPgtoVinc`, alternativa
    `ICMSPrevistoPagtoAntecip`) e restrições afrouxadas (`tpImp` aceita `6`, `emit/IE`
    opcional, `vNFTot` aceita `0`, até 5 `reboque`, `xMsg` com 255).
  - `DFeTiposBasicos`: `gIBSCBS` vira `TCIBS_NFe` (+ `gALCZFMCBS`), entra `gTribSN`, monofasia
    reorganizada, `pISEspec` → `adRemIS`. Nada disso muda o `IBSCBS` que a crate monta hoje.
  - Testes: pacote embutido, fixtures 55/65 sem campo novo, `tpImp=6`, `cAtivSN` no lugar certo
    e fora dele.
  - Detalhe completo, hashes, rollback e roteiro do próximo pacote:
    [`docs/xsd_pacotes.md`](docs/xsd_pacotes.md).

### Known issues
- **NFS-e: produção restrita responde 403 do IIS a todo pedido** (23-11-2026), sem pedir o
  certificado de cliente; a produção, com o mesmo certificado e rede, atende. Detalhe e diagnóstico
  em [`docs/nfse.md`](docs/nfse.md#problema-conhecido-produção-restrita-respondendo-403-a-todo-pedido).
- **CC-e em homologação na SEFAZ-SP volta `999 — Erro não catalogado`** (18-11-2026), no ciclo
  `live_ciclo_nfe_55_homologacao`. Rodando com o PL_010d dá o mesmo resultado, e o evento não passa
  pelo validador de NF-e. Ainda sem causa identificada.

### Added
- **NFS-e Nacional: assinatura, envio e consulta na SEFIN** (feature `nfse`, que passa a ligar
  `validacao-xsd`). `NfseClient::builder().ambiente().cert().build()` com:
  - `assinar(&DpsGerada) -> DpsAssinada`: XMLDSig no perfil do Manual Integrado do SN NFS-e
    §6.1.4 (o mesmo da NF-e: RSA-SHA1, SHA-1, C14N, enveloped, `Reference URI="#DPS…"`),
    reusando a assinatura CAPI/OpenSSL da crate, e conferência no XSD do ambiente.
    `DpsAssinada::from_xml` reabre a DPS guardada e recusa XML alterado depois de assinado.
  - `transmitir` (`POST /nfse`, `dpsXmlGZipB64`): `Autorizada` (chave, XML, alertas) ou
    `Rejeitada` (erros, nas grafias `codigo`/`Codigo`); `ErroEnvio::Local` quando nada saiu e
    `ErroEnvio::Indeterminado` quando a DPS pode ter sido processada (timeout, 5xx, resposta
    ilegível).
  - `consultar_dps` (`GET /dps/{id}`: `Encontrada`/`NaoEncontrada`) e `consultar_nfse`
    (`GET /nfse/{chave}`). Descompressão com teto de 8 MiB.
  - Conferência independente da DPS assinada do teste (`src/nfse/snapshots/`): digest pelo C14N
    do lxml e do `xml.etree` (Python), assinatura por `openssl dgst -sha1 -verify` — `Verified OK`.
    Mesmo XML no Windows (CAPI) e no Linux (OpenSSL).
  - Ciclo real opt-in em produção restrita: `tests/nfse_live.rs` (`sefaz-live` + `nfse`,
    `DFE_TEST_NFSE_*` no `.env`, certificado e documento do prestador próprios, separados dos
    `DFE_TEST_CERT_*` da NF-e; serviço de R$ 1,00), ambiente fixo em `common::HOMOLOGACAO_NFSE`.
  - Respostas lidas em camelCase e em PascalCase (`ChaveAcesso`, `Erros`, `Alertas`…), a
    grafia do Swagger oficial do ADN.
  - Contrato real conferido na SEFIN de **produção** (só leitura, 23-11-2026): base
    `…/SefinNacional`, `GET /dps/{id}` de DPS inexistente → HTTP 404 com `erro.codigo = E2404`
    (travado em teste com o corpo literal).
  - [`docs/nfse.md`](docs/nfse.md): ambientes, convênio do município por ambiente, exceção do MEI
    (Anexo I) e o **403 da produção restrita** com o diagnóstico passo a passo.

- **XSDs da DPS embutidos** (`src/nfse/schemas/`, antes só em `tests/fixtures/nfse/`). O de
  produção tem `TSSerieDPS` corrigido (`^`/`$` literais reprovavam toda série no libxml2);
  hashes e motivo no `README.md` da pasta.
- **Correção no `NfseBuilder` (DPS com aspas ou CR no texto)**: escrevia `&quot;`/`&apos;` e CR,
  divergindo da forma canônica sobre a qual a SEFIN confere o digest (rejeição E0714). Agora só
  `&`, `<` e `>` viram entidade e a quebra de linha sai como LF.
- **DANFE A4 no modelo da Reforma Tributária** (NT 2026.010, vigente a partir de 01/12/2026):
  CRT no quadro do emitente; bloco "TOTAL DO IBS / CBS / IS" (com a linha da monofasia só
  quando há `gMono`); sublinha por item com cClassTrib, base, alíquotas (efetiva com `gRed`) e
  valores de IBS UF, IBS Município, CBS e IS. Campo ausente no XML sai em branco. O
  `xml_extractor` passou a ler `det/imposto/IBSCBS`, `det/imposto/IS` e `IBSCBSTot/gMono`.
- **Amostras do DANFE A4** (`tests/danfe_a4_amostras.rs`): seis PDFs de NF-e fictícias para
  conferir o leiaute sem certificado — `cargo test --test danfe_a4_amostras -- --nocapture`;
  os arquivos ficam em `target/tmp/danfe-a4/` (ou em `DFE_DANFE_SAIDA`).

### Fixed
- As divisórias verticais do cabeçalho do DANFE A4 atravessavam a faixa de CNPJ/IE/datas e
  cortavam os rótulos ao meio; agora param no topo da faixa.

### Added
- **NF-e de saída não referencia mais NFC-e/CF-e** (Ajuste SINIEF 32/2025, adiado para
  14/12/2026 pelo Ajuste SINIEF 29/2026; NT 2026.002 v1.11, rejeições 679 e 375). A montagem
  recusa com `DfeError::Validacao` a NF-e 55 de saída (exceto complementar e devolução) com
  referência de modelo 65 ou 59 — em homologação já, em produção a partir de 14/12/2026. Funções
  públicas para recusar antes: `chave_e_de_cupom`, `finalidade_sujeita_a_vedacao`,
  `vedacao_referencia_cupom_ativa`, `MSG_REFERENCIA_CUPOM_VEDADA`.

### Fixed
- **NFC-e com destinatário CNPJ rejeitada com 181** (NT 2026.007, regra 12E02-10, em produção
  desde 16/11/2026). A SEFAZ procura o CNPJ na LCC-RFB pela chave UF + CNPJ do destinatário, e
  o `<dest>` da NFC-e saía sem `<enderDest>` — sem UF, até CNPJ ativo voltava 181. Agora o
  `<enderDest>` sai na NFC-e quando o `Dest` traz o endereço completo (`x_lgr`, `nro`,
  `x_bairro`, `c_mun`, `x_mun`, `uf`; `cep`/`fone` opcionais). Endereço incompleto continua
  fora, como antes. Validado em homologação na SEFAZ-SP: CNPJ ativo de SP passou de 181 para
  100, e destinatário do PR (emitente de SP, `idDest = 1`) também autoriza.

## [0.8.0] — 2026-11-15

### Linux: certificado, assinatura e mTLS

> **Atualizando de 0.7.x:** nada muda na API nem no Windows. O `Cargo.lock` do seu projeto ganha
> `openssl` na lista de dependências da `dfe` (dependência só de Linux). No Linux, o build pede
> `pkg-config`, `libssl-dev` e `libxml2-dev` — ver
> [docs/certificado-e-plataformas.md](docs/certificado-e-plataformas.md#compilando-no-linux).

#### Added
- Suporte a Linux: `CertInfo`, extração do certificado e assinatura RSA-SHA1 pelo **OpenSSL do
  sistema** (`openssl` só em `cfg(not(target_os = "windows"))`), com o provider `legacy` carregado
  pela crate para os PFX ICP-Brasil em RC2-40/3DES. Resultado idêntico ao CAPI do Windows (titular
  só com o CN, datas UTC, assinatura byte a byte). No Windows nada muda.
- Cadeia ICP-Brasil (`src/data/icp_brasil.pem`, ITI) somada à lista de ACs do sistema nos clientes
  mTLS fora do Windows — sem ela o Linux não confia na maioria dos servidores da SEFAZ.
  Regeneração: `src/data/icp_brasil_gerar.sh`.
- Validado em homologação na SEFAZ-SP a partir do Linux, com A1 real: NF-e (100, CC-e 135,
  cancelamento 135) e NFC-e (100, cancelamento 135). Detalhes:
  [docs/certificado-e-plataformas.md](docs/certificado-e-plataformas.md).

#### Fixed
- **QR Code da NFC-e com ID do CSC com zeros à esquerda** (`000001`, como o portal da SEFAZ mostra):
  saía assim na URL e a NFC-e era recusada pela validação local (o XSD e o manual exigem `1`). O
  `cIdToken` agora vai sem os zeros, na URL e no hash; ID com letra ou mais de 6 dígitos vira
  `DfeError::Validacao`. Quem já informava `1` não vê diferença.
- `EscPosNFCeBuilder`: o exemplo de `print()` na documentação compilava só no Windows.

#### Changed
- DANFE NFC-e 80 mm: QR Code centralizado de 55 mm para 38,5 mm (o lateral segue em 33 mm) e a linha
  do número diz "NFC-e" (dizia "NF-e"); no ESC/POS, sem a linha em branco entre a URL de consulta e
  a chave.

#### Testes
- PFX sintéticos (AES e RC2/3DES) em `tests/fixtures/cert/`, com comparação Windows × Linux.
- Testes que falam com a SEFAZ **sempre em homologação**, fixado no código (`tests/common::HOMOLOGACAO`
  + `tests/homologacao_travada.rs`); o ambiente saiu do `.env`. Seis testes de manifestação usavam
  produção.
- Ciclos completos em homologação (`live_ciclo_nfe_55_homologacao`, `live_ciclo_nfce_65_homologacao`,
  `live_inutilizacao_homologacao`) e prova de TLS com todos os servidores (`live_tls`).

## Até 0.7.3

Entradas anteriores ao corte por versão (todas publicadas até a 0.7.3).

### IBS/CBS sem o grupo de valores, por CST (opt-in)

#### Added
- `NFeBuilder::ibscbs_sem_grupo(&["410"])`: itens com esses CSTs saem só com `CST` + `cClassTrib`,
  sem `gIBSCBS` (opcional no XSD, `xs:choice minOccurs="0"`) — ex.: 410/410014, fornecimento de
  produtor rural não contribuinte na nota de entrada da cooperativa. Não somam no `IBSCBSTot`.
  Sem a chamada, o XML não muda (consumidores atuais intocados).

### Autorização com alerta — cStat 120 (NT 2026.002)

#### Added
- `dfe::cstat_autorizado` / `cstat_autorizado_str` / `CSTAT_AUTORIZADO` (módulo `autorizacao`):
  regra única de "nota autorizada" = 100, 120 (com alerta) ou 150 (fora do prazo).
- `InfProt::alertas: Vec<Alerta>` (`dfe::AlertaAutorizacao`) com os pares `cMsg`/`xMsg` (PR13) do
  protocolo, até 5. Fora do JSON quando vazio — contrato da autorização comum não muda.

#### Fixed
- cStat 120 era tratado como rejeição: o `nfeProc` não era montado e o XML saía sem protocolo,
  embora a nota estivesse autorizada na SEFAZ (produção desde 05/10/2026, NFC-e). O `protNFe`
  montado agora reproduz os alertas.

### ICMS de todos os CST do leiaute

#### Added
- Grupos ICMS02/15/53/61 (monofásico de combustíveis, NT 2023.001), ICMSPart (partilha, pela
  `uf_st`) e ICMSST (repasse de ST retido, pelo `v_bcst_dest`).
- Modalidades `modBC` 0/1/2 (`p_mva_proprio`, `v_pauta`) e `modBCST` 0/1/2/3/5 (`v_pauta_st`) —
  antes eram erro de montagem. `icms_de_parametros` recebe a quantidade tributável.
- `motDesICMSST`/`vICMSSTDeson` (10/70/90), `cBenefRBC` e FCP diferido (51/90), diferimento no
  90, ICMS efetivo e FCP-ST retido (60/500/ICMSST).
- `Det::cred_presumido` (`prod/gCred`, até 4) e `Det::comb` (`prod/comb`, obrigatório nos CST
  monofásicos: sem ANP é erro de montagem).
- ICMSTot: `qBCMono`…`vICMSMonoRet` (só com item monofásico); `vFCPSTRet` soma os itens;
  `vNF` soma `vICMSMonoReten` (regra 610); ICMSPart entra em vBC/vICMS/vBCST/vST.

#### Fixed
- DANFE A4 lia pICMS/vICMS só do ICMS00: agora de qualquer grupo (`xml_extractor::ICMS::grupo`).
- `validar_fragmento` (testes) inclui todos os tipos simples do leiaute (o `<prod>` usa TGuid).

### PIS/COFINS: qualquer CST do leiaute

#### Added
- `PisCofinsParametros` + `Pis/Cofins::por_parametros`: a crate escolhe o grupo pelo CST e
  calcula (01/02 Aliq, 03 Qtde, 04–09 NT, 49–99 Outr por % ou por R$/unidade ou zerado).
- Variantes `OutrAliq`/`OutrQtde`; `St` ganhou `ind_soma` (indSomaPISST/indSomaCOFINSST).

#### Fixed
- 49–99 com valor não existiam (`PISOutr` sem vBC/pPIS); ICMSTot/vPIS e vCOFINS agora somam o
  grupo Outr.
- `PISST`/`COFINSST` saíam dentro de `<PIS>`/`<COFINS>` (inválido): agora grupos irmãos, fora do
  vPIS/vCOFINS do total e somados ao vNF só com indSoma = 1 (NT 2020.005, regra 610).

### PIS/COFINS sem valor no grupo do CST

#### Added
- `Pis::nao_tributado(cst)` / `Cofins::nao_tributado(cst)` e `CST_PIS_COFINS_NT` (04–09).

#### Fixed
- **Rejeição de schema** "CST '07' is not an element of the set {'49', …, '99'}": os
  conversores do PDV e do gravisServer mandavam 04–09 em `Outr`. O grupo agora sai pelo CST na
  serialização (04–09 em PISNT/COFINSNT, 49–99 em PISOutr/COFINSOutr), qualquer que seja a
  variante. `Pis::Nt` com 49–99 leva o próprio CST ao PISOutr — o `Pis::Outr` saía sempre 99.

### Arredondamento fiscal único

#### Added
- **Módulo público `arredondamento`** com `arred`, `arred2`, `arred_decimal`, `fmt_dec` e
  `fmt_dec_ate`, exportados na raiz (`dfe::arred2`). Arredonda meio para cima sobre o valor
  decimal. É a regra do ecossistema: gravisServer, PDV e
  retaguarda reusam, e o JS tem o espelho `arredondamento.js`.

#### Fixed
- **Centavo do item diferente do total** (rejeições 531/532/602/603): o item saía por
  `format!("{:.2}")`, que opera no binário e desempata para o par, e o total somava o f64 cru.
  Com dois itens de 0.125, o XML imprimia 0.12 + 0.12 nos itens e 0.25 no total. Agora todo
  número passa por `fmt_dec` e o total soma os valores **impressos** (`arred2`).
- **Rateio de desconto/acréscimo** desempata para cima (antes era o `round_dp` bancário).
- **`vUnCom`/`vUnTrib` perdiam casas**: 602,6097 saía "602.61" (rejeição 629 com quantidade
  maior). Agora saem com 2 a 10 casas, `qCom`/`qTrib` com 3 a 4, e `qBCProd` do PIS/COFINS por
  quantidade com 4.

### ICMS por parâmetros do cadastro + cBenef

#### Added
- **`Icms::Parametros(IcmsParametros)`**: o consumidor manda CST/CSOSN, percentuais e
  modalidades, e a crate escolhe o grupo e calcula vBC, vICMS, ST, FCP, FCP-ST e desoneração
  sobre a base do item (vProd + frete + acréscimo − desconto, já rateados). Cobre os CST 00, 10,
  20, 30, 40, 41, 50, 51, 60, 70 e 90 e os CSOSN 101, 102, 103, 201, 202, 203, 300, 400, 500 e
  900 (grupos `ICMSSN201`/`ICMSSN202` novos). Um campo exigido que chega vazio vira
  `DfeError::Validacao`. Só as modalidades de valor da operação são calculadas: `mod_bc` 3 e
  `mod_bcst` 4 (MVA) ou 6. Plano: `ICMS_CADASTRO_PLANO.md`.
- **`Det::c_benef`** → `<cBenef>` entre `CEST` e `CFOP`. Vazio é omitido (rejeição 930 nas UFs
  que exigem).

#### Changed
- **Totais**: `vBCST`, `vST`, `vFCP` e `vFCPST` passam a sair da soma dos itens (o valor de
  `Total` é somado a ela). O `vNF` agora soma `vST + vFCPST` e tira o `vICMSDeson` dos itens com
  `ind_deduz_deson = 1` (regra 610, NT 2023.004).

### IBS/CBS calculado pela crate + redução de alíquota

#### Added
- **`IbsCbs::calcular`**: com `true`, a crate calcula
  `vBC = vProd + frete + seg + outro − desc − vPIS − vCOFINS − vICMS − vFCP` do item e os valores
  `vBC × alíquota` (NT 2025.002-RTC, rejeição 1115). Com `false`, a crate usa os valores do
  chamador, como antes.
- **`IbsCbs::p_red_ibs` / `p_red_cbs`** e **`IbsCbs::cst_exige_reducao`**: montam `gRed`
  (`pRedAliq`, `pAliqEfet`) em `gIBSUF`, `gIBSMun` e `gCBS` só nos CST 200 e 515 (rejeições
  1033/1074). Em outro CST a redução recebida é ignorada, porque lá o `gRed` é a rejeição 1032.

#### Changed
- `GRed.p_red_aliq` / `p_aliq_efet` (entidade interna) passam de `Decimal` para `String`
  formatada com 4 casas.

#### Fixed
- **`vCredPres` / `vCredPresCondSus` vazios no `IBSCBSTot`**: saíam `<vCredPres/>` e reprovavam
  no pattern do XSD. Agora saem `0.00`.

### Emitente

#### Fixed
- **`xNome` / `xFant` do emitente com mais de 60 caracteres** reprovavam no XSD. Agora são
  cortados nos 60, contando caracteres e não bytes.

### Testes
- `interno::validation::validar_fragmento` (só `cfg(test)`) valida um grupo isolado (ex.:
  `<ICMS>`, `<IBSCBS>`) contra o `leiauteNFe` embutido, travando a ordem e o formato dos campos.

### Status do serviço por modelo

#### Added
- **`NFeService::modelo(55 | 65)`**: a consulta de status passa a escolher o webservice do modelo.
  Antes era sempre o da NF-e (55) — em UFs com host próprio da NFC-e (SP, por exemplo) o PDV decidia
  a contingência da NFC-e olhando o serviço da NF-e. Padrão continua 55; outro modelo é recusado.

### Impressão da NFC-e em contingência off-line

#### Fixed
- **`DanfeBuilder` e `EscPosNFCeBuilder` não imprimiam a NFC-e em contingência** (`tpEmis=9`):
  a nota é um `<NFe>` assinado, sem `nfeProc`/`protNFe` (a SEFAZ ainda não autorizou), e a leitura
  exigia `nfeProc` — `missing field @versao`. Achado no primeiro teste em campo (2026-09-21).

#### Added
- **`XmlExtractor::nfe_proc_para_impressao`** (e `_from_file`): lê o `nfeProc` como antes e, na
  falta dele, aceita o `<NFe>` **só** se for mod 65 com `tpEmis=9`, devolvendo `prot_nfe.inf_prot =
  None`. Outro `<NFe>` sem protocolo segue recusado. `nfe_proc_from_string` continua estrito.

#### Changed
- **Aviso de contingência consciente do protocolo** (`EscPosNFCeBuilder` e DANFE NFC-e 80mm):
  com `tpEmis=9` e **sem** protocolo continua "PENDENTE DE AUTORIZAÇÃO PELA SEFAZ"; **com**
  protocolo (2ª via depois da transmissão) sai "AUTORIZADA PELA SEFAZ". O `tpEmis` faz parte da
  chave e continua 9 depois da autorização — antes, a reimpressão da nota já autorizada dizia
  "pendente" ao lado do número do protocolo.

### Inutilização de numeração + emissão em duas etapas

#### Added
- **`InutilizacaoBuilder`** (`inutNFe`, serviço `NFeInutilizacao4`): inutiliza uma faixa de
  numeração de NF-e/NFC-e. Validação fail-fast (UF, CNPJ, modelo, faixa, justificativa 15–255,
  ano de 2 dígitos) antes de qualquer I/O; endpoints já existiam no `webservices.json` para as 27
  UFs. `cStat 102` é o único sucesso. Tipos em `tipos::inutilizacao`.
- **`NFeBuilder::assinar() -> NFeAssinada`** e **`NFeAssinada::transmitir()`**: as duas metades da
  emissão, separadas. `NFeAssinada` expõe `chave` e `xml`, o que permite ao consumidor registrar a
  chave de uma tentativa **antes** de saber o resultado do envio — quando a transmissão falha por
  rede, a nota pode ter sido autorizada mesmo assim, e a chave é o que permite conferir depois com
  o `ConsultaSituacaoBuilder`.

#### Changed
- `emitir()` agora é a composição de `assinar()` + `transmitir()`, e `gerar_xml()` é atalho para
  `assinar().await?.xml`. **Mesmo comportamento externo**, com uma diferença de ordem: a checagem
  da flag de autorização passou a ocorrer depois da montagem/assinatura (antes era a primeira
  coisa). Sem efeito prático — a flag hoje retorna sempre `Ready` e nada é gravado em disco antes
  do envio.
- O bloco de validação de campos obrigatórios do builder, que estava duplicado em `gerar_xml()` e
  `emitir()`, virou um helper único (`montar_interno`).


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
