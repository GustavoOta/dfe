use chrono::*;
use chrono_tz::America::Sao_Paulo;

/// Get the current year with the number of digits specified 2 or 4. Default is 4.
pub fn get_current_year(digits: u8) -> String {
    format_sao_paulo_year(Utc::now(), digits)
}

fn format_sao_paulo_year(date_time_utc: DateTime<Utc>, digits: u8) -> String {
    let current_year = date_time_utc.with_timezone(&Sao_Paulo).year();
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
    format_sao_paulo_month(Utc::now())
}

fn format_sao_paulo_month(date_time_utc: DateTime<Utc>) -> String {
    let current_month = date_time_utc.with_timezone(&Sao_Paulo).month();
    let current_month = current_month.to_string();

    // if month is less than 10, add a 0 before the month
    if current_month.len() == 1 {
        let current_month = format!("0{}", current_month);
        current_month
    } else {
        current_month
    }
}

pub fn get_current_date_time() -> String {
    format_sao_paulo_date_time(Utc::now())
}

fn format_sao_paulo_date_time(date_time_utc: DateTime<Utc>) -> String {
    date_time_utc
        .with_timezone(&Sao_Paulo)
        .format("%Y-%m-%dT%H:%M:%S%:z")
        .to_string()
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

#[test]
fn test_get_current_date_time_month_boundary_sao_paulo() {
    // 2026-04-01T00:00:00Z must be 2026-03-31T21:00:00-03:00 in Sao Paulo.
    let date_time_utc = Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).single().unwrap();
    let date_time = format_sao_paulo_date_time(date_time_utc);

    assert_eq!(date_time, "2026-03-31T21:00:00-03:00");
}

#[test]
fn test_get_current_month_month_boundary_sao_paulo() {
    // 2026-04-01T00:00:00Z must still be month 03 in Sao Paulo.
    let date_time_utc = Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).single().unwrap();
    let month = format_sao_paulo_month(date_time_utc);

    assert_eq!(month, "03");
}

#[test]
fn test_get_current_year_year_boundary_sao_paulo() {
    // 2026-01-01T00:00:00Z must still be year 2025 in Sao Paulo.
    let date_time_utc = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single().unwrap();

    assert_eq!(format_sao_paulo_year(date_time_utc, 4), "2025");
    assert_eq!(format_sao_paulo_year(date_time_utc, 2), "25");
}
