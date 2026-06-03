use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Ide ──────────────────────────────────────────────────────────────────────

/// Identificação do documento fiscal (`<ide>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ide {
    /// Código da UF emitente (IBGE, ex.: `35` = SP).
    pub c_uf: u16,
    /// Código numérico da NF-e. Se `None`, um código aleatório é gerado automaticamente.
    pub c_nf: Option<String>,
    /// Natureza da operação (ex.: `"VENDA DE MERCADORIA"`).
    pub nat_op: String,
    /// Indicador de pagamento (legado, opcional).
    pub ind_pag: Option<u8>,
    /// Modelo do documento: `55` = NF-e · `65` = NFC-e.
    pub mod_: u32,
    /// Série da NF-e (0–999).
    pub serie: u32,
    /// Número da NF-e.
    pub n_nf: u64,
    /// Data e hora de emissão (ISO 8601). Gerado automaticamente se `None`.
    pub dh_emi: Option<String>,
    /// Data e hora de saída/entrada. Opcional.
    pub dh_sai_ent: Option<String>,
    /// Tipo da operação: `0` = Entrada · `1` = Saída.
    pub tp_nf: u8,
    /// Identificador de local de destino: `1` = Interna · `2` = Interestadual · `3` = Exterior.
    pub id_dest: u8,
    /// Código do município de ocorrência do fato gerador (IBGE).
    pub c_mun_fg: String,
    /// Tipo de impressão: `1` = DANFE NF-e normal · `4` = DANFE NFC-e.
    pub tp_imp: u8,
    /// Forma de emissão: `1` = Normal · `5` = Contingência EPEC.
    pub tp_emis: u8,
    /// Ambiente: `1` = Produção · `2` = Homologação.
    pub tp_amb: u8,
    /// Finalidade: `1` = Normal · `2` = Complementar · `3` = Ajuste · `4` = Devolução.
    pub fin_nfe: u8,
    /// Consumidor final: `0` = Não · `1` = Sim.
    pub ind_final: u8,
    /// Presença do comprador: `1` = Presencial · `2` = Não-presencial (internet) · `9` = Outros.
    pub ind_pres: u8,
    /// Processo de emissão: `0` = Emissão com aplicativo do contribuinte.
    pub proc_emi: u8,
    /// Versão do processo de emissão (ex.: `"1.0.0"`).
    pub ver_proc: String,
}

impl Default for Ide {
    fn default() -> Self {
        Ide {
            c_uf: 35,
            c_nf: None,
            nat_op: "VENDA".to_string(),
            ind_pag: None,
            mod_: 55,
            serie: 1,
            n_nf: 1,
            dh_emi: None,
            dh_sai_ent: None,
            tp_nf: 1,
            id_dest: 1,
            c_mun_fg: "3550308".to_string(),
            tp_imp: 1,
            tp_emis: 1,
            tp_amb: 2,
            fin_nfe: 1,
            ind_final: 1,
            ind_pres: 1,
            proc_emi: 0,
            ver_proc: "1.0.0".to_string(),
        }
    }
}

// ─── Dest ─────────────────────────────────────────────────────────────────────

/// Dados do destinatário da NF-e (`<dest>`).
/// Opcional para NFC-e; obrigatório para NF-e modelo 55.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Dest {
    pub cnpj: Option<String>,
    pub cpf: Option<String>,
    pub id_estrangeiro: Option<String>,
    pub x_nome: Option<String>,
    pub x_lgr: Option<String>,
    pub nro: Option<String>,
    pub x_bairro: Option<String>,
    pub c_mun: Option<String>,
    pub x_mun: Option<String>,
    pub uf: Option<String>,
    pub cep: Option<String>,
    pub c_pais: Option<String>,
    pub x_pais: Option<String>,
    pub fone: Option<String>,
    pub ind_ie_dest: Option<u8>,
    pub ie: Option<String>,
    pub isuf: Option<String>,
    pub im: Option<String>,
    pub email: Option<String>,
}

impl Default for Dest {
    fn default() -> Self {
        Dest {
            cnpj: None,
            cpf: None,
            id_estrangeiro: None,
            x_nome: None,
            x_lgr: None,
            nro: None,
            x_bairro: None,
            c_mun: None,
            x_mun: None,
            uf: None,
            cep: None,
            c_pais: Some("1058".to_string()),
            x_pais: Some("Brasil".to_string()),
            fone: None,
            ind_ie_dest: Some(9),
            ie: None,
            isuf: None,
            im: None,
            email: None,
        }
    }
}

