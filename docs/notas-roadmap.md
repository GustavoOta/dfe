# Notas importantes e Roadmap

## Notas importantes

- Sempre teste em **homologação** (`tp_amb: 2`) antes de produção.
- O certificado `.pfx` é lido do disco a cada operação — nunca cacheado em memória.
- Em `tp_amb = 2`, o campo `x_prod` do **primeiro item** é substituído automaticamente por `"NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL"` (exigência SEFAZ).
- Os webservices cobrem as **27 UFs** (autorizadores próprios e compartilhados) e o **Ambiente
  Nacional**, em produção e homologação. As URLs ficam em `src/data/webservices.json`; correções
  são bem-vindas por PR.
- A URL do QR Code da NFC-e ainda é a de SP.

---

## Roadmap

Situação em **19-11-2026** (versão 0.8.x). Legenda: ✅ pronto · 🟡 parcial · 🔜 planejado · ⛔ fora do escopo

### Documentos fiscais eletrônicos

| Documento | Modelo | Situação |
|---|:---:|:---:|
| **NF-e** — emissão, cancelamento, CC-e, inutilização, consulta, status, distribuição, manifestação, DANFE A4/80 mm | 55 | ✅ |
| **NFC-e** — emissão (inclusive contingência off-line), cancelamento, cancelamento por substituição, DANFE 80 mm, ESC/POS | 65 | ✅ |
| **NFS-e Nacional** — DPS v1.01 montada, assinada e enviada à SEFIN Nacional, com consulta pela DPS e pela chave (feature `nfse`); faltam IBS/CBS, eventos e DANFSe | — | 🟡 |
| **NF-e Simplificada** (NT 2026.002/003) — leiaute PL_010h pronto; faltam `tpImp=6`, QR Code na NF-e e URL por UF | 55 | 🟡 |
| **MDF-e** | 58 | 🔜 |
| **CT-e** | 57 | 🔜 |
| **DC-e** — Declaração de Conteúdo eletrônica (obrigatória desde 06/04/2026) | 99 | 🔜 |
| **CT-e OS**, **BP-e**, **GTV-e** | 67 · 63 · 64 | 🔜 depois do CT-e |
| **NFCom**, **NF3e**, **NFAg**, **NFGas** (documentos de concessionária) | 62 · 66 · 75 · 76 | 🔜 sob demanda |
| CF-e SAT / MF-e (substituídos pela NFC-e; SAT encerrado em SP em 01/01/2026) | 59 | ⛔ |

### NF-e / NFC-e — tributos e leiaute

| Item | Situação |
|---|:---:|
| `ICMS00` a `ICMS90`, Simples Nacional (`ICMSSN`), ST, partilha, repasse, monofásico | ✅ |
| **DIFAL** (`v_icms_uf_dest`, `v_icms_uf_remet`) | ✅ |
| **IPI** por item, **PIS/COFINS** de qualquer CST (inclusive ST) | ✅ |
| **IBS / CBS** (reforma tributária) — grupo por item e nos totais | ✅ |
| **CNPJ alfanumérico** e validação de CNPJ/CPF | ✅ |
| XSD **PL_010h** embutido | ✅ |
| DANFE A4 no modelo da reforma (NT 2026.010) | ✅ |
| **IBS/CBS do Simples Nacional** no formato da NT 2026.005 (`cAtivSN` por item, sem `gIBSCBS`) — leiaute em produção em 14/12/2026; obrigatório na NF-e/NFC-e do Simples a partir de 01/03/2027 | 🔜 |
| **Nota de crédito e nota de débito** (`finNFe` 5/6, `tpNFCredito`/`tpNFDebito`) | 🔜 |
| Compras governamentais (`gCompraGov`) | 🔜 |
| Valor líquido do item (`vProdLiq`, `vItem`, `vProdLiqTot` — NT 2026.008) | 🔜 |
| Split payment (`gPgtoVinc` — NT 2026.006) | 🔜 |
| QR Code v3 da NFC-e (sem CSC) | 🔜 |
| Grupos do leiaute ainda não montados: importação (II no item, DI), exportação, veículos, medicamentos, armas, rastreabilidade, RECOPI, ISSQN no item, local de retirada, veículo/reboque/volumes no transporte, cobrança (fatura/duplicatas), `autXML`, responsável técnico, intermediador, compra, cana — ver [Cobertura](cobertura.md) | 🔜 |
| Mais de uma forma de pagamento na mesma nota (vários `detPag`) e outros documentos referenciados (modelo 1/1A, produtor rural, ECF, CT-e) | 🟡 |

### Eventos

| Evento | Código | Situação |
|---|:---:|:---:|
| Carta de Correção | 110110 | ✅ |
| Cancelamento | 110111 | ✅ |
| Cancelamento por substituição (NFC-e) | 110112 | ✅ |
| Manifestação do destinatário | 210200 · 210210 · 210220 · 210240 | ✅ |
| Eventos da reforma tributária (NT 2025.002 — crédito presumido, nota de crédito, imobilização…) | 112xxx · 211xxx · 212xxx | 🔜 |
| EPEC | 110140 | 🔜 |
| Comprovante de entrega / insucesso de entrega (e cancelamentos) | 110130/110131 · 110192/110193 | 🔜 |
| Conciliação financeira (e cancelamento) | 110750 · 110751 | 🔜 |
| Ator interessado | 110150 | 🔜 |
| Split payment | 110300 | 🔜 |
| Prorrogação de ICMS (industrialização) | 111500–111503 | 🔜 sob demanda |

### Serviços e contingência

| Item | Situação |
|---|:---:|
| Status do serviço, consulta de situação, inutilização, distribuição de DF-e por NSU/chave | ✅ |
| Contingência off-line da NFC-e (`tpEmis 9`) | ✅ |
| Contingência **SVC-AN / SVC-RS** da NF-e (`tpEmis 6/7`) — endpoints já cadastrados, falta o roteamento | 🟡 |
| Contingência **EPEC** (`tpEmis 4`) | 🔜 |
| **Consulta Cadastro** (`CadConsultaCadastro4`) | 🔜 |
| Lote de manifestações/eventos (até 20 por envio), conferência de NF-e recebida (assinatura + protocolo), manutenção do CSC | 🔜 |
| Consulta GTIN (Cadastro Centralizado de GTIN) | 🔜 |

### Impressão e plataformas

| Item | Situação |
|---|:---:|
| ESC/POS `EscPosBuilder`, `EscPosNFCeBuilder`, `EscPosDanfeNFeBuilder` | ✅ |
| DANFE A4 com [paginação automática](danfe.md#paginação-da-nf-e-em-a4) | ✅ |
| **Windows** — CryptoAPI + SChannel | ✅ |
| **Linux** — OpenSSL do sistema com provider `legacy` + cadeia ICP-Brasil embutida ([detalhes](certificado-e-plataformas.md)); NF-e e NFC-e autorizadas, corrigidas e canceladas em homologação com A1 real | ✅ |
| Impressão ESC/POS direta (`print()`) fora do Windows | 🔜 |
| **macOS** — testar (TLS pelo Security.framework) | 🔜 |

### Problema conhecido

- **CC-e com cStat 999 em homologação**: em investigação; não está ligado ao pacote de XSD.

---

Sugestões e prioridades são discutidas na
[comunidade do dfe no WhatsApp](https://chat.whatsapp.com/GYV2d62n46k7vYaZg5zRBb) e nas
[issues do GitHub](https://github.com/GustavoOta/dfe/issues).
