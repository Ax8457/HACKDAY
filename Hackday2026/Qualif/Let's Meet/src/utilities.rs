use chrono::{Datelike, NaiveDate};

pub fn get_days(year: i32, month: u32) -> Vec<(u32, String)> {
    let mut days = Vec::new();
    let mut date = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid date");
    while date.month() == month {
        days.push((date.day(), date.weekday().to_string()));
        date = date.succ_opt().expect("Invalid next day");
    }
    days
}
