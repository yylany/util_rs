use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 请求结果的枚举类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestResult {
    Successful,       // 处理成功+请求成功  这个是没有命中缓存的
    SuccessfulAndCache,       // 处理成功+请求成功 + 命中缓存
    ParseError,       // 解析错误
    TimeoutError,     // 超时错误
    ConnectionError,  // 连接错误
}


// 用于序列化和反序列化的导入
#[derive(Serialize, Deserialize, Clone, Debug)]
// 表示资源使用情况的结构体
pub struct Usage {
    // 已使用的资源量
    pub used: u64,
    // 资源总量
    pub total: u64,
}

// 系统资源信息结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemResources {
    // CPU 使用率
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: String,
    // 内存使用情况
    #[serde(rename = "memoryUsage")]
    pub memory_usage: Usage,
    // 磁盘使用情况
    #[serde(rename = "diskUsage")]
    pub disk_usage: Usage,
}

// 异常类型统计结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExceptionTypes {
    // 连接错误次数
    #[serde(rename = "connectionError")]
    pub connection_error: i64,
    // 超时错误次数
    #[serde(rename = "timeoutError")]
    pub timeout_error: i64,
    // 解析错误次数
    #[serde(rename = "parseError")]
    pub parse_error: i64,
}

// 时间周期结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimePeriod {
    // 开始时间（毫秒级时间戳）
    pub start: i64,
    // 结束时间（毫秒级时间戳）
    pub end: i64,
}

// 统计信息结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stats {
    // 服务器名称
    #[serde(rename = "serverName")]
    pub server_name: String,
    // 爬虫名称
    #[serde(rename = "scraperName")]
    pub scraper_name: String,
    // 项目代号
    #[serde(rename = "projectCode")]
    pub project_code: String,
    // 爬虫类型
    #[serde(rename = "scraperType")]
    pub scraper_type: String,
    // 请求频率（每秒请求次数）
    #[serde(rename = "requestFrequency")]
    pub request_frequency: i64,
    // 时间周期
    #[serde(rename = "timePeriod")]
    pub time_period: TimePeriod,
    // 错误率（百分比）
    #[serde(rename = "errorRate")]
    pub error_rate: f64,
    // 异常类型统计
    #[serde(rename = "exceptionTypes")]
    pub exception_types: ExceptionTypes,
    // 运行时长（秒）
    #[serde(rename = "runtimeDuration")]
    pub runtime_duration: i64,
    // 请求总次数
    #[serde(rename = "totalRequests")]
    pub total_requests: i64,
    // 缓存命中率（百分比）
    #[serde(rename = "cacheHitRate")]
    pub cache_hit_rate: f64,
    // 缓存命中次数
    #[serde(rename = "cacheHit")]
    pub cache_hit: i64,
    // HTTP 状态码统计（键为状态码，值为次数）
    #[serde(rename = "httpStatusCodes")]
    pub http_status_codes: HashMap<String, i64>,
    // 平均请求延迟（毫秒）
    #[serde(rename = "averageRequestLatency")]
    pub average_request_latency: f64,
    // 主机延迟（键为主机地址，值为延迟时间，单位：毫秒）
    #[serde(rename = "hostsPingDelay")]
    pub hosts_ping_delay: HashMap<String, f64>,
    // 系统资源使用情况
    #[serde(rename = "systemResources")]
    pub system_resources: SystemResources,
}
