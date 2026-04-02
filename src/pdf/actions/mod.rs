pub mod pdf_builder_80mm;

use crate::nfe::xml_extractor::structs::NFeProc;
use pdf_builder_80mm::PdfItem;

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
            .as_ref()
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
}
