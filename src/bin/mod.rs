#[tokio::main]
async fn main() {}

#[cfg(test)]
mod tests {
    use dfe::nfe::service_status;
    use dfe::nfe::types::config::*;

    /// Test the service status with the default configuration in a file
    #[tokio::test]
    async fn test_service_status() {
        let teste = service_status(Use::FileConfig).await;
        if teste.is_err() {
            println!("Error test_service_status:{:?}", teste.err());
            assert!(false);
            return;
        }
        let teste = teste.unwrap();

        println!("tp_amb: {}", teste.tp_amb);
        println!("ver_aplic: {}", teste.ver_aplic);
        println!("c_stat: {}", teste.c_stat);
        println!("x_motivo: {}", teste.x_motivo);
        println!("c_uf: {}", teste.c_uf);
        println!("dh_recbto: {}", teste.dh_recbto);
        println!("t_med: {}", teste.t_med);
    }

    /// Test the service status with custom configuration and password in a file
    #[tokio::test]
    async fn test_service_status_custom() {
        let teste = service_status(Use::ManualConfig(Fields {
            cert_path: "D:/Projetos/go.pfx".to_string(),
            cert_pass: Password::File(PassFile {
                path: "cert_pass.txt".to_string(),
            }),
            federative_unit: "SP".to_string(),
            environment: Environment::Homologation,
        }))
        .await;
        if teste.is_err() {
            println!("Error test_service_status_custom:{:?}", teste.err());
            assert!(false);
            return;
        }
        let teste = teste.unwrap();

        println!("tp_amb: {}", teste.tp_amb);
        println!("ver_aplic: {}", teste.ver_aplic);
        println!("c_stat: {}", teste.c_stat);
        println!("x_motivo: {}", teste.x_motivo);
        println!("c_uf: {}", teste.c_uf);
        println!("dh_recbto: {}", teste.dh_recbto);
        println!("t_med: {}", teste.t_med);
    }

    /// Test the service status with custom configuration and password in a string
    #[tokio::test]
    async fn test_service_status_custom_pass() {
        let teste = service_status(Use::ManualConfig(Fields {
            cert_path: "D:/Projetos/go.pfx".to_string(),
            cert_pass: Password::Phrase("1234".to_string()),
            federative_unit: "SP".to_string(),
            environment: Environment::Homologation,
        }))
        .await;
        if teste.is_err() {
            println!("Error test_service_status_custom_pass:{:?}", teste.err());
            assert!(false);
            return;
        }
        let teste = teste.unwrap();

        println!("tp_amb: {}", teste.tp_amb);
        println!("ver_aplic: {}", teste.ver_aplic);
        println!("c_stat: {}", teste.c_stat);
        println!("x_motivo: {}", teste.x_motivo);
        println!("c_uf: {}", teste.c_uf);
        println!("dh_recbto: {}", teste.dh_recbto);
        println!("t_med: {}", teste.t_med);
    }
}
