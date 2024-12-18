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
        let re = Regex::new(r">\s+<").unwrap();
        let xml = re.replace_all(&xml, "><").to_string();

        let xml = xml.trim().to_string();
        xml.to_string()
    }
}
