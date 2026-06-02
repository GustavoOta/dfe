use quick_xml::events::Event;
use quick_xml::name::QName;
use quick_xml::Reader;

use crate::tipos::service_status::Status;

pub fn parse_status_response(body: &str) -> Result<Status, String> {
    let mut reader = Reader::from_str(body);
    let mut tp_amb_content = String::new();
    let mut ver_aplic_content = String::new();
    let mut c_stat_content = String::new();
    let mut xmotivo_content = String::new();
    let mut c_uf_content = String::new();
    let mut dh_recbto_content = String::new();
    let mut t_med_content = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.name() {
                QName(b"tpAmb") => {
                    tp_amb_content = reader
                        .read_text(e.name())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                }
                QName(b"verAplic") => {
                    ver_aplic_content = reader
                        .read_text(e.name())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                }
                QName(b"cStat") => {
                    c_stat_content = reader
                        .read_text(e.name())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                }
                QName(b"xMotivo") => {
                    xmotivo_content = reader
                        .read_text(e.name())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                }
                QName(b"cUF") => {
                    c_uf_content = reader
                        .read_text(e.name())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                }
                QName(b"dhRecbto") => {
                    dh_recbto_content = reader
                        .read_text(e.name())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                }
                QName(b"tMed") => {
                    t_med_content = reader
                        .read_text(e.name())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                }
                _ => {}
            },
            Ok(Event::Eof) => {
                return Ok(Status {
                    tp_amb: tp_amb_content,
                    ver_aplic: ver_aplic_content,
                    c_stat: c_stat_content,
                    x_motivo: xmotivo_content,
                    c_uf: c_uf_content,
                    dh_recbto: dh_recbto_content,
                    t_med: t_med_content,
                });
            }
            Err(e) => {
                return Err(format!("falha ao processar XML de resposta: {}", e));
            }
            _ => {}
        }
    }
}
