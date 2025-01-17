use std::collections::HashSet;

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
