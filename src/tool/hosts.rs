use anyhow::{anyhow, Result};
use std::time::Duration;

///加载本地hosts文件
async fn load_file_host(path: &str) -> Result<String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|err| anyhow!(err))
}

//加载hosts 文件
pub async fn load_host_file(path: &str) -> Result<String> {
    match path.starts_with("http") {
        true => load_url_host(path).await,
        false => load_file_host(path).await,
    }
}

pub fn split_txt(txt: impl AsRef<str>) -> Vec<String> {
    let d = txt.as_ref();

    match serde_json::from_str::<Vec<String>>(d) {
        Ok(s) => s,
        Err(_) => {
            let ips = d
                .split_ascii_whitespace()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            ips
        }
    }
}

///加载远程地址的hosts文件
async fn load_url_host(url: &str) -> Result<String> {
    super::req::get_with_timeout(url, None, &None, Duration::from_millis(30000)).await
}
