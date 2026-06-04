# Notas importantes e Roadmap

## Notas importantes

- Sempre teste em **homologação** (`tp_amb: 2`) antes de produção.
- O certificado `.pfx` é lido do disco a cada operação — nunca cacheado em memória.
- Em `tp_amb = 2`, o campo `x_prod` do **primeiro item** é substituído automaticamente por `"NOTA FISCAL EMITIDA EM AMBIENTE DE HOMOLOGACAO - SEM VALOR FISCAL"` (exigência SEFAZ).
- Os webservices cobertos são da **SEFAZ/SP** e do **Ambiente Nacional**. Para outras UFs, contribua adicionando URLs em `interno/ws.rs`.

## Roadmap

| Item | Status |
|---|:---:|
| `ICMS10/20/30/51/70` | ✅ |
| `ICMS90` / `Sn900` completos | ✅ |
| **DIFAL** (`v_icms_uf_dest`, `v_icms_uf_remet`) | ✅ |
| **IPI** por item (`Det.ipi`) | ✅ |
| **PIS/COFINS ST** (CST 05) | ✅ |
| **IBS / CBS** (reforma tributária) | ✅ |
| Validação de CNPJ/CPF | ✅ |
| ESC/POS `EscPosBuilder` | ✅ |
| ESC/POS `EscPosNFCeBuilder` | ✅ |
| Contingência (EPEC / FS-DA) | 🔜 |
| **Suporte Linux / macOS** — reimplementar assinatura e extração de certificado sem CAPI (OpenSSL com provider legacy do sistema ou biblioteca puro-Rust com suporte a 3DES) | 🔜 |