// ─── Emit ─────────────────────────────────────────────────────────────────────

/// Dados do emitente da NF-e (`<emit>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Emit {
    pub cnpj: Option<String>,
    pub cpf: Option<String>,
    pub x_nome: String,
    pub x_fant: Option<String>,
    pub x_lgr: String,
    pub nro: String,
    pub x_cpl: Option<String>,
    pub x_bairro: String,
    pub c_mun: String,
    pub x_mun: String,
    pub uf: String,
    pub cep: String,
    pub c_pais: u16,
    pub x_pais: String,
    pub fone: Option<u64>,
    pub ie: Option<String>,
    pub iest: Option<u64>,
    pub im: Option<String>,
    pub cnae: u32,
    pub crt: u8,
}

impl Default for Emit {
    fn default() -> Self {
        Emit {
            cnpj: None,
            cpf: None,
            x_nome: "Empresa Ltda".to_string(),
            x_fant: None,
            x_lgr: "".to_string(),
            nro: "S/N".to_string(),
            x_cpl: None,
            x_bairro: "".to_string(),
            c_mun: "0000000".to_string(),
            x_mun: "".to_string(),
            uf: "".to_string(),
            cep: "00000000".to_string(),
            c_pais: 1058,
            x_pais: "Brasil".to_string(),
            fone: None,
            ie: None,
            iest: None,
            im: None,
            cnae: 0,
            crt: 1,
        }
    }
}

// ─── Icms ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Icms {
    // ── Regime Normal (CRT=3) ─────────────────────────────────────────────────
    /// CST 00 — Tributada integralmente
    Icms00 { orig: u8, mod_bc: u8, v_bc: f64, p_icms: f64, v_icms: f64 },

    /// CST 10 — Tributada e com cobrança do ICMS por substituição tributária
    Icms10 {
        orig: u8,
        mod_bc: u8,
        v_bc: f64,
        p_icms: f64,
        v_icms: f64,
        mod_bcst: u8,
        p_mvast: f64,
        p_red_bcst: Option<f64>,
        v_bcst: f64,
        p_icmsst: f64,
        v_icmsst: f64,
    },

    /// CST 20 — Com redução de base de cálculo
    Icms20 {
        orig: u8,
        mod_bc: u8,
        p_red_bc: f64,
        v_bc: f64,
        p_icms: f64,
        v_icms: f64,
        v_icms_deson: Option<f64>,
        mot_des_icms: Option<u16>,
    },

    /// CST 30 — Isenta/NT para o emitente e com cobrança do ICMS por ST
    Icms30 {
        orig: u8,
        mod_bcst: u8,
        p_mvast: f64,
        p_red_bcst: Option<f64>,
        v_bcst: f64,
        p_icmsst: f64,
        v_icmsst: f64,
        v_icms_deson: Option<f64>,
        mot_des_icms: Option<u16>,
    },

    /// CST 40=Isenta | 41=Não tributada | 50=Suspensão
    Icms40 { orig: u8, cst: u16, v_icms_deson: Option<f64>, mot_des_icms: Option<u16> },

    /// CST 51 — Diferimento total ou parcial (todos os campos opcionais)
    Icms51 {
        orig: u8,
        mod_bc: Option<u8>,
        p_red_bc: Option<f64>,
        v_bc: Option<f64>,
        p_icms: Option<f64>,
        v_icms_op: Option<f64>,
        p_dif: Option<f64>,
        v_icms_dif: Option<f64>,
        v_icms: Option<f64>,
    },

    /// CST 60 — ICMS-ST retido anteriormente
    Icms60 {
        orig: u8,
        v_bcst_ret: Option<f64>,
        p_st: Option<f64>,
        v_icms_substituto: Option<f64>,
        v_icmsst_ret: Option<f64>,
    },

    /// CST 70 — Com redução de BC e cobrança do ICMS por ST
    Icms70 {
        orig: u8,
        mod_bc: u8,
        p_red_bc: Option<f64>,
        v_bc: f64,
        p_icms: f64,
        v_icms: f64,
        mod_bcst: u8,
        p_mvast: f64,
        p_red_bcst: Option<f64>,
        v_bcst: f64,
        p_icmsst: f64,
        v_icmsst: f64,
        v_icms_deson: Option<f64>,
        mot_des_icms: Option<u16>,
    },

    /// CST 90 — Outros (todos os campos opcionais)
    Icms90 {
        orig: u8,
        mod_bc: Option<u8>,
        p_red_bc: Option<f64>,
        v_bc: Option<f64>,
        p_icms: Option<f64>,
        v_icms: Option<f64>,
        mod_bcst: Option<u8>,
        p_mvast: Option<f64>,
        p_red_bcst: Option<f64>,
        v_bcst: Option<f64>,
        p_icmsst: Option<f64>,
        v_icmsst: Option<f64>,
        v_icms_deson: Option<f64>,
        mot_des_icms: Option<u16>,
    },

    // ── Simples Nacional (CRT=1) ──────────────────────────────────────────────
    /// CSOSN 101 — tributada com crédito
    Sn101 { orig: u8, p_cred_sn: f64, v_cred_icmssn: f64 },
    /// CSOSN 102/103/300/400
    Sn102 { orig: u8, csosn: String },
    /// CSOSN 500 — ST retido anteriormente
    Sn500 { orig: u8, v_bcst_ret: Option<f64>, v_icmsst_ret: Option<f64> },
    /// CSOSN 900 — outros; inclui campos opcionais de cálculo e ST
    Sn900 {
        orig: u8,
        mod_bc: Option<u8>,
        v_bc: Option<f64>,
        p_red_bc: Option<f64>,
        p_icms: Option<f64>,
        v_icms: Option<f64>,
        p_cred_sn: Option<f64>,
        v_cred_icmssn: Option<f64>,
        // campos ST opcionais
        mod_bcst: Option<u8>,
        p_mvast: Option<f64>,
        p_red_bcst: Option<f64>,
        v_bcst: Option<f64>,
        p_icmsst: Option<f64>,
        v_icmsst: Option<f64>,
    },
}

