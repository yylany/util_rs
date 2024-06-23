pub mod aes;

pub mod file;
pub mod libtime;
pub mod random;
mod req;
pub mod typ;

/// base 别名移除
pub fn base_trim(base: &str) -> &str {
    let ts = base.trim_start_matches("1000").trim_start_matches('0');

    if ts.ends_with("000") {
        ts.trim_end_matches('0').trim_end_matches('1')
    } else {
        ts
    }
}
