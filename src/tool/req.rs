//! 封装常用请求代码

use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use reqwest::{ClientBuilder, StatusCode, Url};

pub async fn exec_get(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
) -> Result<(StatusCode, String)> {
    let cli = gen_client_builder(&None).build()?;
    let mut req_build = cli.get(url);

    if let Some(req_heads) = req_head {
        // 添加默认的请求头参数
        for (k, v) in req_heads.iter() {
            req_build = req_build.header(k.as_str(), v);
        }
    }
    let resp = req_build.send().await?;
    let status = resp.status();
    // let heads = resp.headers();
    let text = resp.text().await?;
    Ok((status, text))
}

/// 生成 ClientBuilder
fn gen_client_builder(proxy: &Option<String>) -> ClientBuilder {
    let cli = ClientBuilder::new();
    with_proxy(cli, proxy)
}

/// 加载并配置代理
fn with_proxy(mut cli: ClientBuilder, proxy: &Option<String>) -> ClientBuilder {
    cli = cli
        .timeout(Duration::from_millis(3000))
        .danger_accept_invalid_certs(true)
        .pool_idle_timeout(None)
        .tcp_keepalive(None)
        .gzip(true);
    match proxy {
        None => match env::var("HTTP_PROXY") {
            Ok(p) => cli.proxy(reqwest::Proxy::https(p).unwrap()),
            Err(_) => cli,
        },
        Some(proxy) => match proxy.starts_with("local") {
            true => {
                let local_addr = IpAddr::from_str(proxy.trim_start_matches("local://")).unwrap();
                cli.local_address(local_addr)
            }
            false => {
                let url = Url::parse(&proxy).unwrap();
                let scheme = url.scheme();

                match scheme {
                    "socks5" | "http" | "https" | "socks5h" => {
                        let proxy_url = format!(
                            "{}://{}:{}",
                            scheme,
                            url.host().unwrap(),
                            url.port().unwrap()
                        );

                        let mut p = reqwest::Proxy::all(proxy_url).unwrap();

                        if url.has_authority() && url.username() != "" {
                            p = p.basic_auth(url.username(), url.password().unwrap_or_default());
                        }

                        cli.proxy(p)
                    }

                    _ => cli,
                }
            }
        },
    }
}
