/* pub fn sign_xml(xml: &str, cert: &str) -> String {
    let xml = xml.replace("\n", "").replace("\r", "");
    // replace tabs
    let xml = xml.replace("\t", "");

    let digest = sha1_digest(&xml);
    let signature = sign(&digest, cert);

    let signature = r#"<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
            <SignedInfo>
                <CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
                <SignatureMethod Algorithm="http://www.w3.org/2000/09/xmldsig#rsa-sha1"/>
                <Reference URI="\#NFe{}">
                    <Transforms>
                        <Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/>
                        <Transform Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
                    </Transforms>
                    <DigestMethod Algorithm="http://www.w3.org/2000/09/xmldsig#sha1"/>
                    <DigestValue>{}</DigestValue>
                </Reference>
            </SignedInfo>
            <SignatureValue>{}</SignatureValue>
            <KeyInfo>
                <X509Data>
                    <X509Certificate>{}</X509Certificate>
                </X509Data>
            </KeyInfo>
        </Signature>"#;

    let xml = xml.replace("</NFe>", &format!("{}</NFe>", signature));

    xml
}

fn sha1_digest(data: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    let result = base64::encode(&result);
    result
}

fn sign(data: &str, cert: &str) -> String {
    let cert = cert::Cert::from_pem(cert).unwrap();
    let key = cert.get_private_key().unwrap();
    let key = Rsa::private_key_from_der(key).unwrap();

    let mut signer = Signer::new(MessageDigest::sha1(), &key).unwrap();
    signer.update(data.as_bytes()).unwrap();
    let result = signer.sign_to_vec().unwrap();
    let result = base64::encode(&result);
    result
}
 */