impl Icms {
    // ── Regime Normal (CRT=3) ────────────────────────────────────────────────

    pub fn icms00(orig: u8, mod_bc: u8, v_bc: f64, p_icms: f64, v_icms: f64) -> Self {
        Icms::Icms00 { orig, mod_bc, v_bc, p_icms, v_icms }
    }

    /// CST 10 — ICMS próprio + ST; `p_red_bcst` opcional
    pub fn icms10(orig: u8, mod_bc: u8, v_bc: f64, p_icms: f64, v_icms: f64,
                  mod_bcst: u8, p_mvast: f64, v_bcst: f64, p_icmsst: f64, v_icmsst: f64) -> Self {
        Icms::Icms10 { orig, mod_bc, v_bc, p_icms, v_icms,
                       mod_bcst, p_mvast, p_red_bcst: None, v_bcst, p_icmsst, v_icmsst }
    }

    /// CST 20 — BC reduzida; campos de desoneração opcionais
    pub fn icms20(orig: u8, mod_bc: u8, p_red_bc: f64, v_bc: f64, p_icms: f64, v_icms: f64) -> Self {
        Icms::Icms20 { orig, mod_bc, p_red_bc, v_bc, p_icms, v_icms,
                       v_icms_deson: None, mot_des_icms: None }
    }

    /// CST 30 — Isenta/NT + ST; campos de desoneração opcionais
    pub fn icms30(orig: u8, mod_bcst: u8, p_mvast: f64, v_bcst: f64,
                  p_icmsst: f64, v_icmsst: f64) -> Self {
        Icms::Icms30 { orig, mod_bcst, p_mvast, p_red_bcst: None,
                       v_bcst, p_icmsst, v_icmsst, v_icms_deson: None, mot_des_icms: None }
    }

    /// CST 40=Isenta | 41=Não tributada | 50=Suspensão
    pub fn icms40(orig: u8, cst: u16) -> Self {
        Icms::Icms40 { orig, cst, v_icms_deson: None, mot_des_icms: None }
    }

    /// CST 51 — Diferimento; todos os campos são opcionais
    pub fn icms51(orig: u8) -> Self {
        Icms::Icms51 { orig, mod_bc: None, p_red_bc: None, v_bc: None,
                       p_icms: None, v_icms_op: None, p_dif: None, v_icms_dif: None, v_icms: None }
    }

    /// ICMS-ST retido anteriormente — campos ST opcionais (NT 2011/004)
    pub fn icms60(orig: u8) -> Self {
        Icms::Icms60 { orig, v_bcst_ret: None, p_st: None, v_icms_substituto: None, v_icmsst_ret: None }
    }

