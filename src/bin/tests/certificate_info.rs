#[tokio::test]
async fn certificate_info_test() {
    use dfe::nfe::common::cert::CertificateInfo;

    let pfx_path = "cert.pfx";
    let password = "****";
    match CertificateInfo::from_pfx(pfx_path, password) {
        Ok(info) => {
            println!("Subject: {}", info.subject);
            println!("Issuer: {}", info.issuer);
            println!("Valid From: {}", info.valid_from);
            println!("Valid To: {}", info.valid_to);
        }
        Err(e) => {
            eprintln!("Error retrieving certificate info: {}", e);
        }
    }
}
