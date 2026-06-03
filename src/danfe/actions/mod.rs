pub mod pdf_builder_80mm;
pub mod pdf_builder_a4;
pub mod pdf_builder_nfce_80mm;

use crate::xml_extractor::structs::NFeProc;
use pdf_builder_80mm::PdfItem;
use pdf_builder_a4::PdfItemA4;
use pdf_builder_nfce_80mm::NfcePayment;

pub struct DanfeBuilderActions;

impl DanfeBuilderActions {
    pub async fn generate_55_80mm(nfe_proc: NFeProc) -> Result<Vec<u8>, String> {
        let inf = &nfe_proc.nfe.inf_nfe;
        let ide = &inf.ide;
        let emit = &inf.emit;
        let dest = &inf.dest;
        let prot = &nfe_proc.prot_nfe.inf_prot;

        // Campos obrigatórios NT 2020.004 §3.12.4

        // Chave de acesso (Id sem prefixo "NFe")
        let chave_acesso = inf
            .id
            .as_deref()
            .unwrap_or("")
            .strip_prefix("NFe")
            .unwrap_or(inf.id.as_deref().unwrap_or(""))
            .to_string();

        // Protocolo de Autorização de Uso
        let n_prot = prot
            .as_ref()
            .and_then(|p| p.n_prot.clone())
            .unwrap_or_default();
        let dh_recbto = prot
            .as_ref()
            .and_then(|p| p.dh_recbto.clone())
            .unwrap_or_default();

        // b) Emitente
        let emit_x_nome = emit
            .x_fant
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .or(emit.x_nome.as_deref().filter(|v| !v.trim().is_empty()))
            .unwrap_or_default()
            .to_string();
        let emit_uf = emit.ender_emit.uf.clone().unwrap_or_default();
        let emit_cnpj = emit.cnpj.clone().unwrap_or_default();
        let emit_ie = emit.ie.clone().unwrap_or_default();

        // c) Dados gerais
        let tp_nf = ide.tp_nf.clone().unwrap_or_default(); // 0=Entrada, 1=Saída
        let serie = ide.serie.clone().unwrap_or_default();
        let n_nf = ide.n_nf.clone().unwrap_or_default();
        let dh_emi = ide.dh_emi.clone().unwrap_or_default();

        // d) Destinatário
        let dest_x_nome = dest
            .as_ref()
            .and_then(|d| d.x_nome.clone())
            .unwrap_or_default();
        let dest_cnpj_cpf = dest
            .as_ref()
            .and_then(|d| d.cnpj.clone().or_else(|| d.cpf.clone()))
            .unwrap_or_default();
        let dest_uf = dest
            .as_ref()
            .and_then(|d| d.ender_dest.as_ref())
            .and_then(|e| e.uf.clone())
            .unwrap_or_default();
        let dest_ie = dest.as_ref().and_then(|d| d.ie.clone()).unwrap_or_default();

        // e) Valor total
        let v_nf = inf
            .total
            .icms_tot
            .as_ref()
            .and_then(|t| t.v_nf.clone())
            .unwrap_or_default();

        // Observação do contribuinte
        let inf_cpl = inf.inf_adic.inf_cpl.clone().unwrap_or_default();

        // Forma de pagamento
        let t_pag = inf
            .pag
            .det_pag
            .first()
            .and_then(|d| d.t_pag.clone())
            .unwrap_or_default();

        // Itens / Produtos
        let items: Vec<PdfItem> = inf
            .det
            .iter()
            .map(|det| {
                let prod = &det.prod;
                PdfItem {
                    n_item: det.n_item.clone().unwrap_or_default(),
                    x_prod: prod.x_prod.clone().unwrap_or_default(),
                    q_com: prod.q_com.clone().unwrap_or_default(),
                    u_com: prod.u_com.clone().unwrap_or_default(),
                    v_un_com: prod.v_un_com.clone().unwrap_or_default(),
                    v_prod: prod.v_prod.clone().unwrap_or_default(),
                }
            })
            .collect();

        pdf_builder_80mm::build_pdf_80mm(
            &chave_acesso,
            &n_prot,
            &dh_recbto,
            &emit_x_nome,
            &emit_uf,
            &emit_cnpj,
            &emit_ie,
            &tp_nf,
            &serie,
            &n_nf,
            &dh_emi,
            &dest_x_nome,
            &dest_cnpj_cpf,
            &dest_uf,
            &dest_ie,
            &v_nf,
            &t_pag,
            &inf_cpl,
            &items,
        )
    }

