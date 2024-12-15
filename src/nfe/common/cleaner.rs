use rand::Rng;
use regex::Regex;
use std::collections::HashMap;

pub struct Strings;

impl Strings {
    pub fn equilize_parameters(
        std: &mut HashMap<String, Option<String>>,
        possible: &[&str],
        replace_accented_chars: bool,
    ) {
        for &key in possible {
            if !std.contains_key(key) {
                std.insert(key.to_string(), None);
            } else if let Some(value) = std.get_mut(key) {
                if let Some(ref mut val) = value {
                    *val = Strings::replace_unacceptable_characters(val);
                    if replace_accented_chars {
                        *val = Strings::to_ascii(val);
                    }
                }
            }
        }
    }

    pub fn replace_specials_chars(string: &str) -> String {
        let mut string = Strings::squash_characters(string);
        string = string.replace('&', "e");
        let re = Regex::new(r"[^a-zA-Z0-9 @#,-_.;:$%/]").unwrap();
        string = re.replace_all(&string, "").to_string();
        let re = Regex::new(r"[<>]").unwrap();
        re.replace_all(&string, "").to_string()
    }

    pub fn replace_unacceptable_characters(input: &str) -> String {
        if input.is_empty() {
            return input.to_string();
        }
        let mut input = input
            .replace("& ", "&amp; ")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#39;");
        input = Strings::normalize(&input);
        input.trim().to_string()
    }

    pub fn to_ascii(input: &str) -> String {
        let input = Strings::normalize(input);
        Strings::squash_characters(&input)
    }

    pub fn squash_characters(input: &str) -> String {
        let a_find = [
            "á", "à", "ã", "â", "é", "ê", "í", "ó", "ô", "õ", "ú", "ü", "ç", "Á", "À", "Ã", "Â",
            "É", "Ê", "Í", "Ó", "Ô", "Õ", "Ú", "Ü", "Ç",
        ];
        let a_subs = [
            "a", "a", "a", "a", "e", "e", "i", "o", "o", "o", "u", "u", "c", "A", "A", "A", "A",
            "E", "E", "I", "O", "O", "O", "U", "U", "C",
        ];
        let mut result = input.to_string();
        for (find, subs) in a_find.iter().zip(a_subs.iter()) {
            result = result.replace(find, subs);
        }
        result
    }

    pub fn normalize(input: &str) -> String {
        let input = input.replace(&['\r', '\t', '\n'], "");
        let re = Regex::new(r"(?:\s\s+)").unwrap();
        let input = re.replace_all(&input, " ").to_string();
        let re = Regex::new(r"[\x00-\x08\x10\x0B\x0C\x0E-\x19\x7F]|[\x00-\x7F][\x80-\xBF]+|([\xC0\xC1]|[\xF0-\xFF])[\x80-\xBF]*|[\xC2-\xDF]((?![\x80-\xBF])|[\x80-\xBF]{2,})|[\xE0-\xEF](([\x80-\xBF](?![\x80-\xBF]))|(?![\x80-\xBF]{2})|[\x80-\xBF]{3,})/S").unwrap();
        let input = re.replace_all(&input, "").to_string();
        let re = Regex::new(r"\xE0[\x80-\x9F][\x80-\xBF]|\xED[\xA0-\xBF][\x80-\xBF]/S").unwrap();
        let input = re.replace_all(&input, "").to_string();
        let re = Regex::new(r"[[:cntrl:]]").unwrap();
        re.replace_all(&input, "").to_string()
    }

    pub fn only_numbers(string: &str) -> String {
        let re = Regex::new(r"[^0-9]").unwrap();
        re.replace_all(string, "").to_string()
    }

    pub fn clear_xml_string(string: &str) -> String {
        let xml = string.replace("<?xml version=\"1.0\" encoding=\"UTF-8\"?>", "");
        // clean /n and /r white space
        let xml = xml.replace("\n", "");
        let xml = xml.replace("\r", "");
        // clean /t tab space
        let xml = xml.replace("\t", "");
        // clear \ backslash
        let xml = xml.replace("\\", "");
        let re = Regex::new(r">\s+<").unwrap();
        let xml = re.replace_all(&xml, "><").to_string();
        // remove espaços no inicio e final
        let xml = xml.trim().to_string();
        xml.to_string()
    }

    pub fn delete_all_between(string: &str, beginning: &str, end: &str) -> String {
        if let (Some(beginning_pos), Some(end_pos)) = (string.find(beginning), string.find(end)) {
            let text_to_delete = &string[beginning_pos..end_pos + end.len()];
            string.replace(text_to_delete, "")
        } else {
            string.to_string()
        }
    }

    pub fn clear_protocoled_xml(string: &str) -> String {
        let mut proc_xml = Strings::clear_xml_string(string);
        let a_app = ["nfe", "cte", "mdfe"];
        for &app in &a_app {
            proc_xml = proc_xml.replace(
                &format!(r#"xmlns="http://www.portalfiscal.inf.br/{}" xmlns="http://www.w3.org/2000/09/xmldsig#""#, app),
                &format!(r#"xmlns="http://www.portalfiscal.inf.br/{}""#, app),
            );
        }
        proc_xml
    }

    pub fn remove_some_alien_chars_from_txt(txt: &str) -> String {
        let txt = txt.replace(&['\r', '\t'], "");
        let re = Regex::new(r"(?:\s\s+)").unwrap();
        let txt = re.replace_all(&txt, " ").to_string();
        txt.replace(&['|'], "|")
    }

    pub fn random_string(length: usize) -> String {
        let keyspace = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut rng = rand::thread_rng();
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..keyspace.len());
                keyspace.chars().nth(idx).unwrap()
            })
            .collect()
    }
}
