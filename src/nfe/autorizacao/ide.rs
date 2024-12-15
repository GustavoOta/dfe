use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename = "ide")]
pub struct IdeProcess {
    /// Código da UF do emitente do Documento Fiscal
    /// Ex: 35 = São Paulo
    #[serde(rename = "cUF")]
    pub c_uf: u16,

    /// Código Numérico que compõe a Chave de Acesso de 8 dígitos gerado pelo emitente para cada NF-e para evitar acessos indevidos da NF-e.
    #[serde(rename = "cNF")]
    pub c_nf: Option<String>,

    /// Descrição da Natureza da Operação
    /// Ex: Venda de mercadorias
    #[serde(rename = "natOp")]
    pub nat_op: String,

    /// Indicador da forma de pagamento
    /// 0=Pagamento à vista;
    /// 1=Pagamento a prazo;
    /// 2=Outros.
    #[serde(rename = "indPag", skip_serializing_if = "Option::is_none")]
    pub ind_pag: Option<u8>,

    /// Código do Modelo do Documento Fiscal
    /// 55=NF-e emitida em substituição ao modelo 1 ou 1A;
    /// 65=NFC-e, utilizada nas operações de venda no varejo
    #[serde(rename = "mod")]
    pub mod_: u32,

    /// Série do Documento Fiscal.
    /// Ex: 1 ou 001 (3 dígitos)
    pub serie: u32,

    /// Número do Documento Fiscal.
    /// Ex: 1 ou 000000001 (9 dígitos)
    #[serde(rename = "nNF")]
    pub n_nf: u64,

    /// Data e hora de emissão do Documento Fiscal no formato UTC (Universal Coordinated Time):
    /// AAAA-MM-DDThh:mm:ssTZD
    /// Ex: 2021-08-01T12:00:00-03:00
    #[serde(rename = "dhEmi")]
    pub dh_emi: Option<String>,

    /// Data e hora de Saída ou da Entrada da Mercadoria/Produto no formato UTC (Universal Coordinated Time):
    ///  AAAA-MM-DDThh:mm:ssTZD. Não informar este campo para a NFC-e.
    /// Ex: 2021-08-01T12:00:00-03:00
    #[serde(rename = "dhSaiEnt", skip_serializing_if = "Option::is_none")]
    pub dh_sai_ent: Option<String>,

    /// Tipo de Operação
    /// 0=Entrada;
    /// 1=Saída
    #[serde(rename = "tpNF")]
    pub tp_nf: u8,

    /// Identificador de local de destino da operação
    /// 1=Operação interna;
    /// 2=Operação interestadual;
    /// 3=Operação com exterior.
    #[serde(rename = "idDest")]
    pub id_dest: u8,

    /// Código do Município de Ocorrência do Fato Gerador. Informar o município de ocorrência do fato gerador do ICMS.
    /// Ex: 3550308 = São Paulo
    #[serde(rename = "cMunFG")]
    pub c_mun_fg: u64,

    /// Formato de Impressão do DANFE
    /// 0=Sem geração de DANFE;
    /// 1=DANFE normal, Retrato;
    /// 2=DANFE normal, Paisagem;
    /// 3=DANFE Simplificado;
    /// 4=DANFE NFC-e;
    /// 5=DANFE NFC-e em mensagem eletrônica
    #[serde(rename = "tpImp")]
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
    #[serde(rename = "tpEmis")]
    pub tp_emis: u8,

    /// Dígito Verificador da Chave de Acesso da NF-e
    /// Ex: 5 (1 dígito)
    #[serde(rename = "cDV")]
    pub c_dv: Option<u8>,

    /// Identificação do Ambiente
    /// 1=Produção
    /// 2=Homologação
    #[serde(rename = "tpAmb")]
    pub tp_amb: u8,

    /// Finalidade de emissão da NF-e
    /// 1=NF-e normal;
    /// 2=NF-e complementar;
    /// 3=NF-e de ajuste;
    /// 4=Devolução de mercadoria.
    #[serde(rename = "finNFe")]
    pub fin_nfe: u8,

    /// Indica operação com Consumidor final 0=Normal; 1=Consumidor final
    #[serde(rename = "indFinal")]
    pub ind_final: u8,

    /// Indicador de presença do comprador no estabelecimento comercial no momento da operação
    /// 0=Não se aplica (por exemplo, Nota Fiscal complementar ou de ajuste);
    /// 1=Operação presencial;
    /// 2=Operação não presencial, pela Internet;
    /// 3=Operação não presencial, Teleatendimento;
    /// 4=NFC-e em operação com entrega a domicílio;
    /// 9=Operação não presencial, outros.
    #[serde(rename = "indPres")]
    pub ind_pres: u8,

    /// Processo de emissão da NF-e
    /// 0=Emissão de NF-e com aplicativo do contribuinte;
    /// 1=Emissão de NF-e avulsa pelo Fisco;
    /// 2=Emissão de NF-e avulsa, pelo contribuinte com seu certificado digital, através do site do Fisco;
    /// 3=Emissão NF-e pelo contribuinte com aplicativo fornecido pelo Fisco.
    #[serde(rename = "procEmi")]
    pub proc_emi: u8,

    /// Versão do Processo de emissão da NF-e
    /// Informar a versão do aplicativo emissor de NF-e.
    /// Ex: 1.0.0
    #[serde(rename = "verProc")]
    pub ver_proc: String,
}
