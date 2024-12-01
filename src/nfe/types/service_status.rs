use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub tp_amb: String,
    pub ver_aplic: String,
    pub c_stat: String,
    pub x_motivo: String,
    pub c_uf: String,
    pub dh_recbto: String,
    pub t_med: String,
}
