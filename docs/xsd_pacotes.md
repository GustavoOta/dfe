# Pacotes de schemas XSD (NF-e/NFC-e)

A `dfe` valida toda NF-e/NFC-e assinada contra os XSD oficiais **antes** de enviar à SEFAZ
(`interno::validation::is_xml_valid`, chamada em `emissao/xml.rs`). Os arquivos são embutidos no
binário com `include_bytes!` e extraídos para o temp do sistema na primeira validação.

| Pasta | Pacote | Situação |
|-------|--------|----------|
| `src/schemas_pl010h/` | **PL_010h v1.03** (13/11/2026) | **em uso** desde 18-11-2026 |
| `src/schemas_nt2026_004/` | PL_010d v1.03 (CNPJ alfanumérico, NT 2026.004) | rollback |
| `src/schemas/` | leiaute anterior (CNPJ numérico) | rollback |

O pacote em uso é a constante `PACOTE_XSD` em `src/interno/validation.rs`. A pasta temporária
leva o nome dele (`<temp>/dfe_schemas_nfe_PL_010h`). O schema é relido do disco a cada validação,
e com uma pasta só, um processo com a `dfe` de outro pacote (o gravisServer em dev, um teste em
paralelo) sobrescreveria os arquivos do outro.

Escopo: só a **NF-e/NFC-e** (`nfe_v4.00.xsd` e o que ele inclui). Os eventos (cancelamento, CC-e,
substituição, manifestação) não passam por esse validador.

---

## Troca PL_010d → PL_010h (18-11-2026)

### Origem dos arquivos

Portal Nacional da NF-e → Documentos → Esquemas XML → *"Schemas XML NF-e - Pacote de Liberação nº
010h v.1.03 - NT 2026.006 v.1.00, NT 2026.008 v.1.01 e NT 2026.005 v.1.00 (Publicado em
13/11/2026)"*. Copiados de `PL_010h_v1.03/NFe/` **sem nenhuma edição**. A pasta `Evento/` do zip
(eventos 110001, 110300 e 412140) não entra, porque a crate não valida evento contra XSD.

SHA-256 dos arquivos embutidos:

```
22cc19e1bb09808ec475d0ad830303699cbf167d630e57023f7d7f1e7c2cb1cc  DFeTiposBasicos_v1.00.xsd
f1bedcdf38e8f6536be2f1a3fb8fb5c27a23364f62a8f7dab0ff884635a000e6  leiauteNFe_v4.00.xsd
adce3646c13ceb54922ec3142fc1dc45bd4fb839ac35ad583e86c733c07d27df  nfe_v4.00.xsd
772619c85723e598840667ca66e7298a250442df47eeb94b397d2a333ce62047  tiposBasico_v4.00.xsd
f56744a5f51c03f027de13f39f869307091781a9ef1d91b1ebe14719ce28e1ac  xmldsig-core-schema_v1.01.xsd
```

O PL_010h acumula os pacotes que saíram depois do PL_010d:

| Pacote | Data | NTs |
|--------|------|-----|
| 010e v1.02 | 10/07/2026 | 2025.002 v1.40, 2026.002, 2026.003 (`tpImp=6`, NF-e Simplificada Tipo 2) |
| 010f v1.04 | 31/08/2026 | 2025.002 v1.50 (monofásico), 2026.007 (IE do emitente opcional) |
| 010g v1.03 | 26/10/2026 | 2026.006 (split payment), 2026.008 (valor líquido) |
| **010h v1.03** | **13/11/2026** | + 2026.005 (IBS/CBS do Simples Nacional) |

### O que mudou, arquivo por arquivo

**`nfe_v4.00.xsd` e `xmldsig-core-schema_v1.01.xsd`:** iguais.

**`tiposBasico_v4.00.xsd`:** só um texto de documentação (`TCnpj`).

**`leiauteNFe_v4.00.xsd`:** só **acrescenta campos opcionais** e **afrouxa** restrições. Nenhum
campo que já existia ficou obrigatório.

| Onde | Mudança | NT |
|------|---------|----|
| `ide/tpImp` | aceita `6` (DANFE Simplificado Tipo 2) | 2026.002/003 |
| `ide/cIndOp` | novo, opcional (`[0-9]{6}`, local da operação de fornecimento), depois de `indIntermed` | — |
| `emit/IE` | passa a ser **opcional** | 2026.007 |
| `emit/tpRegimeIBSCBS` | novo, opcional (1–4), depois de `CRT` | 2026.005 |
| `emit/ISUFEmit` | novo, opcional (`[0-9]{8,9}`) | 2026.005 |
| `prod/vUnComLiq` + `prod/vProdLiq` | novos, opcionais (sequência), depois de `vProd` | 2026.008 |
| `prod/cAtivSN` | novo, opcional (01, 02, 03, 04, 90), depois de `indTot`/`indBemMovelUsado` | 2026.005 |
| `det/imposto/ICMS` | nova alternativa na escolha do grupo, depois do `ICMSSN900`: `ICMSPrevistoPagtoAntecip/vICMSPrevisto` (os grupos atuais continuam valendo) | — |
| `total/vProdLiqTot` | novo, opcional | 2026.008 |
| `total/.../vNFTot` | `TDec_1302Opc` → `TDec_1302` (passa a aceitar `0`) | — |
| `pag/gPgtoVinc` | novo, opcional (split payment), com unicidade de `nPag` e `idTransacao` | 2026.006 |
| `transp/reboque` | até 5 ocorrências | — |
| `xMsg` | `maxLength` 200 → 255 | — |
| `infNFeSupl/qrCode` | o pattern do QR Code v3 mudou de posição na lista (mesmo conteúdo) | — |
| `TTpNFCredito` (`tpNFCredito`) | aceita `06` = retorno por recusa parcial na entrega | — |

