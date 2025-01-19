use anyhow::{anyhow, Result};
use once_cell::sync::{Lazy, OnceCell};
use std::ops::Deref;
use time::Duration;
use tokio::sync::broadcast::Sender;
use tracing::info;

pub mod entity;
pub mod push;
pub mod stats;
pub use entity::*;

// 使用泛型 T 的包装类型
pub struct Global<T>(OnceCell<T>);

// 为泛型实现 Deref trait
impl<T> Deref for Global<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.get_unchecked() }
    }
}

// 为泛型实现通用方法
impl<T> Global<T> {
    // 创建新实例
    pub const fn new() -> Self {
        Self(OnceCell::new())
    }

    // 初始化方法
    pub fn init(&self, value: T) -> Result<(), T> {
        self.0.set(value)
    }

    // 安全获取值的方法
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }

    // 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.0.get().is_some()
    }
}

/// 爬虫统计
pub static SPIDER_STATS: Lazy<stats::RequestStats> = Lazy::new(|| stats::RequestStats::new());

pub static SPIDER_STATS_PUSH: Global<Sender<String>> = Global::new();

// 初始化爬虫推送
pub fn init_spider_vars(push_target: Vec<String>) -> Result<()> {
    let s = push::load_broadcast_chan(push_target);
    SPIDER_STATS_PUSH.init(s).map_err(|err| anyhow!("{err}"))?;

    Ok(())
}

// 更新爬虫统计状态
pub fn update_stats(
    request_time: i64,
    response_time: i64,
    status_code: String,
    result: RequestResult, // 使用枚举表示请求结果
) {
    SPIDER_STATS.update_stats(request_time, response_time, status_code, result)
}

// 更新爬虫统计状态
pub fn send_stats(
    server_name: String,
    scraper_name: String,
    project_code: String,
    scraper_type: String,

    // 用于测试 hosts 的延迟
    host_info: Option<(Vec<String>, u16)>,
) {
    let stats = SPIDER_STATS.to_stats_and_reset(
        server_name,
        scraper_name,
        project_code,
        scraper_type,
        host_info,
    );

    let msg = serde_json::to_string(&stats).unwrap();
    info!("发送统计信息: {}", msg);

    if let Err(err) = SPIDER_STATS_PUSH.send(msg) {
        info!("发送统计信息失败：{}", err);
    }
}