    pub async fn generate_55_a4(nfe_proc: NFeProc, logo: Option<Vec<u8>>) -> Result<Vec<u8>, String> {
        let inf = &nfe_proc.nfe.inf_nfe;
        let ide = &inf.ide;
        let emit = &inf.emit;
        let dest = &inf.dest;
        let prot = &nfe_proc.prot_nfe.inf_prot;

        // Chave de acesso
        let chave_acesso = inf
            .id
            .as_deref()
            .unwrap_or("")
            .strip_prefix("NFe")
            .unwrap_or(inf.id.as_deref().unwrap_or(""))
            .to_string();

        // Protocolo
        let n_prot = prot
            .as_ref()
            .and_then(|p| p.n_prot.clone())
            .unwrap_or_default();
        let dh_recbto = prot
            .as_ref()
            .and_then(|p| p.dh_recbto.clone())
            .unwrap_or_default();

        // Ide
        let tp_nf = ide.tp_nf.clone().unwrap_or_default();
        let n_nf = ide.n_nf.clone().unwrap_or_default();
        let serie = ide.serie.clone().unwrap_or_default();
        let dh_emi = ide.dh_emi.clone().unwrap_or_default();
        let dh_sai_ent = ide.dh_sai_ent.clone().unwrap_or_default();
        let nat_op = ide.nat_op.clone().unwrap_or_default();
        let tp_amb = ide.tp_amb.clone().unwrap_or_default();

        // Emitente
        let emit_x_nome = emit.x_nome.as_deref().unwrap_or_default().to_string();
        let emit_cnpj = emit.cnpj.clone().unwrap_or_default();
        let emit_ie = emit.ie.clone().unwrap_or_default();
        let emit_uf = emit.ender_emit.uf.clone().unwrap_or_default();
        let emit_x_lgr = emit.ender_emit.x_lgr.clone().unwrap_or_default();
        let emit_nro = emit.ender_emit.nro.clone().unwrap_or_default();
        let emit_x_bairro = emit.ender_emit.x_bairro.clone().unwrap_or_default();
        let emit_x_mun = emit.ender_emit.x_mun.clone().unwrap_or_default();
        let emit_cep = emit.ender_emit.cep.clone().unwrap_or_default();
        let emit_fone = emit.ender_emit.fone.clone().unwrap_or_default();

        // Destinatário
        let dest_x_nome = dest
            .as_ref()
            .and_then(|d| d.x_nome.clone())
            .unwrap_or_default();
        let dest_cnpj_cpf = dest
            .as_ref()
            .and_then(|d| d.cnpj.clone().or_else(|| d.cpf.clone()))
            .unwrap_or_default();
        let dest_ie = dest.as_ref().and_then(|d| d.ie.clone()).unwrap_or_default();
        let dest_ender = dest.as_ref().and_then(|d| d.ender_dest.as_ref());
        let dest_x_lgr = dest_ender.and_then(|e| e.x_lgr.clone()).unwrap_or_default();
        let dest_nro = dest_ender.and_then(|e| e.nro.clone()).unwrap_or_default();
        let dest_x_bairro = dest_ender
            .and_then(|e| e.x_bairro.clone())
            .unwrap_or_default();
        let dest_x_mun = dest_ender.and_then(|e| e.x_mun.clone()).unwrap_or_default();
        let dest_uf = dest_ender.and_then(|e| e.uf.clone()).unwrap_or_default();
        let dest_cep = dest_ender.and_then(|e| e.cep.clone()).unwrap_or_default();

        // Totais
        let icms_tot = inf.total.icms_tot.as_ref();
        let v_bc = icms_tot.and_then(|t| t.v_bc.clone()).unwrap_or_default();
        let v_icms = icms_tot.and_then(|t| t.v_icms.clone()).unwrap_or_default();
        let v_bc_st = icms_tot.and_then(|t| t.v_bc_st.clone()).unwrap_or_default();
        let v_st = icms_tot.and_then(|t| t.v_st.clone()).unwrap_or_default();
        let v_prod = icms_tot.and_then(|t| t.v_prod.clone()).unwrap_or_default();
        let v_frete = icms_tot.and_then(|t| t.v_frete.clone()).unwrap_or_default();
        let v_seg = icms_tot.and_then(|t| t.v_seg.clone()).unwrap_or_default();
        let v_desc = icms_tot.and_then(|t| t.v_desc.clone()).unwrap_or_default();
        let v_outro = icms_tot.and_then(|t| t.v_outro.clone()).unwrap_or_default();
        let v_ipi = icms_tot.and_then(|t| t.v_ipi.clone()).unwrap_or_default();
        let v_nf = icms_tot.and_then(|t| t.v_nf.clone()).unwrap_or_default();

        // Transporte
        let mod_frete = inf.transp.mod_frete.as_deref().unwrap_or("");
        let mod_frete_label = match mod_frete {
            "0" => "0-Emitente",
            "1" => "1-Destinatário",
            "2" => "2-Terceiros",
            "9" => "9-Sem frete",
            other => other,
        };

        // Observação
        let inf_cpl = inf.inf_adic.inf_cpl.clone().unwrap_or_default();

        // Itens
        let items: Vec<PdfItemA4> = inf
            .det
            .iter()
            .map(|det| {
                let prod = &det.prod;
                let imposto = &det.imposto;
                let icms = imposto.icms.as_ref().and_then(|i| i.icms00.as_ref());
                PdfItemA4 {
                    n_item: det.n_item.clone().unwrap_or_default(),
                    c_prod: prod.c_prod.clone().unwrap_or_default(),
                    x_prod: prod.x_prod.clone().unwrap_or_default(),
                    ncm: prod.ncm.clone().unwrap_or_default(),
                    cfop: prod.cfop.clone().unwrap_or_default(),
                    u_com: prod.u_com.clone().unwrap_or_default(),
                    q_com: prod.q_com.clone().unwrap_or_default(),
                    v_un_com: prod.v_un_com.clone().unwrap_or_default(),
                    v_prod: prod.v_prod.clone().unwrap_or_default(),
                    v_desc: String::new(),
                    v_ipi: String::new(),
                    p_icms: icms.and_then(|i| i.p_icms.clone()).unwrap_or_default(),
                    v_icms: icms.and_then(|i| i.v_icms.clone()).unwrap_or_default(),
                    p_ipi: String::new(),
                }
            })
            .collect();

        pdf_builder_a4::build_pdf_a4(
            logo.as_deref(),
            &chave_acesso,
            &n_prot,
            &dh_recbto,
            &tp_nf,
            &n_nf,
            &serie,
            &dh_emi,
            &dh_sai_ent,
            &nat_op,
            &tp_amb,
            &emit_x_nome,
            &emit_cnpj,
            &emit_ie,
            "", // emit_iest
            &emit_x_lgr,
            &emit_nro,
            &emit_x_bairro,
            &emit_x_mun,
            &emit_uf,
            &emit_cep,
            &emit_fone,
            &dest_x_nome,
            &dest_cnpj_cpf,
            &dest_ie,
            &dest_x_lgr,
            &dest_nro,
            &dest_x_bairro,
            &dest_x_mun,
            &dest_uf,
            &dest_cep,
            "", // dest_fone
            &v_bc,
            &v_icms,
            &v_bc_st,
            &v_st,
            &v_prod,
            &v_frete,
            &v_seg,
            &v_desc,
            &v_outro,
            &v_ipi,
            &v_nf,
            "", // transp_x_nome
            "", // transp_cnpj_cpf
            mod_frete_label,
            "", // transp_uf
            "", // placa
            "", // marca
            "", // vol_qtd
            "", // vol_esp
            "", // vol_peso_b
            "", // vol_peso_l
            &inf_cpl,
            "", // inf_fisco
            &items,
        )
    }

