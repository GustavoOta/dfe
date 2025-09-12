use crate::nfe::xml_rules::dest::models::Dest;
use crate::nfe::xml_rules::ide::models::Ide;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFe {
    pub cert_path: String,
    pub cert_pass: String,
    pub id_csc: Option<String>,
    pub csc: Option<String>,
    pub ide: Ide,
    pub emit: Emit,
    pub dest: Option<Dest>,
    pub det: Vec<Det>,
    pub total: Total,
    pub transp: Transp,
    pub pag: Pag,
    pub inf_adic: Option<InfAdic>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Emit {
    /// CNPJ do emitente Ex: 12345678000123 String<14 Digits>
    pub cnpj: Option<String>,
    /// CPF do emitente Ex: 00011122233 String<11 Digits>
    pub cpf: Option<String>,
    /// Razão social do emitente Ex: Empresa Ltda String<2-60 Chars>
    pub x_nome: String,
    /// Nome fantasia do emitente Ex: Empresa do João String<1-60 Chars>
    pub x_fant: Option<String>,
    /// Logradouro do emitente Ex: Rua das Flores String<2-60 Chars>
    pub x_lgr: String,
    /// Número do endereço Ex: 1234 ou S/N String<1-60 Chars>
    pub nro: String,
    /// Complemento do endereço Ex: Sala 1 String<1-60 Chars>
    pub x_cpl: Option<String>,
    /// Bairro do emitente Ex: Centro String<2-60 Chars>
    pub x_bairro: String,
    /// Código do município Ex: 4205407 para Lages u<7 Digits>
    /// Utilizar a Tabela do IBGE (Anexo IX- Tabela de UF, Município e País).
    pub c_mun: String,
    /// Nome do município Ex: Lages String<2-60 Chars>
    pub x_mun: String,
    /// Sigla da UF Ex: SC String<2 Chars>
    pub uf: String,
    /// CEP do emitente Ex: 88509900 String<8 Digits>
    /// Informar os zeros não significativos. (NT 2011/004)
    pub cep: String,
    /// Código do país Ex: 1058 para Brasil u<4 Digits>
    pub c_pais: Option<u16>,
    /// Nome do país Ex: Brasil String<1-60 Chars>
    pub x_pais: Option<String>,
    /// Telefone do emitente Ex: 4999999999 u<6-14 Digits>
    /// Preencher com o Código DDD + número do telefone. Nas operações com exterior é permitido informar o código do país + código da localidade + número do telefone (v2.0)
    pub fone: Option<u64>,
    /// Inscrição estadual do emitente Ex: 123456789 u<2-14 Digits>
    pub ie: Option<String>,
    /// IE do Substituto Tributário da UF de destino da mercadoria, quando houver a retenção do ICMS ST para a UF de destino.
    pub iest: Option<u64>,
    /// Inscrição Municipal do Prestador de Serviço Ex: 123456789 u<1-15 Digits>
    /// Informado na emissão de NF-e conjugada, com itens de produtos sujeitos ao ICMS e itens de serviços sujeitos ao ISSQN.
    /// Campo opcional, pode ser informado na NF-e conjugada, com itens de produtos sujeitos ao ICMS e itens de serviços sujeitos ao ISSQN.
    pub im: Option<String>,
    /// CNAE fiscal Ex: 1234567 u<7 Digits>
    /// Campo Opcional. Pode ser informado quando a Inscrição Municipal (id:C19) for informada.
    pub cnae: u32,
    /// Código de Regime Tributário do emitente
    /// Ex: 1 para Simples Nacional
    /// Ex: 2 para Simples Nacional - excesso de sublimite de receita bruta
    /// Ex: 3 para Regime Normal (Lucro Presumido ou Lucro Real)
    /// Ex: 4 para MEI - Microempreendedor Individual
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
            c_pais: Some(1058),
            x_pais: Some("Brasil".to_string()),
            fone: None,
            ie: None,
            iest: None,
            im: None,
            cnae: 0000000,
            crt: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Det {
    /// Código do produto ou serviço String<1-60 Chars>
    /// Preencher com CFOP, caso se trate de itens não relacionados com mercadorias/produtos e que o contribuinte não possua codificação própria.
    /// Formato ”CFOP9999”.
    pub c_prod: String,
    /// Preencher com o código GTIN-8, GTIN-12, GTIN-13 ou GTIN-14 (antigos códigos EAN, UPC e DUN-14),
    /// não informar o conteúdo da TAG em caso de o produto não possuir este código.
    pub c_ean: String,
    /// Descrição do produto ou serviço String<1-120 Chars>
    pub x_prod: String,
    /// Código NCM com 8 dígitos ou 2 dígitos (gênero) ou 4 dígitos (capítulo 77) String<2-8 Chars>
    pub ncm: String,
    /// Codificação opcional que detalha alguns NCM. Formato: duas letras maiúsculas e 4 algarismos. Se a
    /// mercadoria se enquadrar em mais de uma codificação, informar até 8 codificações principais.
    /// Vide: Anexo XII.03 - Identificador NVE
    /// String<6 Chars>
    pub nve: Option<String>,
    /// Preencher de acordo com o código EX da TIPI. Em caso de serviço, não incluir a TAG.
    /// u<2-3 Chars>
    pub extipi: Option<u8>,
    /// Código CEST String<7 Chars>
    pub cest: Option<String>,
    /// Código Fiscal de Operações e Prestações String<4 Chars>
    pub cfop: u16,
    /// Unidade Comercial String<1-6 Chars>
    pub u_com: String,
    /// Quantidade Comercial  11v0-4 Informar a quantidade de comercialização do produto (v2.0).
    /// Decimal com até 4 dígitos, sendo 7 inteiros e 4 decimais. u<1-11.0-4 Chars>
    pub q_com: f64,
    /// Valor Unitário de Comercialização 11v0-10 Informar o valor unitário
    /// de comercialização do produto, campo meramente informativo, o contribuinte pode utilizar a precisão desejada (0-10 decimais).
    /// Para efeitos de cálculo, o valor unitário será obtido pela divisão do valor do produto pela quantidade comercial (NT 2013/003)
    /// u<1-11.0-10 Chars>
    pub v_un_com: f64,
    /// Valor Total Bruto dos Produtos ou Serviços
    /// 13v2 Informar o valor total bruto dos produtos ou serviços, campo
    pub v_prod: f64,
    /// Preencher com o código GTIN-8, GTIN-12, GTIN-13 ou GTIN-14 (antigos códigos EAN, UPC e DUN-14) da unidade tributável
    /// do produto, não informar o conteúdo da TAG em caso de o
    /// produto não possuir este código.
    pub c_ean_trib: String,
    /// Unidade Tributável 1-6 Informar a unidade de tributação do produto (v2.0).
    /// String<1-6 Chars>
    pub u_trib: String,
    /// Quantidade Tributável 11v0-4
    /// Informar a quantidade de tributação do produto (v2.0).
    /// Decimal com até 4 dígitos, sendo 7 inteiros e 4 decimais.
    /// u<1-11.0-4 Chars>
    pub q_trib: f64,
    /// Informar o valor unitário de tributação do produto, campo
    /// meramente informativo, o contribuinte pode utilizar a precisão
    /// desejada (0-10 decimais). Para efeitos de cálculo, o valor
    /// unitário será obtido pela divisão do valor do produto pela
    /// quantidade tributável (NT 2013/003)
    pub v_un_trib: f64,
    /// Valor Total do Frete 13v2 Informar o valor total do frete da mercadoria ou serviço.
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_frete: Option<f64>,
    /// Valor Total do Seguro 13v2 Informar o valor total do seguro da mercadoria ou serviço.
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_seg: Option<f64>,
    /// Valor do Desconto 13v2 Informar o valor total do desconto do item / produto.
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_desc: Option<f64>,
    /// Outras despesas acessórias 13v2 Informar o valor total de outras despesas acessórias.
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_outro: Option<f64>,
    /// Indica se valor do Item (vProd) entra no valor total da NF-e (vProd)
    /// 0=Valor do item (vProd) não compõe o valor total da NF-e (vProd)
    /// 1=Valor do item (vProd) compõe o valor total da NF-e (vProd)
    pub ind_tot: u8,
    /// xPed
    pub x_ped: Option<String>,
    /// nItemPed
    pub n_item_ped: Option<String>,
    // ******* ICMS ******* //
    pub icms: String,
    pub orig: Option<u8>,
    pub cst: Option<String>,
    pub mod_bc: Option<u8>,
    pub v_bc: Option<f64>,
    pub p_icms: Option<f64>,
    pub v_icms: Option<f64>,
    pub csosn: Option<String>,
    pub p_cred_sn: Option<f64>,
    pub v_cred_icmssn: Option<f64>,
    // ******* PIS ******* //
    pub pis: String,
    // PISAliq ---
    pub pis_cst: Option<String>,
    pub pis_v_bc: Option<f64>,
    pub pis_p_pis: Option<f64>,
    pub pis_v_pis: Option<f64>,
    // PisQtde ---
    // pub pis_cst: Option<String>,
    pub pis_q_bc_prod: Option<f64>,
    pub pis_v_aliq_prod: Option<f64>,
    // pub pis_v_pis: Option<f64>,
    // PISNT ---
    // pub pis_cst: Option<String>,
    // PISOutr ---
    // pis_cst: Option<String>,
    // -*-
    // pub pis_v_bc: Option<f64>,
    // pub pis_p_pis: Option<f64>,
    // -*-
    // PISST ---
    // -*-
    // pub pis_v_bc: Option<f64>,
    // pub pis_p_pis: Option<f64>,
    // -*-
    // pub pis_q_bc_prod: Option<f64>,
    // pub pis_v_aliq_prod: Option<f64>,
    // pub pis_v_pis: Option<f64>,

    // ******* COFINS ******* //
    pub cofins: String,
    // COFINSAliq ---
    pub cofins_cst: Option<String>,
    pub cofins_v_bc: Option<f64>,
    pub cofins_p_cofins: Option<f64>,
    pub cofins_v_cofins: Option<f64>,
    // COFINSQtde ---
    // pub cofins_cst: Option<String>,
    pub cofins_q_bc_prod: Option<f64>,
    pub cofins_v_aliq_prod: Option<f64>,
    // pub cofins_v_cofins: Option<f64>,
    // COFINSNT ---
    // pub cofins_cst: Option<String>,
    // COFINSOutr ---
    // pub cofins_cst: Option<String>,
    // -*-
    // pub cofins_v_bc: Option<f64>,
    // pub cofins_p_cofins: Option<f64>,
    // -*-
    // pub cofins_q_bc_prod: Option<f64>,
    // pub cofins_v_aliq_prod: Option<f64>,
    // pub cofins_v_cofins: Option<f64>,
    // TODO: ISSQN
    // IBS CBS : TESTES

    // TODO: impostoDevol
    /// vTotTrib 13v2 Valor aproximado total de tributos federais, estaduais e municipais.
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_tot_trib: f64,
    /// infAdProd
    pub inf_ad_prod: Option<String>,
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
            u_trib: "ng".to_string(),
            q_trib: 0.0,
            v_un_trib: 0.0,
            v_frete: None,
            v_seg: None,
            v_desc: None,
            v_outro: None,
            ind_tot: 1,
            x_ped: None,
            n_item_ped: None,
            icms: "ng".to_string(),
            orig: None,
            cst: None,
            mod_bc: None,
            v_bc: None,
            p_icms: None,
            v_icms: None,
            csosn: None,
            p_cred_sn: None,
            v_cred_icmssn: None,
            pis: "ng".to_string(),
            pis_cst: None,
            pis_p_pis: None,
            pis_q_bc_prod: None,
            pis_v_aliq_prod: None,
            pis_v_bc: None,
            pis_v_pis: None,
            cofins: "ng".to_string(),
            cofins_cst: None,
            cofins_p_cofins: None,
            cofins_q_bc_prod: None,
            cofins_v_aliq_prod: None,
            cofins_v_bc: None,
            cofins_v_cofins: None,
            v_tot_trib: 0.0,
            inf_ad_prod: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Total {
    /// Base de Cálculo do ICMS 13v2 Informar o valor da BC do ICMS
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_bc: f64,
    /// Valor Total do ICMS 13v2 Informar o valor total do ICMS
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_icms: f64,
    /// Valor Total do ICMS Desonerado 13v2 Informar o valor total do ICMS desonerado
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_icms_deson: f64,

    /// v_fcpuf_dest    
    /// Valor do ICMS UF Destino 13v2 Informar o valor do ICMS de UF destino
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_fcpuf_dest: f64,
    /// Valor do ICMS UF Destino 13v2 Informar o valor do ICMS de UF destino
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_icms_uf_dest: f64,
    /// Valor do ICMS UF Remetente 13v2 Informar o valor do ICMS de UF remetente
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_icms_uf_remet: f64,
    /// vFCP
    pub v_fcp: f64,
    /// Base de Cálculo do ICMS ST 13v2 Informar o valor da BC do ICMS ST
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_bc_st: f64,
    /// Valor Total do ICMS ST 13v2 Informar o valor total do ICMS ST
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_st: f64,
    /// vFCPST
    pub v_fcpst: f64,
    // vFCPSTRet
    pub v_fcpst_ret: f64,
    /// Valor Total dos Produtos e Serviços 13v2 Informar o valor total dos produtos e serviços
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_prod: f64,
    /// Valor Total do Frete 13v2 Informar o valor total do frete
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_frete: f64,
    /// Valor Total do Seguro 13v2 Informar o valor total do seguro
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_seg: f64,
    /// Valor Total do Desconto 13v2 Informar o valor total do desconto
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_desc: f64,
    /// Valor Total do II 13v2 Informar o valor total do II
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_ii: f64,
    /// Valor Total do IPI 13v2 Informar o valor total do IPI
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_ipi: f64,
    // vIPIDevol
    pub v_ipi_devol: f64,
    /// Valor do PIS 13v2 Informar o valor do PIS
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_pis: f64,
    /// Valor da COFINS 13v2 Informar o valor da COFINS
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_cofins: f64,
    /// Outras Despesas acessórias 13v2 Informar o valor de outras despesas acessórias
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_outro: f64,
    /// Valor Total da NF-e 13v2 Informar o valor total da NF-e
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_nf: f64,
    /// Valor aproximado total de tributos federais, estaduais e municipais.
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_tot_trib: f64,
}

impl Default for Total {
    fn default() -> Self {
        Total {
            v_bc: 0.0,
            v_icms: 0.0,
            v_icms_deson: 0.0,
            v_fcpuf_dest: 0.0,
            v_icms_uf_dest: 0.0,
            v_icms_uf_remet: 0.0,
            v_fcp: 0.0,
            v_bc_st: 0.0,
            v_st: 0.0,
            v_fcpst: 0.0,
            v_fcpst_ret: 0.0,
            v_prod: 0.0,
            v_frete: 0.0,
            v_seg: 0.0,
            v_desc: 0.0,
            v_ii: 0.0,
            v_ipi: 0.0,
            v_ipi_devol: 0.0,
            v_pis: 0.0,
            v_cofins: 0.0,
            v_outro: 0.0,
            v_nf: 0.0,
            v_tot_trib: 0.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transp {
    /// Modalidade do frete
    /// 0=Por conta do emitente;
    /// 1=Por conta do destinatário/remetente;
    /// 2=Por conta de terceiros;
    /// 9=Sem frete.
    pub mod_frete: u8,
    /// CNPJ do transportador
    pub cnpj: Option<String>,
    /// CPF do transportador
    pub cpf: Option<String>,
    /// Razão Social ou nome do transportador
    pub x_nome: Option<String>,
    /// Inscrição Estadual do transportador
    pub ie: Option<u64>,
    /// Endereço completo do transportador
    pub x_end: Option<String>,
    /// Nome do município
    pub x_mun: Option<String>,
    /// Sigla da UF
    pub uf: Option<String>,
}

impl Default for Transp {
    fn default() -> Self {
        Transp {
            mod_frete: 9,
            cnpj: None,
            cpf: None,
            x_nome: None,
            ie: None,
            x_end: None,
            x_mun: None,
            uf: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pag {
    /// Indicador de forma de pagamento
    /// 0=Pagamento à vista;
    /// 1=Pagamento a prazo;
    pub ind_pag: u8,
    /// Forma de pagamento
    /// 01=Dinheiro
    /// 02=Cheque
    /// 03=Cartão de Crédito
    /// 04=Cartão de Débito
    /// 05=Crédito Loja
    /// 10=Vale Alimentação
    /// 11=Vale Refeição
    /// 12=Vale Presente
    /// 13=Vale Combustível
    /// 15=Boleto Bancário
    /// 16=Depósito Bancário
    /// 17=Pagamento Instantâneo (PIX)
    /// 18=Transferência bancária, Carteira Digital
    /// 19=Programa de fidelidade, Cashback, Crédito Virtual
    /// 90=Sem pagamento
    /// 99=Outros
    pub t_pag: String,
    /// Descrição do pagamento
    /// Informar uma descrição do pagamento
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_pag: Option<String>,
    /// Valor do pagamento
    pub v_pag: f64,
    // Campos para cartão (opcionais)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tp_integra: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cnpj: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_band: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_aut: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v_troco: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfAdic {
    /// Informações Adicionais de Interesse do Contribuinte
    pub inf_ad_fisco: Option<String>,
    /// Informações Complementares de interesse do Contribuinte
    pub inf_cpl: Option<String>,
}

impl Default for InfAdic {
    fn default() -> Self {
        InfAdic {
            inf_ad_fisco: None,
            inf_cpl: None,
        }
    }
}