    /// CST 70 — BC reduzida + ST; `p_red_bc` e `p_red_bcst` opcionais
    pub fn icms70(orig: u8, mod_bc: u8, v_bc: f64, p_icms: f64, v_icms: f64,
                  mod_bcst: u8, p_mvast: f64, v_bcst: f64, p_icmsst: f64, v_icmsst: f64) -> Self {
        Icms::Icms70 { orig, mod_bc, p_red_bc: None, v_bc, p_icms, v_icms,
                       mod_bcst, p_mvast, p_red_bcst: None, v_bcst, p_icmsst, v_icmsst,
                       v_icms_deson: None, mot_des_icms: None }
    }

    /// CST 90 — Outros; todos os campos são opcionais
    pub fn icms90(orig: u8) -> Self {
        Icms::Icms90 { orig, mod_bc: None, p_red_bc: None, v_bc: None,
                       p_icms: None, v_icms: None, mod_bcst: None, p_mvast: None,
                       p_red_bcst: None, v_bcst: None, p_icmsst: None, v_icmsst: None,
                       v_icms_deson: None, mot_des_icms: None }
    }

    // ── Simples Nacional (CRT=1) ─────────────────────────────────────────────

    /// CSOSN 101 — tributada com crédito
    pub fn sn101(orig: u8, p_cred_sn: f64, v_cred_icmssn: f64) -> Self {
        Icms::Sn101 { orig, p_cred_sn, v_cred_icmssn }
    }

    /// CSOSN 102/103/300/400
    pub fn sn102(orig: u8, csosn: &str) -> Self {
        Icms::Sn102 { orig, csosn: csosn.to_string() }
    }

    /// CSOSN 500 — ST retido anteriormente; campos ST opcionais
    pub fn sn500(orig: u8) -> Self {
        Icms::Sn500 { orig, v_bcst_ret: None, v_icmsst_ret: None }
    }

    /// CSOSN 900 — outros; todos os campos são opcionais
    pub fn sn900(orig: u8) -> Self {
        Icms::Sn900 {
            orig, mod_bc: None, v_bc: None, p_red_bc: None, p_icms: None,
            v_icms: None, p_cred_sn: None, v_cred_icmssn: None,
            mod_bcst: None, p_mvast: None, p_red_bcst: None,
            v_bcst: None, p_icmsst: None, v_icmsst: None,
        }
    }
}

// ─── Pis ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pis {
    /// CST 01/02 — tributada por alíquota
    Aliq { cst: String, v_bc: f64, p_pis: f64, v_pis: f64 },
    /// CST 99 — outros (zeros automáticos)
    Outr,
    /// CST 04-09 — não tributado/isento/suspenso
    Nt { cst: String },
    /// CST 03 — tributada por quantidade
    Qtde { cst: String, q_bc_prod: f64, v_aliq_prod: f64, v_pis: f64 },
    /// CST 05 — substituição tributária
    /// Use `v_bc + p_pis` OU `q_bc_prod + v_aliq_prod` (os outros ficam `None`)
    St { v_bc: Option<f64>, p_pis: Option<f64>, q_bc_prod: Option<f64>, v_aliq_prod: Option<f64>, v_pis: f64 },
}

// ─── Cofins ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cofins {
    /// CST 01/02 — tributada por alíquota
    Aliq { cst: String, v_bc: f64, p_cofins: f64, v_cofins: f64 },
    /// CST 99 — outros
    Outr { cst: String },
    /// CST 04-09 — não tributado/isento/suspenso
    Nt { cst: String },
    /// CST 03 — tributada por quantidade
    Qtde { cst: String, q_bc_prod: f64, v_aliq_prod: f64, v_cofins: f64 },
    /// CST 05 — substituição tributária
    St { v_bc: Option<f64>, p_cofins: Option<f64>, q_bc_prod: Option<f64>, v_aliq_prod: Option<f64>, v_cofins: f64 },
}

