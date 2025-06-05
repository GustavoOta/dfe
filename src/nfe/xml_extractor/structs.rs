use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "nfeProc")]
pub struct NFeProc {
    #[serde(rename = "@versao")]
    pub versao: String,
    #[serde(rename = "NFe")]
    pub nfe: NFe,
    #[serde(rename = "protNFe")]
    pub prot_nfe: ProtNFe,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFe {
    #[serde(rename = "infNFe")]
    pub inf_nfe: InfNFe,
}

/// InfNFe *****
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfNFe {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "ide")]
    pub ide: Ide,
    #[serde(rename = "emit")]
    pub emit: Emit,
    #[serde(rename = "dest")]
    pub dest: Option<Dest>,
    #[serde(rename = "det")]
    pub det: Vec<Det>, // Detalhes dos produtos, pode ser uma lista
    #[serde(rename = "total")]
    pub total: Total,
    #[serde(rename = "transp")]
    pub transp: Transp,
    #[serde(rename = "pag")]
    pub pag: Pag,
    #[serde(rename = "infAdic")]
    pub inf_adic: InfAdic,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ide {
    #[serde(rename = "cUF")]
    pub c_uf: Option<String>,
    #[serde(rename = "cNF")]
    pub c_nf: Option<String>,
    #[serde(rename = "natOp")]
    pub nat_op: Option<String>,
    #[serde(rename = "mod")]
    pub mod_: Option<String>,
    #[serde(rename = "serie")]
    pub serie: Option<String>,
    #[serde(rename = "nNF")]
    pub n_nf: Option<String>,
    #[serde(rename = "dhEmi")]
    pub dh_emi: Option<String>,
    #[serde(rename = "dhSaiEnt")]
    pub dh_sai_ent: Option<String>,
    #[serde(rename = "tpNF")]
    pub tp_nf: Option<String>,
    #[serde(rename = "idDest")]
    pub id_dest: Option<String>,
    #[serde(rename = "cMunFG")]
    pub c_mun_fg: Option<String>,
    #[serde(rename = "tpImp")]
    pub tp_imp: Option<String>,
    #[serde(rename = "tpEmis")]
    pub tp_emis: Option<String>,
    #[serde(rename = "cDV")]
    pub c_dv: Option<String>,
    #[serde(rename = "tpAmb")]
    pub tp_amb: Option<String>,
    #[serde(rename = "finNFe")]
    pub fin_nfe: Option<String>,
    #[serde(rename = "indFinal")]
    pub ind_final: Option<String>,
    #[serde(rename = "indPres")]
    pub ind_pres: Option<String>,
    #[serde(rename = "procEmi")]
    pub proc_emi: Option<String>,
    #[serde(rename = "verProc")]
    pub ver_proc: Option<String>,
    // Outros campos podem ser adicionados aqui
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Emit {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "xFant")]
    pub x_fant: Option<String>,
    #[serde(rename = "enderEmit")]
    pub ender_emit: EnderEmit,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "CRT")]
    pub crt: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnderEmit {
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dest {
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "xNome")]
    pub x_nome: Option<String>,
    #[serde(rename = "enderDest")]
    pub ender_dest: Option<EnderDest>,
    #[serde(rename = "indIEDest")]
    pub ind_ie_dest: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EnderDest {
    #[serde(rename = "xLgr")]
    pub x_lgr: Option<String>,
    #[serde(rename = "nro")]
    pub nro: Option<String>,
    #[serde(rename = "xBairro")]
    pub x_bairro: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "xMun")]
    pub x_mun: Option<String>,
    #[serde(rename = "UF")]
    pub uf: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: Option<String>,
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "xPais")]
    pub x_pais: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Det {
    #[serde(rename = "@nItem")]
    pub n_item: Option<String>,
    #[serde(rename = "prod")]
    pub prod: Prod,
    #[serde(rename = "imposto")]
    pub imposto: Imposto,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prod {
    #[serde(rename = "cProd")]
    pub c_prod: Option<String>,
    #[serde(rename = "cEAN")]
    pub c_ean: Option<String>,
    #[serde(rename = "xProd")]
    pub x_prod: Option<String>,
    #[serde(rename = "NCM")]
    pub ncm: Option<String>,
    #[serde(rename = "CFOP")]
    pub cfop: Option<String>,
    #[serde(rename = "uCom")]
    pub u_com: Option<String>,
    #[serde(rename = "qCom")]
    pub q_com: Option<String>,
    #[serde(rename = "vUnCom")]
    pub v_un_com: Option<String>,
    #[serde(rename = "vProd")]
    pub v_prod: Option<String>,
    #[serde(rename = "cEANTrib")]
    pub c_ean_trib: Option<String>,
    #[serde(rename = "uTrib")]
    pub u_trib: Option<String>,
    #[serde(rename = "qTrib")]
    pub q_trib: Option<String>,
    #[serde(rename = "vUnTrib")]
    pub v_un_trib: Option<String>,
    #[serde(rename = "indTot")]
    pub ind_tot: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Imposto {
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: Option<String>,
    #[serde(rename = "ICMS")]
    pub icms: Option<ICMS>,
    #[serde(rename = "PIS")]
    pub pis: Option<PIS>,
    #[serde(rename = "COFINS")]
    pub cofins: Option<COFINS>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS {
    #[serde(rename = "ICMS00")]
    pub icms00: Option<ICMS00>,
    // Outros tipos de ICMS podem ser adicionados aqui
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMS00 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PIS {
    #[serde(rename = "PISAliq")]
    pub pis_aliq: Option<PISAliq>,
    // Outros tipos de PIS podem ser adicionados aqui
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PISAliq {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pPIS")]
    pub p_pis: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINS {
    #[serde(rename = "COFINSAliq")]
    pub cofins_aliq: Option<COFINSAliq>,
    // Outros tipos de COFINS podem ser adicionados aqui
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct COFINSAliq {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pCOFINS")]
    pub p_cofins: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Total {
    #[serde(rename = "ICMSTot")]
    pub icms_tot: Option<ICMSTot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ICMSTot {
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vICMSDeson")]
    pub v_icms_deson: Option<String>,
    #[serde(rename = "vFCPUFDest")]
    pub v_fcp_uf_dest: Option<String>,
    #[serde(rename = "vICMSUFDest")]
    pub v_icms_uf_dest: Option<String>,
    #[serde(rename = "vICMSUFRemet")]
    pub v_icms_uf_remet: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bc_st: Option<String>,
    #[serde(rename = "vST")]
    pub v_st: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcp_st: Option<String>,
    #[serde(rename = "vFCPSTRet")]
    pub v_fcp_st_ret: Option<String>,
    #[serde(rename = "vProd")]
    pub v_prod: Option<String>,
    #[serde(rename = "vFrete")]
    pub v_frete: Option<String>,
    #[serde(rename = "vSeg")]
    pub v_seg: Option<String>,
    #[serde(rename = "vDesc")]
    pub v_desc: Option<String>,
    #[serde(rename = "vII")]
    pub v_ii: Option<String>,
    #[serde(rename = "vIPI")]
    pub v_ipi: Option<String>,
    #[serde(rename = "vIPIDevol")]
    pub v_ipi_devol: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
    #[serde(rename = "vOutro")]
    pub v_outro: Option<String>,
    #[serde(rename = "vNF")]
    pub v_nf: Option<String>,
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transp {
    #[serde(rename = "modFrete")]
    pub mod_frete: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pag {
    #[serde(rename = "detPag")]
    pub det_pag: Option<DetPag>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DetPag {
    #[serde(rename = "indPag")]
    pub ind_pag: Option<String>,
    #[serde(rename = "tPag")]
    pub t_pag: Option<String>,
    #[serde(rename = "vPag")]
    pub v_pag: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfAdic {
    #[serde(rename = "infCpl")]
    pub inf_cpl: Option<String>,
}

/// ProtNFe *****
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtNFe {
    #[serde(rename = "infProt")]
    pub inf_prot: Option<InfProt>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InfProt {
    #[serde(rename = "tpAmb")]
    pub tp_amb: Option<String>,
    #[serde(rename = "verAplic")]
    pub ver_aplic: Option<String>,
    #[serde(rename = "chNFe")]
    pub ch_nfe: Option<String>,
    #[serde(rename = "dhRecbto")]
    pub dh_recbto: Option<String>,
    #[serde(rename = "nProt")]
    pub n_prot: Option<String>,
    #[serde(rename = "digVal")]
    pub dig_val: Option<String>,
    #[serde(rename = "cStat")]
    pub c_stat: Option<String>,
    #[serde(rename = "xMotivo")]
    pub x_motivo: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XMLExtractorError {
    pub error: u8,
    pub msg: String,
    pub data: Option<String>,
}