**`DFeTiposBasicos_v1.00.xsd` (tipos da Reforma Tributária):** aqui a mudança é estrutural.

| Tipo | Mudança |
|------|---------|
| `TTribNFe` (o `<IBSCBS>` do item) | `gIBSCBS` passa a ser `TCIBS_NFe`; entra `gTribSN` opcional (`vIBSSN`, `vCBSSN`) depois da escolha `gIBSCBS`/`gIBSCBSMono`/`gTransfCred`/`gAjusteCompet` |
| `TCIBS_NFe` (novo) | igual ao `TCIBS`, mais o `gALCZFMCBS` opcional (`tpALCZFMCBS`, `nProcSuframa`) |
| `TMonofasia` | reorganizado em `gIBSMonoAdRem` etc. (escolha por tributo) |
| `TIS` (Imposto Seletivo) | `pISEspec` → `adRemIS` |
| `TTribNFGas` | `gIBSCBS` sem a alternativa `gIBSCBSMono` |
| `TCompraGov` / `TPagRef` | `tpOperGov`, `refDFeAnt`; tipos `TChDFeRTC`, `TCnpjRTC`, `TCnpjBaseRTC` |

### Por que a troca não quebra o que a crate emite

- A `dfe` **não emite** nenhum dos campos novos. Nota sem eles valida igual nos dois pacotes.
- No IBS/CBS, a crate monta só `CST`, `cClassTrib` e `gIBSCBS` (`gIBSUF`, `gIBSMun`, `gCBS`,
  com `gRed` quando o CST exige), ou o grupo sem `gIBSCBS`. Isso cabe no `TCIBS_NFe` sem
  mudança. Ela não emite `IS`, monofasia, `gTribSN` nem compra governamental, que são os pontos
  que mudaram de forma.

### Como foi validado

1. **Suíte padrão** (`cargo test`): toda verde. Os testes de fragmento do `IBSCBS`
   (`emissao/det.rs`, com e sem `gRed`, sem grupo, fora de ordem reprovando) rodam contra o
   pacote novo.
2. **Testes do pacote** (`interno/validation.rs`):
   - `pacote_embutido_e_o_pl010h`: os campos novos existem no leiaute embutido e a pasta temp
     tem o nome do pacote;
   - `nota_sem_campos_novos_continua_valida_no_pacote_novo`: as fixtures NF-e 55 (CRT 3, com
     `IBSCBS`) e NFC-e 65 (Simples) passam sem mudança;
   - `tp_imp_6_da_nfe_simplificada_tipo_2_passa_no_xsd`: `tpImp=6` passa e `7` reprova;
   - `c_ativ_sn_vale_no_lugar_certo_e_reprova_fora_dele`: `cAtivSN` depois de `indTot` passa;
     fora de ordem ou com código fora do domínio reprova.
3. **Homologação na SEFAZ-SP** (`cargo test --features sefaz-live`, ambiente travado em 2), com a
   nota montada pela crate e validada localmente no PL_010h antes do envio:

   | Teste | Resultado |
   |-------|-----------|
   | `live_status_servico` | 107, serviço em operação |
   | `live_ciclo_nfe_55_homologacao` | emissão **100**; a CC-e voltou 999 (ver abaixo) |
   | `live_ciclo_nfce_65_homologacao` | emissão **100**, cancelamento 135 |
   | `live_nfce_65_destinatario_cnpj_homologacao` | emissão **100**, cancelamento 135 |

   A SEFAZ aceitou o XML de hoje e a validação local nova não recusou nada. Por isso a troca foi
   **aplicada já**, sem esperar a data de vigência dos campos novos.

**CC-e com 999 em homologação.** No ciclo da NF-e 55, a carta de correção voltou `999 — Erro não
catalogado`. O mesmo ciclo rodado com o PL_010d deu o mesmo 999, e o evento nem passa pelo
validador de NF-e. Então a causa não é a troca de pacote. O caso fica para investigar à parte.

### Rollback

Em `src/interno/validation.rs`: trocar `schemas_pl010h` por `schemas_nt2026_004` nos cinco
`include_bytes!` e `PACOTE_XSD` para `"PL_010d"`, depois ajustar o teste `pacote_embutido_e_o_pl010h`.
O teste `tp_imp_6_…` reprova no PL_010d, como esperado.

---

## Próximo pacote — roteiro

1. Baixar o zip no portal (lista de esquemas: `listaConteudo.aspx?tipoConteudo=BMPFMBoln3w=`;
   o portal pede cookie, `curl -c/-b` com redirect) para uma pasta fora do repositório.
2. Criar `src/schemas_<pacote>/` com os cinco arquivos de `NFe/`, **sem editar**, e manter a pasta
   anterior.
3. Comparar com o pacote em uso (`diff -w --strip-trailing-cr`): campo obrigatório novo, pattern
   mais restrito, enumeração removida ou tipo trocado em grupo que a crate emite são os pontos de
   risco.
4. Trocar os `include_bytes!` e o `PACOTE_XSD`, depois atualizar `pacote_embutido_…`.
5. `cargo test` e os ciclos `sefaz-live` (55, 65, 65 com CNPJ) em homologação.
6. Se passar, aplicar. Se a SEFAZ recusar o XML de hoje, guardar a pasta e aplicar na data de
   vigência do pacote.
7. Atualizar esta página e o `CHANGELOG.md`.