// ─── Ipi ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipi {
    /// Código de Enquadramento Legal do IPI (3 dígitos, "999" = outros)
    pub c_enq: String,
    /// CST IPI: "50"=saída tributada, "53"=saída NT, etc.
    pub cst: String,
    /// Base de cálculo por valor (exclusivo com q_bc_prod)
    pub v_bc: Option<f64>,
    /// Alíquota ad valorem (usado junto com v_bc)
    pub p_ipi: Option<f64>,
    /// Base de cálculo por quantidade (exclusivo com v_bc)
    pub q_bc_prod: Option<f64>,
    /// Alíquota por unidade de medida (usado junto com q_bc_prod)
    pub v_aliq_prod: Option<f64>,
    /// Valor do IPI; obrigatório para CST de saída tributada ("50", "99")
    pub v_ipi: Option<f64>,
    /// Código do selo de controle IPI
    pub c_selo: Option<String>,
    /// Quantidade de selos
    pub q_selo: Option<u32>,
}

impl Ipi {
    /// CST 50 — saída tributada por alíquota ad valorem
    pub fn tributado(c_enq: &str, v_bc: f64, p_ipi: f64, v_ipi: f64) -> Self {
        Ipi { c_enq: c_enq.to_string(), cst: "50".to_string(),
              v_bc: Some(v_bc), p_ipi: Some(p_ipi), v_ipi: Some(v_ipi),
              q_bc_prod: None, v_aliq_prod: None, c_selo: None, q_selo: None }
    }

    /// CST 53 — saída não tributada (isento/imune/etc.)
    pub fn nao_tributado(c_enq: &str, cst: &str) -> Self {
        Ipi { c_enq: c_enq.to_string(), cst: cst.to_string(),
              v_bc: None, p_ipi: None, q_bc_prod: None, v_aliq_prod: None,
              v_ipi: None, c_selo: None, q_selo: None }
    }
}

// ─── IbsCbs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IbsCbs {
    pub cst: String,
    pub class_trib: String,
    pub v_bc: Decimal,
    pub p_ibs_uf: Decimal,
    pub v_ibs_uf: Decimal,
    pub p_ibs_mun: Decimal,
    pub v_ibs_mun: Decimal,
    pub p_cbs: Decimal,
    pub v_cbs: Decimal,
}

// ─── Det ──────────────────────────────────────────────────────────────────────

/// Dados de um item da NF-e (`<det>`).
///
/// Os totais de ICMS, PIS e COFINS são **calculados automaticamente** pelo builder
/// a partir dos valores declarados em `icms`, `pis` e `cofins`.
/// Use `..Default::default()` para preencher os campos opcionais com zeros/`None`.
///
/// # Exemplo
///
/// ```
/// use dfe::tipos::{Det, Icms, Pis, Cofins};
///
/// let item = Det {
///     c_prod: "001".into(),
///     x_prod: "PRODUTO EXEMPLO".into(),
///     ncm: "22030000".into(),
///     cfop: 5102,
///     u_com: "UN".into(),
///     q_com: 2.0,
///     v_un_com: 50.0,
///     v_prod: 100.0,
///     icms: Icms::sn102(0, "400"),
///     pis: Pis::Nt { cst: "07".into() },
///     cofins: Cofins::Nt { cst: "07".into() },
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Det {
    pub c_prod: String,
    pub c_ean: String,
    pub x_prod: String,
    pub ncm: String,
    pub nve: Option<String>,
    pub extipi: Option<u8>,
    pub cest: Option<String>,
    pub cfop: u16,
    pub u_com: String,
    pub q_com: f64,
    pub v_un_com: f64,
    pub v_prod: f64,
    pub c_ean_trib: String,
    pub u_trib: String,
    pub q_trib: f64,
    pub v_un_trib: f64,
    pub v_frete: Option<f64>,
    pub v_seg: Option<f64>,
    pub v_desc: Option<f64>,
    pub v_outro: Option<f64>,
    pub ind_tot: u8,
    pub x_ped: Option<String>,
    pub n_item_ped: Option<String>,
    pub icms: Icms,
    pub ipi: Option<Ipi>,
    pub pis: Pis,
    pub cofins: Cofins,
    pub v_tot_trib: f64,
    pub inf_ad_prod: Option<String>,
    pub ibs_cbs: Option<IbsCbs>,
}

