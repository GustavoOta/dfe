use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Ide ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ide {
    pub c_uf: u16,
    pub c_nf: Option<String>,
    pub nat_op: String,
    pub ind_pag: Option<u8>,
    pub mod_: u32,
    pub serie: u32,
    pub n_nf: u64,
    pub dh_emi: Option<String>,
    pub dh_sai_ent: Option<String>,
    pub tp_nf: u8,
    pub id_dest: u8,
    pub c_mun_fg: String,
    pub tp_imp: u8,
    pub tp_emis: u8,
    pub tp_amb: u8,
    pub fin_nfe: u8,
    pub ind_final: u8,
    pub ind_pres: u8,
    pub proc_emi: u8,
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
    // Regime Normal (CRT=3)
    Icms00 { orig: u8, mod_bc: u8, v_bc: f64, p_icms: f64, v_icms: f64 },
    Icms40 { orig: u8, cst: u16, v_icms_deson: Option<f64>, mot_des_icms: Option<u16> },
    Icms60 {
        orig: u8,
        v_bcst_ret: Option<f64>,
        p_st: Option<f64>,
        v_icms_substituto: Option<f64>,
        v_icmsst_ret: Option<f64>,
    },
    Icms90 { orig: u8 },
    // Simples Nacional (CRT=1)
    Sn101 { orig: u8, p_cred_sn: f64, v_cred_icmssn: f64 },
    Sn102 { orig: u8, csosn: String },
    Sn500 { orig: u8, v_bcst_ret: Option<f64>, v_icmsst_ret: Option<f64> },
    Sn900 {
        orig: u8,
        mod_bc: Option<u8>,
        v_bc: Option<f64>,
        p_red_bc: Option<f64>,
        p_icms: Option<f64>,
        v_icms: Option<f64>,
        p_cred_sn: Option<f64>,
        v_cred_icmssn: Option<f64>,
    },
}

impl Icms {
    // ── Regime Normal (CRT=3) ────────────────────────────────────────────────

    pub fn icms00(orig: u8, mod_bc: u8, v_bc: f64, p_icms: f64, v_icms: f64) -> Self {
        Icms::Icms00 { orig, mod_bc, v_bc, p_icms, v_icms }
    }

    /// CST 40=Isenta | 41=Não tributada | 50=Suspensão
    pub fn icms40(orig: u8, cst: u16) -> Self {
        Icms::Icms40 { orig, cst, v_icms_deson: None, mot_des_icms: None }
    }

    /// ICMS-ST retido anteriormente — campos ST opcionais (NT 2011/004)
    pub fn icms60(orig: u8) -> Self {
        Icms::Icms60 { orig, v_bcst_ret: None, p_st: None, v_icms_substituto: None, v_icmsst_ret: None }
    }

    pub fn icms90(orig: u8) -> Self {
        Icms::Icms90 { orig }
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

    /// CSOSN 900 — outros; todos os campos de cálculo são opcionais
    pub fn sn900(orig: u8) -> Self {
        Icms::Sn900 { orig, mod_bc: None, v_bc: None, p_red_bc: None, p_icms: None, v_icms: None, p_cred_sn: None, v_cred_icmssn: None }
    }
}

// ─── Pis ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pis {
    Aliq { cst: String, v_bc: f64, p_pis: f64, v_pis: f64 },
    Outr,
    Nt { cst: String },
    Qtde { cst: String, q_bc_prod: f64, v_aliq_prod: f64, v_pis: f64 },
}

// ─── Cofins ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cofins {
    Aliq { cst: String, v_bc: f64, p_cofins: f64, v_cofins: f64 },
    Outr { cst: String },
    Nt { cst: String },
    Qtde { cst: String, q_bc_prod: f64, v_aliq_prod: f64, v_cofins: f64 },
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
            pis: Pis::Outr,
            cofins: Cofins::Outr { cst: "99".to_string() },
            v_tot_trib: 0.0,
            inf_ad_prod: None,
            ibs_cbs: None,
        }
    }
}

// ─── Total ────────────────────────────────────────────────────────────────────
//
// Campos auto-calculados a partir dos itens (não informar):
//   v_bc, v_icms, v_icms_deson, v_prod, v_desc, v_pis, v_cofins, v_nf, v_tot_trib
//
// Informar apenas o que não deriva dos itens:

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Total {
    // ST (ICMS10/ICMS70 — ainda não implementados)
    pub v_bc_st: f64,
    pub v_st: f64,
    // Fundo de Combate à Pobreza
    pub v_fcp: f64,
    pub v_fcpst: f64,
    pub v_fcpst_ret: f64,
    // Diferencial de alíquota UF destino
    pub v_fcpuf_dest: f64,
    pub v_icms_uf_dest: f64,
    pub v_icms_uf_remet: f64,
    // Despesas da NF (globais, não por item)
    pub v_frete: f64,
    pub v_seg: f64,
    pub v_outro: f64,
    // Impostos específicos
    pub v_ii: f64,
    pub v_ipi: f64,
    pub v_ipi_devol: f64,
}

// ─── Transp ───────────────────────────────────────────────────────────────────

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
