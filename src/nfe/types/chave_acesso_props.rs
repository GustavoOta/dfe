use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChaveAcessoProps {
    /// Federal Unit code from IBGE
    pub uf: u16,
    /// CNPJ or CPF of the company or PErson that will issue the NFe
    pub doc: String,
    /// Model of the NFe 55 or 65
    pub modelo: u32,
    /// Serie of the NFe from 0 to 999
    pub serie: u32,
    /// Numerical code from 0 to 999999999
    pub numero: u64,
    /// Type of emission 1 for normal, 2 for contingency
    pub tp_emis: u8,
    /// Numerical code from 0 to 99999999 This code is used to generate the access key
    /// and must be unique for each NFe issued
    /// If not informed, a random number will be generated
    pub codigo_numerico: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtractComposition {
    pub uf: String,
    pub ano: String,
    pub mes: String,
    pub doc: String,
    pub modelo: String,
    pub serie: String,
    pub numero: String,
    pub tp_emis: String,
    pub codigo_numerico: String,
}
