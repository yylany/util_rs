use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PriceCeiling {
    pub coin: String,
    // 最大可用溢价
    pub max_price: f64,
    // 结束时间；单位ms
    pub end_ts: i64,
    // 触发时间 ;ms
    pub target: i64,
}
