use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFe {
    pub cert_path: String,
    pub cert_pass: String,
    pub ide: Ide,
    pub emit: Emit,
    pub dest: Dest,
    pub det: Vec<Det>,
    pub total: Total,
    pub transp: Transp,
    pub pag: Pag,
    pub inf_adic: Option<InfAdic>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ide {
    /// Código da UF do emitente do Documento Fiscal
    /// Ex: 35 = São Paulo
    pub c_uf: u16,

    /// Código Numérico que compõe a Chave de Acesso de 8 dígitos gerado pelo emitente para cada NF-e para evitar acessos indevidos da NF-e.
    /// Default: None (Random generated)
    pub c_nf: Option<String>,

    /// Descrição da Natureza da Operação
    /// Ex: Venda de mercadorias
    pub nat_op: String,

    /// Indicador da forma de pagamento
    /// 0=Pagamento à vista;
    /// 1=Pagamento a prazo;
    /// 2=Outros.
    /// Default: Some(0)
    pub ind_pag: Option<u8>,

    /// Código do Modelo do Documento Fiscal
    /// 55=NF-e emitida em substituição ao modelo 1 ou 1A;
    /// 65=NFC-e, utilizada nas operações de venda no varejo
    /// Default: 55
    pub mod_: u32,

    /// Série do Documento Fiscal.
    /// Ex: 1 ou 001 (3 dígitos)
    pub serie: u32,

    /// Número do Documento Fiscal.
    /// Ex: 1 ou 000000001 (9 dígitos)
    pub n_nf: u64,

    /// Data e hora de emissão do Documento Fiscal no formato UTC (Universal Coordinated Time):
    /// AAAA-MM-DDThh:mm:ssTZD
    /// Ex: 2021-08-01T12:00:00-03:00
    pub dh_emi: Option<String>,

    /// Data e hora de Saída ou da Entrada da Mercadoria/Produto no formato UTC (Universal Coordinated Time):
    ///  AAAA-MM-DDThh:mm:ssTZD. Não informar este campo para a NFC-e.
    /// Ex: 2021-08-01T12:00:00-03:00
    pub dh_sai_ent: Option<String>,

    /// Tipo de Operação
    /// 0=Entrada;
    /// 1=Saída
    /// Default: 1
    pub tp_nf: u8,

    /// Identificador de local de destino da operação
    /// 1=Operação interna;
    /// 2=Operação interestadual;
    /// 3=Operação com exterior.
    pub id_dest: u8,

    /// Código do Município de Ocorrência do Fato Gerador. Informar o município de ocorrência do fato gerador do ICMS.
    /// Ex: 3550308 = São Paulo
    pub c_mun_fg: String,

    /// Formato de Impressão do DANFE
    /// 0=Sem geração de DANFE;
    /// 1=DANFE normal, Retrato;
    /// 2=DANFE normal, Paisagem;
    /// 3=DANFE Simplificado;
    /// 4=DANFE NFC-e;
    /// 5=DANFE NFC-e em mensagem eletrônica
    pub tp_imp: u8,

    /// Tipo de Emissão da NF-e
    /// 1=Emissão normal (não em contingência);
    /// 2=Contingência FS-IA, com impressão do DANFE em formulário de segurança;
    /// 3=Contingência SCAN (Sistema de Contingência do Ambiente Nacional);
    /// 4=Contingência DPEC (Declaração Prévia da Emissão em Contingência);
    /// 5=Contingência FS-DA, com impressão do DANFE em formulário de segurança;
    /// 6=Contingência SVC-AN (SEFAZ Virtual de Contingência do AN);
    /// 7=Contingência SVC-RS (SEFAZ Virtual de Contingência do RS);
    /// 9=Contingência off-line da NFC-e
    pub tp_emis: u8,

    /// Dígito Verificador da Chave de Acesso da NF-e
    /// Ex: 5 (1 dígito)
    pub c_dv: Option<u8>,

    /// Identificação do Ambiente
    /// 1=Produção
    /// 2=Homologação
    pub tp_amb: u8,

    /// Finalidade de emissão da NF-e
    /// 1=NF-e normal;
    /// 2=NF-e complementar;
    /// 3=NF-e de ajuste;
    /// 4=Devolução de mercadoria.
    pub fin_nfe: u8,

    /// Indica operação com Consumidor final 0=Normal; 1=Consumidor final
    pub ind_final: u8,

    /// Indicador de presença do comprador no estabelecimento comercial no momento da operação
    /// 0=Não se aplica (por exemplo, Nota Fiscal complementar ou de ajuste);
    /// 1=Operação presencial;
    /// 2=Operação não presencial, pela Internet;
    /// 3=Operação não presencial, Teleatendimento;
    /// 4=NFC-e em operação com entrega a domicílio;
    /// 9=Operação não presencial, outros.
    pub ind_pres: u8,

    /// Processo de emissão da NF-e
    /// 0=Emissão de NF-e com aplicativo do contribuinte;
    /// 1=Emissão de NF-e avulsa pelo Fisco;
    /// 2=Emissão de NF-e avulsa, pelo contribuinte com seu certificado digital, através do site do Fisco;
    /// 3=Emissão NF-e pelo contribuinte com aplicativo fornecido pelo Fisco.
    pub proc_emi: u8,

    /// Versão do Processo de emissão da NF-e
    /// Informar a versão do aplicativo emissor de NF-e.
    /// Ex: 1.0.0
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
            c_dv: None,
            tp_amb: 2,
            fin_nfe: 1,
            ind_final: 1,
            ind_pres: 1,
            proc_emi: 0,
            ver_proc: "1.0.0".to_string(),
        }
    }
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
pub struct Dest {
    /// CNPJ do destinatário Ex: 12345678000123
    pub cnpj: Option<String>,
    /// CPF do destinatário Ex: 00011122233
    pub cpf: Option<String>,
    /// Identificação do destinatário no caso de comprador estrangeiro
    pub id_estrangeiro: Option<String>,
    /// Razão social do destinatário Ex: Empresa Ltda
    pub x_nome: Option<String>,
    /// Logradouro do destinatário Ex: Rua das Flores
    pub x_lgr: Option<String>,
    /// Número do endereço Ex: 1234 ou S/N
    pub nro: Option<String>,
    /// Bairro do destinatário Ex: Centro
    pub x_bairro: Option<String>,
    /// Código do município Ex: 4205407 para Lages
    pub c_mun: Option<String>,
    /// Nome do município Ex: Lages
    pub x_mun: Option<String>,
    /// Sigla da UF Ex: SC
    pub uf: Option<String>,
    /// CEP do destinatário Ex: 88509900
    pub cep: Option<String>,
    /// Código do país Ex: 1058 para Brasil
    pub c_pais: Option<String>,
    /// Nome do país Ex: Brasil
    pub x_pais: Option<String>,
    /// Telefone do destinatário Ex: 4999999999
    pub fone: Option<String>,
    /// Indicador da IE do destinatário
    /// 1 = Contribuinte ICMS (informar a IE do destinatário);
    /// 2 = Contribuinte isento de Inscrição no cadastro de Contribuintes
    /// 9 = Não Contribuinte, que pode ou não possuir Inscrição
    /// Estadual no Cadastro de Contribuintes do ICMS.
    /// Nota 1: No caso de NFC-e informar indIEDest=9 e não informar
    /// a tag IE do destinatário;
    /// Nota 2: No caso de operação com o Exterior informar
    /// indIEDest=9 e não informar a tag IE do destinatário;
    /// Nota 3: No caso de Contribuinte Isento de Inscrição
    /// (indIEDest=2), não informar a tag IE do destinatário
    pub ind_ie_dest: Option<u8>,
    /// Inscrição estadual do destinatário Ex: 123456789
    /// Default: None, não preencher quando ind_ie_dest=9
    /// String<2-14>
    pub ie: Option<String>,
    /// Indicador da SUFRAMA
    /// Obrigatório, nas operações que se beneficiam de incentivos
    /// fiscais existentes nas áreas sob controle da SUFRAMA.
    /// A omissão desta informação impede o processamento da
    /// operação pelo Sistema de Mercadoria Nacional da SUFRAMA e
    /// a liberação da Declaração de Ingresso, prejudicando a
    /// comprovação do ingresso / internamento da mercadoria nestas
    /// áreas. (v2.0)
    pub isuf: Option<String>,
    /// Inscrição Municipal do Tomador do Serviço
    /// Campo opcional, pode ser informado na NF-e conjugada, com
    /// itens de produtos sujeitos ao ICMS e itens de serviços sujeitos
    /// ao ISSQN.
    pub im: Option<String>,
    /// Email
    /// Campo pode ser utilizado para informar o e-mail de recepção da
    /// NF-e indicada pelo destinatário (v2.0)
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
    // ******* ICMS ******* //
    pub icms: String,
    pub orig: Option<u8>,
    pub cst: Option<String>,
    pub mod_bc: Option<u8>,
    pub v_bc: Option<f64>,
    pub p_icms: Option<f64>,
    pub v_icms: Option<f64>,
    pub csosn: Option<u16>,
    pub p_cred_sn: Option<f64>,
    pub v_cred_icmssn: Option<f64>,
    // ******* PIS ******* //
    pub pis: String,
    // ******* COFINS ******* //
    pub cofins: String,
    /// vTotTrib 13v2 Valor aproximado total de tributos federais, estaduais e municipais.
    /// Decimal com até 2 dígitos, sendo 11 inteiros e 2 decimais.
    /// u<1-13.0-2 Chars>
    pub v_tot_trib: f64,
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
            cofins: "ng".to_string(),
            v_tot_trib: 0.0,
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
    /// Valor do pagamento
    pub v_pag: f64,
}

impl Default for Pag {
    fn default() -> Self {
        Pag {
            ind_pag: 0,
            t_pag: "99".to_string(),
            v_pag: 0.0,
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
