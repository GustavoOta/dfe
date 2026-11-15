# Certificado digital e plataformas

Como a `dfe` abre o certificado A1 (`.pfx`), assina o XML e conversa com a SEFAZ em cada
sistema operacional — e por que cada escolha foi feita.

- [Resumo por plataforma](#resumo-por-plataforma)
- [Os dois usos do certificado](#os-dois-usos-do-certificado)
- [Windows — CAPI](#windows--capi)
- [Linux — OpenSSL do sistema](#linux--openssl-do-sistema)
- [Provider `legacy` do OpenSSL 3 — o motivo](#provider-legacy-do-openssl-3--o-motivo)
- [Cadeia ICP-Brasil embutida (TLS com a SEFAZ no Linux)](#cadeia-icp-brasil-embutida-tls-com-a-sefaz-no-linux)
- [Mesmo resultado nos dois sistemas](#mesmo-resultado-nos-dois-sistemas)
- [Compilando no Linux](#compilando-no-linux)
- [macOS](#macos)
- [Como isso é testado](#como-isso-é-testado)
- [Limitações conhecidas](#limitações-conhecidas)

---

## Resumo por plataforma

| | Windows | Linux | macOS |
|---|:--:|:--:|:--:|
| Ler o certificado (`CertInfo`) | ✅ CAPI | ✅ OpenSSL | ⚠️ não testado |
| Assinar o XML (RSA-SHA1) | ✅ CAPI | ✅ OpenSSL | ⚠️ não testado |
| TLS mútuo com a SEFAZ (mTLS) | ✅ SChannel | ✅ OpenSSL + cadeia ICP-Brasil | ⚠️ não testado |
| PFX legado (RC2/3DES) | ✅ nativo | ✅ provider `legacy` | ⚠️ não testado |
| Validação XSD (`libxml2`) | ✅ estático (vcpkg) | ✅ `libxml2` do sistema | ⚠️ não testado |
| DANFE em PDF, ESC/POS (montar bytes) | ✅ | ✅ | ⚠️ não testado |
| `EscPosNFCeBuilder::print()` (impressão RAW) | ✅ | ❌ (mande os bytes do `build()`) | ❌ |

Linux validado no Ubuntu 26.04 com OpenSSL 3.5, inclusive **em homologação com um A1 ICP-Brasil
real** (PFX legado RC2-40 + 3DES) na SEFAZ-SP: status 107; NF-e autorizada (100), carta de correção
e cancelamento registrados (135); NFC-e autorizada (100, com QR Code) e cancelada (135) — mesmos
resultados do Windows. A API pública é **a mesma** em
todos os sistemas: a diferença fica toda dentro da crate, por `#[cfg(target_os = "windows")]`.

---

## Os dois usos do certificado

O mesmo `.pfx` é usado de duas formas independentes:

1. **Assinatura do XML (XML-DSig).** A chave privada assina o `SignedInfo` com RSA-SHA1
   (padding PKCS#1 v1.5), e o certificado vai em base64 no `<X509Certificate>` do `KeyInfo`.
   Vale para emissão, cancelamento, carta de correção, inutilização, manifestação e o QR Code
   da NFC-e em contingência off-line.
2. **TLS mútuo (mTLS).** A SEFAZ exige o certificado também na conexão HTTPS. Aqui ele vira um
   `reqwest::Identity`, e o TLS é feito pelo `native-tls`.

O certificado **nunca é instalado** no sistema nem guardado em cache: o arquivo é lido a cada
operação e descartado no fim.

---

## Windows — CAPI

No Windows tudo passa pela **CryptoAPI do próprio sistema** (crate `windows-sys`), sem OpenSSL:

- **Abrir o PFX:** `PFXImportCertStore` em memória, a partir dos bytes do arquivo.
- **Achar o certificado da empresa:** o PFX traz a empresa **e** a cadeia de ACs, e a ordem
  não é garantida. O certificado certo é o único com chave privada associada
  (`CERT_KEY_PROV_INFO_PROP_ID` ou `CERT_KEY_CONTEXT_PROP_ID`).
- **Assinar:** a chave pode vir como CNG ou CAPI legado; a crate usa `NCryptSignHash` no
  primeiro caso e `CryptSignHashW` no segundo (que devolve a assinatura invertida, desinvertida
  em seguida).
- **Ler os dados:** `CertGetNameStringW` para titular e emissor, `NotBefore`/`NotAfter` para a
  validade. O keyset criado na importação é apagado logo depois.
- **mTLS:** `native-tls` usa o **SChannel**, com o repositório de ACs do Windows — que já
  conhece as raízes ICP-Brasil e baixa sozinho as ACs intermediárias que o servidor não manda.

O CAPI lê nativamente os algoritmos antigos dos A1 ICP-Brasil (RC2-40, 3DES) em qualquer versão
do Windows. Por isso o Windows não precisa de nada além do `libxml2` (ver
[Instalação](instalacao.md)).

---

## Linux — OpenSSL do sistema

Fora do Windows não existe CAPI. A crate usa o **OpenSSL do sistema** pela crate `openssl`,
declarada **só** para esses alvos:

```toml
[target.'cfg(not(target_os = "windows"))'.dependencies]
openssl = "0.10"
```

No Windows ela nem entra no grafo de dependências. No Linux ela já vinha pelo `native-tls`; a
`dfe` só passa a usá-la diretamente.

O fluxo (`src/interno/cert_openssl.rs`):

1. Carrega o provider `legacy` (ver a seção seguinte).
2. `Pkcs12::from_der(bytes).parse2(senha)` → chave privada **e o certificado ligado a ela**. A
   AC da cadeia vem à parte e é descartada, que é o equivalente da busca pelo certificado da
   empresa no Windows.
3. Conforme a operação:
   - **dados:** CN do titular e do emissor, validade em UTC;
   - **certificado:** DER do certificado da empresa;
   - **assinatura:** `Signer` com SHA-1 e padding PKCS#1 v1.5.
4. **mTLS:** o mesmo `reqwest::Identity::from_pkcs12_der` do Windows, que no Linux também passa
   pelo OpenSSL (e por isso também precisa do provider `legacy`).

Por que o OpenSSL **do sistema** e não uma cópia embutida (`vendored`): o provider `legacy` já
vem no pacote do OpenSSL das distribuições (`ossl-modules/legacy.so`), e as correções de
segurança chegam pelo gerenciador de pacotes, sem recompilar a aplicação.

---

## Provider `legacy` do OpenSSL 3 — o motivo

Um `.pfx` é um arquivo cifrado com a senha. Os A1 ICP-Brasil — especialmente os exportados
pelo Windows e os emitidos há mais tempo — usam os algoritmos do padrão PKCS#12 antigo:

| Parte do PFX | Algoritmo típico do A1 ICP-Brasil | PFX moderno (OpenSSL 3) |
|---|---|---|
| Certificados | `pbeWithSHA1And40BitRC2-CBC` (RC2-40) | AES-256-CBC (PBES2/PBKDF2) |
| Chave privada | `pbeWithSHA1And3-KeyTripleDES-CBC` (3DES) | AES-256-CBC (PBES2/PBKDF2) |
| MAC | SHA-1 | SHA-256 |

O **OpenSSL 3 tirou o RC2 do conjunto padrão** e o pôs no provider `legacy`, que não é carregado
sozinho. Sem ele, abrir um PFX desses falha **com o mesmo erro de senha errada** — o pior tipo de
falha, porque manda o usuário conferir uma senha que está certa.

Por isso a crate:

- carrega o `legacy` **uma vez por processo** (`OnceLock`) e o mantém carregado enquanto o
  programa roda (descarregar no meio derrubaria o PFX seguinte);
- carrega com `retain_fallbacks = true`, para manter o provider `default` (carregar um provider
  à mão desligaria o padrão);
- carrega **antes** de abrir o PFX para assinar **e** antes de montar a identidade do mTLS —
  como vale para o processo inteiro, cobre também o `native-tls`;
- se o sistema não tiver o módulo (OpenSSL compilado sem ele), o PFX moderno continua abrindo e
  o legado falha com a mensagem *"Senha inválida ou PFX corrompido (o OpenSSL desta máquina não
  tem o provider legacy, necessário para PFX em RC2/3DES)"*.

Nada disso muda a configuração global do OpenSSL nem do Windows.

---

## Cadeia ICP-Brasil embutida (TLS com a SEFAZ no Linux)

Além de **apresentar** o certificado, o cliente precisa **confiar** no certificado do servidor
da SEFAZ. No Linux isso não funciona com a lista de ACs do sistema:

- a maioria dos servidores da SEFAZ (SP, PR, RS/SVRS, BA, GO, MS, MT, AM, PE…) usa certificado
  **ICP-Brasil** (Raiz Brasileira v10, AC Soluti SSL EV G4, AC Certisign SSL EV G4,
  Serpro SSLv1);
- a lista de ACs das distribuições Linux (a da Mozilla) **não tem** nenhuma ICP-Brasil;
- vários servidores **não mandam a AC intermediária**. O Windows a baixa sozinho; o OpenSSL não.

Medido em 15-11-2026: com a lista do sistema, **34 de 43** servidores da tabela de webservices
falhavam no Linux (`unable to get local issuer certificate`). Com a cadeia ICP-Brasil somada,
**todos os que respondem** passam.

Como a crate resolve:

- `src/data/icp_brasil.pem` traz a cadeia ICP-Brasil publicada pelo ITI (raízes e ACs
  intermediárias, só certificados válidos), embutida no binário com `include_bytes!` **só fora do
  Windows**;
- todo cliente HTTP que fala com a SEFAZ nasce em `WebService::builder()`, que **soma** essa
  cadeia à lista do sistema (`add_root_certificate`) — não substitui: os servidores com
  certificado comum (MG, Ambiente Nacional) continuam validando pela lista do sistema;
- no Windows nada muda.

A validação do servidor **nunca é desligada** (`danger_accept_invalid_certs`): o XML fiscal e a
resposta da SEFAZ trafegam por esse canal.

Para atualizar a cadeia (o ITI publica ACs novas e as antigas vencem):

```bash
bash src/data/icp_brasil_gerar.sh   # baixa do ITI, descarta vencidos, ordena; só troca o arquivo se der certo
cargo test                          # reprova se faltar AC usada pela SEFAZ ou houver certificado vencido
```

---

## Mesmo resultado nos dois sistemas

O Linux não só funciona: ele devolve **exatamente** o que o Windows devolve.

- **`CertInfo`:** `subject` e `issuer` são **só o valor do CN**, não a string X.500 completa.
  Para um e-CNPJ, isso dá `RAZAO SOCIAL:CNPJ` (é daí que se costuma tirar o CNPJ do titular).
  Datas em `AAAA/MM/DD`, UTC.
- **Assinatura:** RSA-SHA1 com PKCS#1 v1.5 é **determinística** — mesmo PFX e mesmo texto dão a
  mesma assinatura. Os testes comparam o Linux, byte a byte, com o valor medido no CAPI.
- **Erros:** senha errada → `DfeError::Certificado("Senha inválida ou PFX corrompido")` nos
  dois; arquivo inexistente → `DfeError::Io` (nunca "senha inválida").

---

## Compilando no Linux

Dependências do sistema:

```bash
# Debian / Ubuntu
sudo apt install build-essential pkg-config libssl-dev libxml2-dev

# Fedora / RHEL
sudo dnf install gcc pkgconf-pkg-config openssl-devel libxml2-devel
```

Em produção, o executável precisa das bibliotecas correspondentes (`libssl3`/`openssl-libs` —
que trazem o módulo `legacy` — e `libxml2`). Numa imagem Docker baseada em Debian/Ubuntu:

```dockerfile
FROM rust:1 AS build
RUN apt-get update && apt-get install -y pkg-config libssl-dev libxml2-dev
# ... cargo build --release ...

FROM debian:stable-slim
RUN apt-get update && apt-get install -y libssl3 libxml2 ca-certificates && rm -rf /var/lib/apt/lists/*
```

Conferir se o módulo `legacy` existe na máquina:

```bash
ls "$(openssl version -m | sed 's/.*"\(.*\)"/\1/')"   # deve listar legacy.so
```

Nenhuma configuração do `openssl.cnf` é necessária: a crate carrega o provider sozinha.

---

## macOS

**Não testado.** O código compila para macOS pelo mesmo caminho do Linux
(`cfg(not(target_os = "windows"))`), mas há diferenças que não foram verificadas:

- assinatura e leitura do certificado usam a crate `openssl`, que no macOS exige o OpenSSL 3
  instalado (ex.: Homebrew `openssl@3`, que inclui o provider `legacy`);
- o mTLS **não** passa pelo OpenSSL: no macOS o `native-tls` usa o **Security.framework**, que
  tem a própria leitura de PKCS#12 e o próprio repositório de ACs. O suporte dele a PFX legado e
  a soma da cadeia ICP-Brasil não foram conferidos.

Relatos e correções são bem-vindos.

---

## Como isso é testado

- **PFX sintéticos** em `tests/fixtures/cert/` (gerados por `gerar.sh`): um moderno (AES-256) e
  um legado (RC2-40 + 3DES), os dois com cadeia de AC e acento no nome. São autoassinados, com
  CNPJ fictício e senha pública — a SEFAZ recusa qualquer coisa assinada com eles, e por isso
  podem ficar no repositório.
- **Comparação entre sistemas** (`src/interno/cert.rs`): `CertInfo`, certificado extraído (SHA-1
  do DER inteiro, calculado à parte pela linha de comando) e assinatura têm de bater com os
  valores medidos no Windows; senha errada, arquivo inexistente e PFX corrompido têm de dar o erro
  certo. A identidade do mTLS tem de abrir os dois PFX.
- **Cadeia ICP-Brasil** (`src/interno/connection.rs`, só Linux): carrega inteira, contém as ACs
  da SEFAZ e não tem certificado vencido (aviso para os que vencem em menos de 90 dias).
- **Ao vivo**, na feature `sefaz-live` e **sem certificado nem documento**:
  `cargo test --features sefaz-live --lib live_tls` confere que o cliente confia em cada servidor
  da tabela de webservices — e que, no Linux, um cliente **sem** a cadeia recusa SP (prova de que o
  teste detecta o problema).
- **Com o A1 real**, também na `sefaz-live` (`.env` com caminho, senha e dados do emitente):
  `live_certificado_real` compara dados, certificado e assinatura entre os sistemas sem enviar
  nada; `live_ciclo_nfe_55_homologacao` emite, corrige (CC-e) e cancela uma NF-e, exigindo 100 e
  135; `live_ciclo_nfce_65_homologacao` emite e cancela uma NFC-e; `live_inutilizacao_homologacao` exige 102, ou 998 nas UFs que dispensaram a inutilização.

Todo teste que envia algo à SEFAZ roda **sempre em homologação**: o ambiente é uma constante no
código (`tests/common::HOMOLOGACAO`), não configuração, e `tests/homologacao_travada.rs` reprova
qualquer teste que escolha outro. Detalhes em [Testes](testes.md).

---

## Limitações conhecidas

- **Inutilização** não foi provada contra a SEFAZ a partir do Linux: em SP o serviço responde
  `998 Serviço Desativado` (a inutilização foi dispensada no estado — ver
  [Inutilização](../API.md#inutilização-de-numeração--inutilizacaobuilder)). A assinatura usada
  é a mesma da emissão e dos eventos, que foram aceitos.
- PFX com a AC gravada **antes** do certificado da empresa não está nos testes (o `openssl pkcs12
  -export` sempre põe o da empresa primeiro). O Linux pega o certificado pela chave, então a ordem
  não deveria importar.
- `EscPosNFCeBuilder::print()` (job RAW na impressora) existe só no Windows. Nos outros sistemas,
  gere os bytes com `build()` e envie à impressora (ex.: `/dev/usb/lp0`, CUPS em modo raw).
