use std::time::SystemTime;

use chrono::prelude::*;
use tracing::error;

pub fn get_now_str() -> String {
    format_day_by_micros(get_now_micros())
}

pub fn get_now_millis() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub fn get_now_micros() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64
}

pub const FORMAT: &str = "%F %H:%M:%S%.6f";

pub fn format_day(millis: i64) -> String {
    Local.timestamp_millis(millis).format(FORMAT).to_string()
}

pub fn format_day_by_micros(nanos: i64) -> String {
    Local
        .timestamp_nanos(nanos * 1000)
        .format(FORMAT)
        .to_string()
}

pub fn format_day_by_rfc3339(s: &str) -> String {
    let da = DateTime::parse_from_rfc3339(s).unwrap();
    da.format(FORMAT).to_string()
}

pub fn get_micros_by_rfc3339(s: &str) -> i64 {
    let da = DateTime::parse_from_rfc3339(s).unwrap();
    da.timestamp_millis()
}

pub fn time_to_seconds(s: &str) -> i64 {
    match DateTime::parse_from_str(
        s,
        "%a %b %d %H:%M:%S %z %Y",
        // "1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z"
    ) {
        Ok(dt) => dt.timestamp(),
        Err(e) => {
            let msg = e.to_string();
            error!("解析时间异常：{}=> {}", s, msg);
            0
        }
    }
    // dt.with_timezone()
}

#[cfg(test)]
mod tests {
    use crate::tool::libtime::time_to_seconds;

    #[test]
    fn format_str_day() {
        println!("{}", time_to_seconds("Fri Jul 08 19:11:43 +0000 2021"));
    }
}