    pub async fn generate_65_80mm(nfe_proc: NFeProc, qr_side: bool) -> Result<Vec<u8>, String> {
        let inf = &nfe_proc.nfe.inf_nfe;
        let ide = &inf.ide;
        let emit = &inf.emit;
        let dest = &inf.dest;
        let prot = &nfe_proc.prot_nfe.inf_prot;
        let supl = &nfe_proc.nfe.inf_nfe_supl;

        // Chave de acesso (Id sem prefixo "NFe")
        let chave_acesso = inf
            .id
            .as_deref()
            .unwrap_or("")
            .strip_prefix("NFe")
            .unwrap_or(inf.id.as_deref().unwrap_or(""))
            .to_string();

        // Protocolo
        let n_prot = prot
            .as_ref()
            .and_then(|p| p.n_prot.clone())
            .unwrap_or_default();
        let dh_recbto = prot
            .as_ref()
            .and_then(|p| p.dh_recbto.clone())
            .unwrap_or_default();

        // Emitente
        let emit_x_nome = emit
            .x_fant
            .as_deref()
            .filter(|v| !v.trim().is_empty())
            .or(emit.x_nome.as_deref().filter(|v| !v.trim().is_empty()))
            .unwrap_or_default()
            .to_string();
        let emit_cnpj = emit.cnpj.clone().unwrap_or_default();
        let emit_ie = emit.ie.clone().unwrap_or_default();
        let emit_uf = emit.ender_emit.uf.clone().unwrap_or_default();
        let emit_x_lgr = emit.ender_emit.x_lgr.clone().unwrap_or_default();
        let emit_nro = emit.ender_emit.nro.clone().unwrap_or_default();
        let emit_x_bairro = emit.ender_emit.x_bairro.clone().unwrap_or_default();
        let emit_x_mun = emit.ender_emit.x_mun.clone().unwrap_or_default();

        // Ambiente / série / número / data
        let tp_amb = ide.tp_amb.clone().unwrap_or_default();
        let serie = ide.serie.clone().unwrap_or_default();
        let n_nf = ide.n_nf.clone().unwrap_or_default();
        let dh_emi = ide.dh_emi.clone().unwrap_or_default();

        // Destinatário
        let dest_cpf_cnpj = dest
            .as_ref()
            .and_then(|d| d.cnpj.clone().or_else(|| d.cpf.clone()))
            .unwrap_or_default();
        let dest_x_nome = dest
            .as_ref()
            .and_then(|d| d.x_nome.clone())
            .unwrap_or_default();

        // Totais
        let icms_tot = inf.total.icms_tot.as_ref();
        let v_nf = icms_tot.and_then(|t| t.v_nf.clone()).unwrap_or_default();
        let v_desc = icms_tot.and_then(|t| t.v_desc.clone()).unwrap_or_default();
        let v_prod = icms_tot.and_then(|t| t.v_prod.clone()).unwrap_or_default();
        // Soma v_tot_trib de cada item; fallback para ICMSTot (XML pode tê-lo zerado)
        let v_tot_trib_items: f64 = inf.det.iter()
            .filter_map(|det| det.imposto.v_tot_trib.as_deref())
            .filter_map(|s| s.parse::<f64>().ok())
            .sum();
        let v_tot_trib = if v_tot_trib_items > 0.0 {
            format!("{:.2}", v_tot_trib_items)
        } else {
            icms_tot.and_then(|t| t.v_tot_trib.clone()).unwrap_or_default()
        };

        // Troco
        let v_troco = inf.pag.v_troco.clone().unwrap_or_default();

        // Formas de pagamento
        let payments: Vec<NfcePayment> = inf
            .pag
            .det_pag
            .iter()
            .map(|d| NfcePayment {
                t_pag: d.t_pag.clone().unwrap_or_default(),
                v_pag: d.v_pag.clone().unwrap_or_default(),
            })
            .collect();

        // Observação
        let inf_cpl = inf.inf_adic.inf_cpl.clone().unwrap_or_default();

        // QR Code URL e URL de consulta
        let qr_code_url = supl
            .as_ref()
            .and_then(|s| s.qr_code.clone())
            .unwrap_or_default();
        // Itens
        let items: Vec<PdfItem> = inf
            .det
            .iter()
            .map(|det| {
                let prod = &det.prod;
                PdfItem {
                    n_item: det.n_item.clone().unwrap_or_default(),
                    x_prod: prod.x_prod.clone().unwrap_or_default(),
                    q_com: prod.q_com.clone().unwrap_or_default(),
                    u_com: prod.u_com.clone().unwrap_or_default(),
                    v_un_com: prod.v_un_com.clone().unwrap_or_default(),
                    v_prod: prod.v_prod.clone().unwrap_or_default(),
                }
            })
            .collect();

        pdf_builder_nfce_80mm::build_pdf_nfce_80mm(
            &chave_acesso,
            &n_prot,
            &dh_recbto,
            &emit_x_nome,
            &emit_cnpj,
            &emit_ie,
            &emit_uf,
            &emit_x_lgr,
            &emit_nro,
            &emit_x_bairro,
            &emit_x_mun,
            &tp_amb,
            &serie,
            &n_nf,
            &dh_emi,
            &dest_cpf_cnpj,
            &dest_x_nome,
            &v_nf,
            &v_desc,
            &v_prod,
            &v_troco,
            &v_tot_trib,
            &payments,
            &inf_cpl,
            &items,
            &qr_code_url,
            qr_side,
        )
    }
}
