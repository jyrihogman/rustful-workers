use chrono::{Datelike, Utc};

pub fn get_current_date_in_utc() -> String {
    let current_date = Utc::today();
    current_date.format("%Y-%m-%d").to_string()
}
