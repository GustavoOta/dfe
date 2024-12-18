//use rand::Rng;
use regex::Regex;
//use std::collections::HashMap;

pub struct Strings;

impl Strings {
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
}