impl Default for Det {
    fn default() -> Self {
        Det {
            c_prod: "".to_string(),
            c_ean: "SEM GTIN".to_string(),
            x_prod: "".to_string(),
            ncm: "".to_string(),
            nve: None,
            extipi: None,
            cest: None,
            cfop: 5102,
            u_com: "".to_string(),
            q_com: 0.0,
            v_un_com: 0.0,
            v_prod: 0.0,
            c_ean_trib: "SEM GTIN".to_string(),
            u_trib: "UN".to_string(),
            q_trib: 0.0,
            v_un_trib: 0.0,
            v_frete: None,
            v_seg: None,
            v_desc: None,
            v_outro: None,
            ind_tot: 1,
            x_ped: None,
            n_item_ped: None,
            icms: Icms::Icms40 { orig: 0, cst: 40, v_icms_deson: None, mot_des_icms: None },
            ipi: None,
            pis: Pis::Outr,
            cofins: Cofins::Outr { cst: "99".to_string() },
            v_tot_trib: 0.0,
            inf_ad_prod: None,
            ibs_cbs: None,
        }
    }
}

// ─── Total ────────────────────────────────────────────────────────────────────

/// Totais da NF-e (`<total><ICMSTot>`).
///
/// Os campos `v_bc`, `v_icms`, `v_icms_deson`, `v_prod`, `v_desc`, `v_pis`,
/// `v_cofins`, `v_nf` e `v_tot_trib` são **calculados automaticamente** dos itens.
/// Informe apenas o que não deriva dos itens:
///
/// ```
/// use dfe::tipos::Total;
///
/// // Venda simples sem frete ou extras
/// let total = Total::default();
///
/// // Venda com frete e DIFAL
/// let total = Total {
///     v_frete: 15.00,
///     v_icms_uf_dest: 7.00,
///     v_icms_uf_remet: 3.00,
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Total {
    /// BC do ICMS ST — auto-calculado dos itens com ICMS10/30/70; informe apenas ST global.
    pub v_bc_st: f64,
    /// Valor do ICMS ST — auto-calculado dos itens; informe apenas ST global.
    pub v_st: f64,
    /// FCP (Fundo de Combate à Pobreza).
    pub v_fcp: f64,
    /// FCP retido por ST.
    pub v_fcpst: f64,
    /// FCP retido anteriormente por ST.
    pub v_fcpst_ret: f64,
    /// FCP diferencial de alíquota UF destino (DIFAL).
    pub v_fcpuf_dest: f64,
    /// ICMS diferencial de alíquota UF destino (DIFAL).
    pub v_icms_uf_dest: f64,
    /// ICMS diferencial de alíquota UF remetente (DIFAL).
    pub v_icms_uf_remet: f64,
    /// Frete global (não por item).
    pub v_frete: f64,
    /// Seguro global.
    pub v_seg: f64,
    /// Outras despesas globais.
    pub v_outro: f64,
    /// Imposto de Importação.
    pub v_ii: f64,
    /// IPI global — somado automaticamente aos itens com `Det.ipi`.
    pub v_ipi: f64,
    /// IPI devolvido.
    pub v_ipi_devol: f64,
}

// ─── Transp ───────────────────────────────────────────────────────────────────

/// Dados de transporte da NF-e (`<transp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transp {
    pub mod_frete: u8,
    pub cnpj: Option<String>,
    pub cpf: Option<String>,
    pub x_nome: Option<String>,
    pub ie: Option<u64>,
    pub x_end: Option<String>,
    pub x_mun: Option<String>,
    pub uf: Option<String>,
}

impl Default for Transp {
    fn default() -> Self {
        Transp {
            mod_frete: 9,
            cnpj: None, cpf: None, x_nome: None,
            ie: None, x_end: None, x_mun: None, uf: None,
        }
    }
}

// ─── Pag ──────────────────────────────────────────────────────────────────────

/// Dados de pagamento da NF-e (`<pag>`).
///
/// Para uma NF-e sem pagamento específico: `Pag::default()`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pag {
    pub ind_pag: u8,
    pub t_pag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_pag: Option<String>,
    pub v_pag: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tp_integra: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cnpj: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_band: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_aut: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_troco: Option<Decimal>,
}

impl Default for Pag {
    fn default() -> Self {
        Pag {
            ind_pag: 0,
            t_pag: "99".to_string(),
            x_pag: None,
            v_pag: 0.0,
            tp_integra: None,
            cnpj: None,
            t_band: None,
            c_aut: None,
            v_troco: None,
        }
    }
}

// ─── InfAdic ──────────────────────────────────────────────────────────────────

/// Informações adicionais da NF-e (`<infAdic>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfAdic {
    pub inf_ad_fisco: Option<String>,
    pub inf_cpl: Option<String>,
}

impl Default for InfAdic {
    fn default() -> Self {
        InfAdic { inf_ad_fisco: None, inf_cpl: None }
    }
}
