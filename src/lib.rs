use tokio::runtime::Runtime;

pub mod client;

#[cfg(feature = "notify")]
pub mod notify;
pub mod spider;
pub mod tool;

pub fn get_new_rn(num: usize, th_name: &str) -> Runtime {
    let rn = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num)
        .thread_name(th_name)
        .enable_all()
        .build()
        .unwrap();
    rn
}

#[cfg(test)]
mod tests {
    use crate::spider;
    use crate::spider::stats::get_system_resources;
    use crate::spider::{RequestStatsConfig, StatsBase};
    use anyhow::Result;

    #[test]
    fn it_works() {
        // 1000XXXUSDT，10000XXXUSDT，1000000XXXUSDT 1MXXXUSDT

        assert_eq!(super::tool::base_trim("1000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("10000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1000000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1MXXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M10000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M0000XXXUSDT"), "XXXUSDT");
        assert_eq!(super::tool::base_trim("1M0000XXXUSDT10000"), "XXXUSDT");

        // 获取系统资源数据
        let system_resources = get_system_resources();

        // 打印系统资源数据
        println!("系统资源数据: {:?}", system_resources);

        // 打印详细信息
        println!("CPU 使用率: {}", system_resources.cpu_usage);
        println!(
            "内存使用: 已使用 {} MB / 总计 {} MB",
            system_resources.memory_usage.used, system_resources.memory_usage.total
        );
        println!(
            "磁盘使用: 已使用 {} MB / 总计 {} MB",
            system_resources.disk_usage.used, system_resources.disk_usage.total
        );

        let base = StatsBase {
            server_name: "".to_string(),
            scraper_name: "".to_string(),
            project_code: "".to_string(),
            scraper_type: "".to_string(),
            request_frequency: 0,
        };

        spider::init_spider_vars(
            RequestStatsConfig {
                target: vec![],
                reporting_cycle: Default::default(),
                host_test_port: 0,
            },
            base,
            // Box::new(|| Ok(vec!["ssss".to_string()])),
            Box::new(get_hosts),
        )
            .unwrap()
    }

    fn get_hosts() -> Result<Vec<String>> {
        Ok(vec!["ssss".to_string()])
    }
}
