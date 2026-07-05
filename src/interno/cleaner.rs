//use rand::Rng;
use regex::Regex;
//use std::collections::HashMap;

pub struct Strings;

impl Strings {
    pub fn clear_xml_string(string: &str) -> String {
        let xml = string.replace("<?xml version=\"1.0\" encoding=\"UTF-8\"?>", "");
        let xml = xml.replace("\n", "");
        let xml = xml.replace("\r", "");
        let xml = xml.replace("\t", "");
        let xml = xml.replace(" /", "/");
        let xml = xml.replace("\\", "");
        // Regex constante e válido; fallback defensivo para nunca panicar no fluxo de
        // assinatura (A2) caso a compilação falhasse por algum motivo inesperado.
        let xml = match Regex::new(r">\s+<") {
            Ok(re) => re.replace_all(&xml, "><").to_string(),
            Err(_) => xml,
        };

        let xml = xml.trim().to_string();
        xml.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Caracteriza a "limpeza" que simula a canonicalização C14N (usada na assinatura XML-DSig).
    /// Qualquer alteração aqui pode invalidar assinaturas na SEFAZ — o golden trava o comportamento.
    #[test]
    fn golden_clear_xml_string() {
        let entrada = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<a>\t <b x=\"1\" />  <c>texto</c>\r\n</a>";
        let saida = Strings::clear_xml_string(entrada);
        insta::assert_snapshot!(saida);
    }
}
