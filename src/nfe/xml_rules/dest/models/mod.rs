use serde::{Deserialize, Serialize};

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
