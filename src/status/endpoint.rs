use crate::interno::ws::nfe_status_servico;

/// URL do `NfeStatusServico` da UF. `modelo` escolhe o webservice: 55 (NF-e) ou 65 (NFC-e) —
/// em várias UFs (SP, por exemplo) a NFC-e roda em outro host, e um pode cair sem o outro.
pub fn status_url(environment: u8, uf: &str, modelo: u32) -> Result<String, String> {
    nfe_status_servico(environment, uf, modelo, false)
        .map(|url| url.to_string())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modelo_65_usa_o_webservice_da_nfce() {
        assert_eq!(
            status_url(1, "SP", 65).unwrap(),
            "https://nfce.fazenda.sp.gov.br/ws/NFeStatusServico4.asmx"
        );
    }

    #[test]
    fn modelo_55_usa_o_webservice_da_nfe() {
        assert_eq!(
            status_url(1, "SP", 55).unwrap(),
            "https://nfe.fazenda.sp.gov.br/ws/nfestatusservico4.asmx"
        );
    }

    #[test]
    fn todas_as_ufs_tem_status_para_os_dois_modelos() {
        const UFS: [&str; 27] = [
            "AC", "AL", "AP", "AM", "BA", "CE", "DF", "ES", "GO", "MA", "MT", "MS", "MG", "PA",
            "PB", "PR", "PE", "PI", "RJ", "RN", "RS", "RO", "RR", "SC", "SP", "SE", "TO",
        ];
        for uf in UFS {
            for amb in [1, 2] {
                for modelo in [55, 65] {
                    assert!(
                        status_url(amb, uf, modelo).is_ok(),
                        "sem NfeStatusServico {modelo} p/ {uf} amb {amb}"
                    );
                }
            }
        }
    }
}
