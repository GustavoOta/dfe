use chrono::*;
use chrono_tz::America::Sao_Paulo;

/// Get the current year with the number of digits specified 2 or 4. Default is 4.
pub fn get_current_year(digits: u8) -> String {
    let current_year = chrono::Utc::now().year();
    let current_year = current_year.to_string();

    match digits {
        2 => {
            let current_year = current_year.chars().skip(2).take(2).collect::<String>();
            current_year
        }
        4 => current_year,
        _ => current_year,
    }
}

/// Get the current month in the format MM.
pub fn get_current_month() -> String {
    let current_month = chrono::Utc::now().month();
    let current_month = current_month.to_string();

    // if month is less than 10, add a 0 before the month
    if current_month.len() == 1 {
        let current_month = format!("0{}", current_month);
        current_month
    } else {
        current_month
    }
}

/// Date and time in format UTC (Universal Coordinated Time): AAAA-MM-DDThh:mm:ssTZD
pub fn get_current_date_time() -> String {
    let current_date_time: DateTime<Utc> = Utc::now();
    let sao_paulo_time = current_date_time.with_timezone(&Sao_Paulo);
    let formatted_date_time = format!(
        "{}T{:02}:{:02}:{:02}-03:00",
        sao_paulo_time.date_naive(),
        sao_paulo_time.hour(),
        sao_paulo_time.minute(),
        sao_paulo_time.second()
    );
    formatted_date_time
}

#[test]
fn test_get_current_year() {
    let year = get_current_year(2);
    println!("Year: {}", year);
}

#[test]
fn test_get_current_month() {
    let month = get_current_month();
    println!("Month: {}", month);
}

#[test]
fn test_get_current_date_time() {
    let date_time = get_current_date_time();
    println!("Date Time: {}", date_time);
}
