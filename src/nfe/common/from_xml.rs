use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn get_chnfe(xml_file_path: String) -> Result<String, Box<dyn Error>> {
    let file = File::open(xml_file_path)?;
    let reader = BufReader::new(file);

    let mut chnfe = String::new();
    for line in reader.lines() {
        let line = line?;
        if line.contains("<chNFe>") {
            let start = line.find("<chNFe>").expect("Error: <chNFe> not found") + 7;
            let end = line.find("</chNFe>").expect("Error: </chNFe> not found");
            chnfe = line[start..end].to_string();
            break;
        }
    }

    if chnfe.is_empty() {
        return Err("Error: <chNFe> not found in the file".into());
    }

    Ok(chnfe)
}
