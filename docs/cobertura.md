# Cobertura da NF-e e da NFC-e

O que a `dfe` faz hoje, item por item, nos serviços da SEFAZ, na montagem do XML e na contingência
— inclusive o que ainda falta.

> **Conferido em 23-11-2026** no código da `dfe` 0.8.2. Se notar algo desatualizado, abra uma
> [issue](https://github.com/GustavoOta/dfe/issues).

Legenda: ✅ tem · 🟡 parcial · 🔜 ainda não (no [roadmap](notas-roadmap.md)) · — não se aplica

---

## Serviços da SEFAZ

| Serviço | O que faz | dfe |
|---|---|:---:|
| Autorização | Envio da NF-e/NFC-e para autorização | ✅ `NFeBuilder::emitir()`, ou `assinar()` + `transmitir()` para ter a chave antes do envio. Envio síncrono, uma nota por vez |
| Consulta de lote assíncrono | Resultado de lote enviado em modo assíncrono | — a `dfe` envia em modo síncrono |
| Consulta pela chave | Situação e protocolo da nota | ✅ `ConsultaSituacaoBuilder` |
| Inutilização | Faixa de numeração não usada | ✅ `InutilizacaoBuilder` |
| Status do serviço | Disponibilidade por UF e modelo | ✅ `NFeService` |
| Consulta cadastro | Situação cadastral do contribuinte na UF | 🔜 |
| Distribuição de DF-e | Documentos de interesse por NSU ou pela chave (inclusive download da NF-e) | ✅ `Distribuicao`, `DistribuicaoNSU`, `DistribuicaoChaveAcesso` |
| Cancelamento | Evento 110111 | ✅ `CancelarBuilder` |
| Cancelamento por substituição | Evento 110112 (NFC-e) | ✅ `SubstituicaoBuilder` |
| Carta de correção | Evento 110110, com `nSeqEvento` acumulativo | ✅ `CartaCorrecaoBuilder` |
| Manifestação do destinatário | 210200, 210210, 210220, 210240 | ✅ `ManifestacaoBuilder` |
| Manifestação ou eventos em lote | Até 20 eventos num envio | 🔜 (um por envio) |
| EPEC | Contingência por evento (110140), inclusive NFC-e | 🔜 |
| Ator interessado | Evento 110150 | 🔜 |
| Comprovante e insucesso de entrega | Eventos 110130/110131 e 110192/110193 | 🔜 |
| Conciliação financeira | Eventos 110750/110751 | 🔜 |
| Prorrogação do ICMS | Eventos 111500–111503 | 🔜 sob demanda |
| Eventos da Reforma Tributária | IBS/CBS (112xxx, 211xxx, 212xxx) e vinculação de pagamento (110300) | 🔜 |
| Manutenção do CSC | Consulta, novo e revogação do CSC da NFC-e | 🔜 sob demanda |
| Conferência de NF-e recebida | Assinatura e protocolo conferidos na SEFAZ | 🔜 |

## Montagem do XML

| Grupo do leiaute | dfe |
|---|:---:|
| Identificação, emitente (CNPJ ou CPF), destinatário, local de entrega | ✅ `Ide`, `Emit`, `Dest`, `Entrega` |
| Local de retirada | 🔜 |
| Documento referenciado | 🟡 chave de NF-e/NFC-e (`refNFe`); faltam nota modelo 1/1A, produtor rural, cupom ECF e CT-e |
| Produto: código, GTIN, NCM, NVE, CEST, EXTIPI, cBenef, pedido, informação adicional | ✅ `Det` |
| ICMS — todos os grupos, inclusive ST, partilha, repasse, desoneração, monofásico e DIFAL | ✅ `Icms` (valores informados ou calculados a partir do cadastro) |
| IPI, PIS e COFINS de qualquer CST | ✅ |
| IBS/CBS (Reforma Tributária) e crédito presumido | ✅ |
| Combustíveis (ANP, encerrante, origem) | ✅ `Comb` |
| Importação (II no item, DI e adições) | 🔜 |
| Exportação (`detExport`, grupo `exporta`) | 🔜 |
| Veículos novos, medicamentos, armas, rastreabilidade (lote), RECOPI | 🔜 |
| ISSQN no item (NF-e conjugada) | 🔜 |
| Totais | ✅ calculados a partir dos itens |
| Transporte: modalidade e transportadora | ✅ `Transp` |
| Transporte: veículo, reboque, volumes, retenção do ICMS | 🔜 |
| Cobrança (fatura e duplicatas) | 🔜 |
| Pagamento: forma, valor, troco, cartão | 🟡 `Pag` — uma forma de pagamento por nota |
| Informações adicionais (fisco e contribuinte) | ✅ `InfAdic` |
| Observações do contribuinte/fisco por campo, processo referenciado | 🔜 |
| Autorizados a baixar o XML (`autXML`), responsável técnico (`infRespTec`), intermediador | 🔜 |
| Compra, cana-de-açúcar, compras governamentais, agropecuário | 🔜 |
| QR Code da NFC-e | ✅ versão 2 (versão 3 🔜) |
| IBS/CBS do Simples no formato da NT 2026.005, split payment | 🔜 |

## Protocolo, contingência e validação

| Item | dfe |
|---|:---:|
| NF-e autorizada com o protocolo (`nfeProc`) | ✅ devolvida pela emissão |
| Evento e inutilização com o protocolo (`procEventoNFe`, `procInutNFe`) | 🟡 a resposta traz o XML enviado e o recebido, sem montar o `proc` |
| Contingência off-line da NFC-e (`tpEmis` 9) | ✅ |
| Contingência SVC-AN / SVC-RS | 🟡 endereços cadastrados, falta o roteamento |
| Validação no XSD oficial | ✅ pacote PL_010h embutido |
| Download de NFC-e pelo SAE da SEFAZ-SP | 🔜 |

## Na mesma crate

| Recurso | dfe |
|---|:---:|
| DANFE em PDF (A4 e 80 mm), DANFE da NFC-e, modelo da Reforma (NT 2026.010) | ✅ `DanfeBuilder` |
| Cupom da NFC-e e DANFE simplificado em impressora térmica (ESC/POS) | ✅ `EscPosNFCeBuilder`, `EscPosDanfeNFeBuilder` |
| NFS-e Nacional (DPS, assinatura, envio e consulta na SEFIN) | ✅ feature `nfse` |
| Windows (CryptoAPI) e Linux (OpenSSL) com a mesma assinatura byte a byte | ✅ |
