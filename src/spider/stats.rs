use super::entity::*;
use crate::tool::libtime;
use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};

pub struct RequestStats {
    inner: Mutex<InnerStats>,
}

impl RequestStats {
    /// 创建一个新的统计实例，并记录初始化时间和开始时间
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InnerStats::new()),
        }
    }

    /// 更新统计信息的方法
    pub fn update_stats(
        &self,
        request_time: i64,
        response_time: i64,
        status_code: u16,
        result: RequestResult, // 使用枚举表示请求结果
    ) {
        self.inner
            .lock()
            .update_stats(request_time, response_time, status_code, result);
    }

    /// 将当前统计数据拼装到 `Stats` 结构体中，并清空当前统计数据
    /// 统计的时候需要传入 hosts + port 信息
    pub fn to_stats_and_reset<'a>(
        &self,
        base: &'a StatsBase,

        // 用于测试 hosts 的延迟
        host_info: Option<(Vec<String>, u16)>,
    ) -> Stats<'a> {
        let mut host_ping = HashMap::new();

        if let Some((hosts, port)) = host_info {
            let timeout = Duration::from_secs(3);

            for host in hosts {
                let connet_ts = match run_test_tcp(&host, port, timeout) {
                    Ok(d) => d,
                    Err(_) => timeout.as_micros() as u64,
                };

                // 微秒转成毫秒
                let ms = connet_ts as f64 / 1000.0;
                host_ping.insert(host, ms);
            }
        }

        let mut data = self.inner.lock();
        let mut d = data.to_stats_and_reset(base);
        d.hosts_ping_delay = host_ping;

        // 替换统计对象信息
        *data = InnerStats::new();

        d
    }
}

struct InnerStats {
    // 对象初始化时间（毫秒级时间戳）
    pub init_time: i64,
    // 当前统计周期的开始时间（毫秒级时间戳）
    pub start_time: i64,
    // 总请求数
    pub total_requests: i64,
    // 成功请求数
    pub successful_requests: i64,
    // 命中缓存的次数；在请求成功的情况下才统计
    pub cache_hit: i64,
    // 解析失败次数
    pub parse_errors: i64,
    // 超时错误次数
    pub timeout_errors: i64,
    // 连接失败次数
    pub connection_errors: i64,
    // HTTP 状态码统计（键为状态码，值为出现次数）
    pub http_status_codes: HashMap<u16, i64>,
    // 总请求延迟（毫秒）
    pub total_latency: i64,
    // 平均请求延迟（毫秒）
    pub average_latency: f64,
}

impl InnerStats {
    /// 创建一个新的统计实例，并记录初始化时间和开始时间
    fn new() -> Self {
        let current_time = libtime::get_now_millis();
        Self {
            init_time: current_time,
            start_time: current_time,
            total_requests: 0,
            successful_requests: 0,
            parse_errors: 0,
            timeout_errors: 0,
            connection_errors: 0,
            cache_hit: 0,
            http_status_codes: HashMap::new(),
            total_latency: 0,
            average_latency: 0.0,
        }
    }

    /// 更新统计信息的方法
    pub fn update_stats(
        &mut self,
        request_time: i64,
        response_time: i64,
        status_code: u16,
        result: RequestResult, // 使用枚举表示请求结果
    ) {
        // 增加总请求数
        self.total_requests += 1;

        // 计算请求延迟
        let latency = response_time - request_time;
        self.total_latency += latency;

        // 更新平均延迟
        self.average_latency = self.total_latency as f64 / self.total_requests as f64;

        // 更新 HTTP 状态码统计
        // 很多爬虫都是使用0 代替；这里直接忽略0 的情况
        if status_code != 0 {
            *self
                .http_status_codes
                .entry(status_code.clone())
                .or_insert(0) += 1;
        }

        // 根据请求结果更新对应的统计数据
        match result {
            RequestResult::Successful => {
                self.successful_requests += 1;
            }
            RequestResult::SuccessfulAndCache => {
                self.successful_requests += 1;
                self.cache_hit += 1;
            }

            RequestResult::ParseError => {
                self.parse_errors += 1;
            }
            RequestResult::TimeoutError => {
                self.timeout_errors += 1;
            }
            RequestResult::ConnectionError => {
                self.connection_errors += 1;
            }
        }
    }

