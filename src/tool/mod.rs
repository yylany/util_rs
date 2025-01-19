use serde::{Deserialize, Deserializer};
use std::collections::HashSet;
use std::str::FromStr;
use std::time::Duration;

pub mod aes;
pub mod file;
pub mod hosts;
pub mod libtime;
pub mod random;
pub mod remove_list;
pub mod req;
pub mod typ;

#[cfg(feature = "openssl")]
pub mod openssl_aes;
pub mod num;

/// base 别名移除
pub fn base_trim(base: &str) -> &str {
    let ts = base.trim_start_matches("1M").trim_start_matches("1000").trim_start_matches('0');

    if ts.ends_with("000") {
        ts.trim_end_matches('0').trim_end_matches('1')
    } else {
        ts
    }
}

pub fn blacklist_detach(li: &str) -> HashSet<String> {
    li.split(",")
        .map(|s| s.trim().trim_matches('\"').trim().to_string())
        .collect()
}


pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let value: String = Deserialize::deserialize(deserializer)?;
    Ok(humantime::Duration::from_str(&value)
        .map_err(|err| D::Error::custom(err.to_string()))?
        .into())
}
