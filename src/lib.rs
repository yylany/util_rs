pub mod client;

#[cfg(feature = "notify")]
pub mod notify;
pub mod tool;
mod spider;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::spider::stats::get_system_resources;

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

    }
}