    /// 将当前统计数据拼装到 `Stats` 结构体中，并清空当前统计数据
    pub fn to_stats_and_reset<'a>(&mut self, base: &'a StatsBase) -> Stats<'a> {
        // 获取当前时间作为结束时间
        let end_time = libtime::get_now_millis();

        // 构造时间周期
        let time_period = TimePeriod {
            start: self.start_time,
            end: end_time,
        };

        // 构造异常类型统计
        let exception_types = ExceptionTypes {
            connection_error: self.connection_errors,
            timeout_error: self.timeout_errors,
            parse_error: self.parse_errors,
        };

        // 计算错误率
        let error_rate = if self.total_requests > 0 {
            (self.parse_errors + self.timeout_errors + self.connection_errors) as f64
                / self.total_requests as f64
        } else {
            0.0
        };

        // 计算运行时长（从对象初始化到当前时间）
        let runtime_duration = (end_time - self.init_time) / 1000;

        let cache_hit_rate = if self.successful_requests == 0 {
            0.0
        } else {
            let cache_hit_rate = self.cache_hit as f64 / self.successful_requests as f64;
            (cache_hit_rate * 1000.0).round() / 1000.0
        };

        // 构造 `Stats` 结构体
        let stats = Stats {
            base,
            time_period,
            error_rate,
            exception_types,
            runtime_duration,
            total_requests: self.total_requests,
            cache_hit_rate,            // 假设没有缓存相关数据，可以根据需要补充
            cache_hit: self.cache_hit, // 假设没有缓存相关数据，可以根据需要补充
            http_status_codes: self
                .http_status_codes
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
            average_request_latency: (self.average_latency * 1000.0).round() / 1000.0,
            hosts_ping_delay: HashMap::new(), // 假设没有主机延迟数据，可以根据需要补充
            system_resources: get_system_resources(),
        };

        stats
    }
}

/// 获取系统资源数据
pub fn get_system_resources() -> SystemResources {
    // 创建一个 System 实例
    let mut system = System::new_all();

    // 刷新系统信息
    system.refresh_all();

    // 获取 CPU 使用率
    let cpu_usage = format!("{:.2}%", system.global_cpu_info().cpu_usage());

    // 获取内存使用情况（单位从 KB 转换为 MB）
    let total_memory = system.total_memory() / (1024 * 1024); // 总内存（MB）
    let used_memory = system.used_memory() / (1024 * 1024); // 已使用内存（MB）

    let memory_usage = Usage {
        used: used_memory,
        total: total_memory,
    };

    // 获取所有磁盘的使用情况（单位从字节转换为 MB）
    let mut total_disk_space = 0;
    let mut total_disk_used = 0;

    for disk in system.disks() {
        total_disk_space += disk.total_space() / (1024 * 1024); // 累加磁盘总空间（MB）
        total_disk_used += (disk.total_space() - disk.available_space()) / (1024 * 1024);
        // 累加磁盘已使用空间（MB）
    }

    let disk_usage = Usage {
        used: total_disk_used,
        total: total_disk_space,
    };

    // 构造 SystemResources
    SystemResources {
        cpu_usage,
        memory_usage,
        disk_usage,
    }
}

/// 测试tcp 连接耗时; 返回连接的耗时
pub fn run_test_tcp(addr: &str, port: u16, ping_timeout: Duration) -> Result<u64> {
    let resolve_ip = IpAddr::from_str(addr)?;

    let start_time = Instant::now();
    let _ = TcpStream::connect_timeout(&SocketAddr::new(resolve_ip, port), ping_timeout).map_err(
        |err| {
            anyhow!(
                "当前连接时长：{} ms;错误信息：{err}",
                start_time.elapsed().as_millis()
            )
        },
    )?;
    let elapsed_time = start_time.elapsed();
    Ok(elapsed_time.as_micros() as u64)
}
