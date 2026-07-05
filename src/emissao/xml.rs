//! Montagem e assinatura do XML da NF-e/NFC-e (sem envio). Extraído do `mod.rs` na fase A7 (§2.1).

use crate::error::{DfeError, Result};
use crate::interno::cert::{DigestValue, RawPubKey, Sign};
use crate::interno::chave_acesso::ChaveAcesso;
use crate::interno::chave_acesso_props::ChaveAcessoProps;
use crate::interno::cleaner;
use crate::interno::cleaner::Strings;
use crate::interno::dates::get_current_date_time;
use crate::interno::dest_xml::DestTAG;
use crate::interno::validation::is_xml_valid;
use quick_xml::se::to_string;

use super::det::det_process;
use super::emit::{EmitProcess, EnderEmitProcess};
use super::ide::*;
use super::inf_adic::inf_adic_process;
use super::pag::pag_process;
use super::total::total_process;
use super::transp::transp_process;
use super::types::{NFeInterno, SignedNfe};

// Constrói e assina o XML da NF-e sem enviar à SEFAZ.
pub(super) async fn build_signed_xml(nfe: NFeInterno) -> Result<SignedNfe> {
    let cert_path = nfe.cert_path.clone();
    let cert_pass = nfe.cert_pass.clone();
    let ide_mod = nfe.ide.mod_;
    let ide_tp_amb = nfe.ide.tp_amb;
    let ide_c_uf = nfe.ide.c_uf;
    let id_csc = nfe.id_csc.clone();
    let csc = nfe.csc.clone();
    let inf_adic = nfe.inf_adic.clone();
    let referencias = nfe.referencias.clone();

    let codigo_numerico = ChaveAcesso::gerar_codigo_numerico(nfe.ide.c_nf.clone());
    let doc = match (nfe.emit.cnpj.as_ref(), nfe.emit.cpf.as_ref()) {
        (Some(cnpj), _) => cnpj.clone(),
        (None, Some(cpf)) => cpf.clone(),
        (None, None) => String::new(),
    };

    let ch_acc = ChaveAcesso::gerar_chave_acesso(ChaveAcessoProps {
        uf: nfe.ide.c_uf,
        doc,
        modelo: nfe.ide.mod_,
        serie: nfe.ide.serie,
        numero: nfe.ide.n_nf,
        tp_emis: nfe.ide.tp_emis,
        codigo_numerico: codigo_numerico.clone(),
    })?;
    let chave_acesso = ch_acc.chave;
    let dv = ch_acc.dv;

    let dh_emi = nfe.ide.dh_emi.clone().unwrap_or_else(get_current_date_time);
    let dh_sai_ent = nfe.ide.dh_sai_ent.clone().unwrap_or_else(get_current_date_time);

    let mut ide_process = IdeProcess {
        c_uf: nfe.ide.c_uf,
        c_nf: Some(codigo_numerico.clone()),
        nat_op: nfe.ide.nat_op.clone(),
        ind_pag: nfe.ide.ind_pag,
        mod_: nfe.ide.mod_.clone(),
        serie: nfe.ide.serie,
        n_nf: nfe.ide.n_nf,
        dh_emi: Some(dh_emi.clone()),
        dh_sai_ent: Some(dh_sai_ent),
        tp_nf: nfe.ide.tp_nf,
        id_dest: nfe.ide.id_dest,
        c_mun_fg: nfe.ide.c_mun_fg.clone(),
        tp_imp: nfe.ide.tp_imp,
        tp_emis: nfe.ide.tp_emis,
        c_dv: Some(dv),
        tp_amb: nfe.ide.tp_amb,
        fin_nfe: nfe.ide.fin_nfe,
        ind_final: nfe.ide.ind_final,
        ind_pres: nfe.ide.ind_pres,
        proc_emi: nfe.ide.proc_emi,
        ver_proc: nfe.ide.ver_proc.clone(),
    };
    if nfe.ide.mod_ == 65 {
        ide_process.dh_sai_ent = None;
        ide_process.c_nf = Some(codigo_numerico);
        ide_process.dh_emi = Some(dh_emi);
    }

    // Serializa <ide> e injeta tags <NFref> antes de </ide> quando houver referências
    let nf_refs_xml: String = referencias
        .iter()
        .map(|ch| format!("<NFref><refNFe>{}</refNFe></NFref>", ch))
        .collect();
    let ide_xml = {
        let base = to_string(&ide_process).unwrap_or_default();
        if nf_refs_xml.is_empty() {
            base
        } else {
            base.replace("</ide>", &format!("{}</ide>", nf_refs_xml))
        }
    };

    let emit_process = EmitProcess {
        cnpj: nfe.emit.cnpj.clone(),
        cpf: nfe.emit.cpf.clone(),
        x_nome: nfe.emit.x_nome.clone(),
        x_fant: nfe.emit.x_fant.clone(),
        ender_emit: EnderEmitProcess {
            x_lgr: nfe.emit.x_lgr.clone(),
            nro: nfe.emit.nro.clone(),
            x_bairro: nfe.emit.x_bairro.clone(),
            c_mun: nfe.emit.c_mun.clone(),
            x_mun: nfe.emit.x_mun.clone(),
            uf: nfe.emit.uf.clone(),
            cep: nfe.emit.cep.clone(),
            c_pais: nfe.emit.c_pais,
            x_pais: nfe.emit.x_pais.clone(),
        },
        ie: nfe.emit.ie.clone(),
        crt: nfe.emit.crt,
    };

    let dest_string = DestTAG::build(&nfe.dest, &nfe.ide)?;

    let dets = det_process(
        nfe.det.clone(), nfe.ide.mod_, nfe.ide.tp_amb,
        nfe.desconto_rateio.clone(), nfe.active_ibs_cbs.clone(),
    )?;
    let dets_total = dets.clone();

    let mut det_string = String::new();
    for (i, det) in dets.iter().enumerate() {
        let prod    = to_string(&det.prod).unwrap_or_default();
        let imposto = det.imposto.to_xml();
        let inf_ad  = det.inf_ad_prod.as_ref().map(|v| format!("<infAdProd>{}</infAdProd>", v)).unwrap_or_default();
        det_string.push_str(&format!(r#"<det nItem="{}">{}{}{}</det>"#, i + 1, prod, imposto, inf_ad));
    }

    let total_process_result = total_process(nfe.total.clone(), dets_total, nfe.ide.tp_amb, nfe.active_ibs_cbs.clone())?;
    let v_nf: f64 = total_process_result.icms_tot.v_nf.parse().unwrap_or(0.0);
    let transp_process_result = transp_process(nfe.transp.clone())?;
    let inf_adic_process_result = inf_adic_process(inf_adic)?;

    // pag_process recebe NFeInterno por valor — chamado por último
    let pag_process_result = pag_process(nfe, v_nf)?;

    let xml = format!(
        "<infNFe xmlns=\"http://www.portalfiscal.inf.br/nfe\" Id=\"NFe{}\" versao=\"4.00\">{}{}{}{}{}{}{}{}{}",
        chave_acesso,
        ide_xml,
        to_string(&emit_process).unwrap_or_default(),
        dest_string, det_string,
        to_string(&total_process_result).unwrap_or_default(),
        to_string(&transp_process_result).unwrap_or_default(),
        to_string(&pag_process_result).unwrap_or_default(),
        to_string(&inf_adic_process_result).unwrap_or_default(),
        "</infNFe>"
    );

    let xml = Strings::clear_xml_string(&xml);
    let digest_value = DigestValue::sha1(&xml)?;
    let x509_cert = RawPubKey::get_from_file(&cert_path, &cert_pass).await?;

    // A3b: montagem unificada em `interno::assinatura` (idêntica byte-a-byte à anterior;
    // Reference URI `#NFe{chave}`).
    let signed_info =
        crate::interno::assinatura::signed_info_xml(&format!("#NFe{}", chave_acesso), &digest_value);

    let signature_base64 = Sign::xml_string(&signed_info, &cert_path, &cert_pass).await?;
    let signature_xml =
        crate::interno::assinatura::signature_xml(&signed_info, &signature_base64, &x509_cert);

    let mut qrcode = String::new();
    if ide_mod == 65 {
        let url_base = if ide_tp_amb == 2 {
            "https://www.homologacao.nfce.fazenda.sp.gov.br/NFCeConsultaPublica/Paginas/ConsultaQRCode.aspx"
        } else {
            "https://www.nfce.fazenda.sp.gov.br/qrcode"
        };
        let versao_qr = "2";
        let ambiente = ide_tp_amb.to_string();
        let id_csc = id_csc.ok_or_else(|| DfeError::Validacao("ID do CSC não foi informado.".to_string()))?;
        let csc = csc.ok_or_else(|| DfeError::Validacao("CSC não foi informado.".to_string()))?;
        let c_hash = qrcode_hash(&chave_acesso, versao_qr, &ambiente, &id_csc, &csc)?;
        let url_consulta = if ide_tp_amb == 2 {
            "https://www.homologacao.nfce.fazenda.sp.gov.br/consulta"
        } else {
            "https://www.nfce.fazenda.sp.gov.br/consulta"
        };
        qrcode = cleaner::Strings::clear_xml_string(&format!(
            r#"<infNFeSupl><qrCode><![CDATA[{url_base}?p={chave_acesso}|{versao_qr}|{ambiente}|{id_csc}|{c_hash}]]></qrCode><urlChave>{url_consulta}</urlChave></infNFeSupl>"#
        ));
    }

    let nfe_xml = "<NFe xmlns=\"http://www.portalfiscal.inf.br/nfe\">".to_string()
        + &xml + &qrcode + &signature_xml + "</NFe>";

    let validated_xml = match is_xml_valid(&nfe_xml) {
        Ok(x) => x,
        Err(e) => return Err(DfeError::Validacao(format!("is_xml_valid: [{}]", e))),
    };

    Ok(SignedNfe { nfe_xml, validated_xml, cert_path, cert_pass, ide_mod, ide_tp_amb, ide_c_uf })
}

pub(super) fn qrcode_hash(chave_acesso: &str, versao_qr: &str, ambiente: &str, id_csc: &str, csc: &str) -> Result<String> {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(format!("{chave_acesso}|{versao_qr}|{ambiente}|{id_csc}{csc}").as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}
