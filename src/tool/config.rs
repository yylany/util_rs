use anyhow::{anyhow, Result};

use serde::de::{DeserializeOwned, Visitor};
use serde::Deserializer;

use std::str::FromStr;

use toml::Value;

// =============================================================================
// TOML loader 层：在反序列化前统一做 key 命名规范化
//
// 背景：
//   - 本项目自身的结构体按 Rust 惯例使用 snake_case 字段（去掉了所有
//     `rename_all = "camelCase"` / `#[serde(alias = "snake_case")]`）
//   - 第三方 crate（例如 stats crate）的结构体仍然用 `rename_all = "camelCase"`
//     并且期望 TOML 里写 camelCase
//   - 历史上存在 camelCase / snake_case / kebab-case 三种配置文件，都必须兼容
//
// 策略：在 toml::Value 层对每个 Table key 做"双向扩展"：
//   - 原 key 保留（兼容第三方 crate 期望的 camelCase）
//   - 同时补充一份 snake_case 版（兼容本项目的 Rust 惯例字段）
//   - kebab-case 统一归一到 snake_case
// 这样同一份 toml::Value 里同时存在 `reportingCycle` 和 `reporting_cycle`，
// 无论下级结构体期望哪种命名都能匹配到
// =============================================================================

/// 解析 TOML 字符串，自动兼容 camelCase / snake_case / kebab-case 三种 key 命名
///
/// 内部流程：
/// 1. 先 parse 成 [`toml::Value`]（保留原始 key 形式）
/// 2. 递归把每个 Table key 做双向扩展（原 key + snake_case 版本）
/// 3. 最后 `try_into::<T>()` 交给 serde 做类型映射
pub fn load_toml<T: DeserializeOwned>(content: &str) -> Result<T> {
    let value: Value = toml::from_str(content).map_err(|e| anyhow!("TOML 解析失败: {}", e))?;
    let normalized = normalize_keys(value);
    normalized
        .try_into::<T>()
        .map_err(|e| anyhow!("TOML 结构映射失败: {}", e))
}

/// 递归对 TOML Value 做 key 归一化
///
/// 对每个 Table key：
/// - 保留原始 key（保持对第三方 crate 的 camelCase 期望兼容）
/// - 若原 key 不是 snake_case，额外补一份 snake_case 版本
///
/// Array 递归处理每个元素；Value 本体不变（只动 key，不动 value）
fn normalize_keys(v: Value) -> Value {
    match v {
        Value::Table(t) => {
            // 预估容量 * 2 ：最坏情况每个 key 都要补一份 snake_case
            let mut out = toml::map::Map::with_capacity(t.len() * 2);
            for (k, v) in t {
                let normalized_value = normalize_keys(v);
                let snake = to_snake_case(&k);
                if snake != k {
                    // 原 key 保留 + 额外插入 snake_case 版本
                    // 若 snake_case key 已经存在（极少见，比如同时写了 camelCase 和
                    // snake_case），以先出现的为准，不覆盖
                    out.entry(snake).or_insert_with(|| normalized_value.clone());
                    out.insert(k, normalized_value);
                } else {
                    out.insert(k, normalized_value);
                }
            }
            Value::Table(out)
        }
        Value::Array(a) => Value::Array(a.into_iter().map(normalize_keys).collect()),
        other => other,
    }
}

/// 把字符串转 snake_case
///
/// 规则：
/// - `peakMinuteMultiple` → `peak_minute_multiple`
/// - `peak-minute-multiple` → `peak_minute_multiple` （kebab-case 一起支持）
/// - `peak_minute_multiple` → `peak_minute_multiple` （已是 snake_case，保持不变）
/// - 首字符为大写时直接转小写，不加前导下划线
///   （`PeakRate` → `peak_rate`，不是 `_peak_rate`）
/// - 前一个字符是 `_` / `-` 时，当前大写字符不再追加下划线
fn to_snake_case(s: &str) -> String {
    // 快速路径：已经是 snake_case（无大写、无连字符）直接返回
    if s.is_empty() || !s.bytes().any(|b| b.is_ascii_uppercase() || b == b'-') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 4);
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'-' {
            // kebab 分隔符统一归一成 snake 分隔符
            out.push('_');
        } else if b.is_ascii_uppercase() {
            // 非首字符且前一个字符不是分隔符时补一个下划线
            if i > 0 && bytes[i - 1] != b'_' && bytes[i - 1] != b'-' {
                out.push('_');
            }
            // ASCII 大写 → 小写（+32）
            out.push((b + 32) as char);
        } else {
            out.push(b as char);
        }
    }
    out
}
