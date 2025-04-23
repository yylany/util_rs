//! 封装常用请求代码

use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use reqwest::redirect::Policy;
use reqwest::{Client, ClientBuilder, Method, Response, StatusCode, Url};

static CLI: Lazy<Mutex<HashMap<Option<String>, Client>>> = Lazy::new(|| Default::default());

pub fn get_or_create_client(
    proxy: &Option<String>,
    url: &str,
    dns: Option<&str>,
    timeout: Duration,
) -> Client {
    CLI.lock()
        .entry(proxy.clone())
        .or_insert_with(|| {
            println!("创建 proxy: {:?} 的cli", proxy);
            build_cli(proxy, url, dns, timeout)
        })
        .clone()
}
// 清理所有缓存中的cli
pub fn clean_all_cli() {
    CLI.lock().clear();
}

fn build_cli(proxy: &Option<String>, url: &str, dns: Option<&str>, timeout: Duration) -> Client {
    let mut cli = gen_client_builder(proxy, timeout).redirect(Policy::default());
    if let Some(addr) = dns {
        let h = url.parse::<Url>().unwrap().host().unwrap().to_string();
        cli = cli.resolve(&h, addr.parse().unwrap())
    }

    cli.build().expect("Failed to build client")
}

pub async fn get(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
    timeout: Duration,
) -> Result<String> {
    let resp = exec_req(
        url,
        req_head,
        proxy,
        Method::GET,
        None,
        false,
        None,
        timeout,
    )
        .await;
    let mut _req_head = Vec::new();
    if let Some(req_heads) = req_head {
        // 添加默认的请求头参数
        for (k, v) in req_heads.iter() {
            _req_head.push(format!("{}: {}", k, v));
        }
    }
    handler_resp(resp, url, &_req_head).await
}

pub async fn exec_get(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
    timeout: Duration,
) -> Result<(StatusCode, String)> {
    let resp = exec_req(
        url,
        req_head,
        proxy,
        Method::GET,
        None,
        false,
        None,
        timeout,
    )
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    Ok((status, text))
}

pub async fn post(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
    body: String,
    timeout: Duration,
) -> Result<String> {
    let resp = exec_req(
        url,
        req_head,
        proxy,
        Method::POST,
        Some(body),
        false,
        None,
        timeout,
    )
        .await;
    let mut _req_head = Vec::new();

    if let Some(req_heads) = req_head {
        // 添加默认的请求头参数
        for (k, v) in req_heads.iter() {
            _req_head.push(format!("{}: {}", k, v));
        }
    }

    handler_resp(resp, url, &_req_head).await
}

pub async fn exec_post(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
    body: String,
    timeout: Duration,
) -> Result<(StatusCode, String)> {
    let resp = exec_req(
        url,
        req_head,
        proxy,
        Method::GET,
        Some(body),
        false,
        None,
        timeout,
    )
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    Ok((status, text))
}

pub async fn exec_req(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
    method: Method,
    body: Option<String>,
    force_newcli: bool, // 是否强制使用新的cli
    dns: Option<&str>,
    timeout: Duration,
) -> Result<Response> {
    let cli = if force_newcli {
        build_cli(proxy, url, dns, timeout)
    } else {
        get_or_create_client(proxy, url, dns, timeout)
    };

    let mut req_build = cli.request(method, url);
    if let Some(req_heads) = req_head {
        // 添加默认的请求头参数
        for (k, v) in req_heads.iter() {
            req_build = req_build.header(k.as_str(), v);
        }
    }
    if let Some(d) = body {
        req_build = req_build.body(d);
    }

    let resp = req_build.send().await?;
    Ok(resp)
}

pub async fn handler_resp(
    rs_resp: Result<Response>,
    ur: &str,
    head: &Vec<String>,
) -> Result<String> {
    let resp = match rs_resp {
        Ok(resp) => resp,
        Err(err) => {
            let msg = format!(
                r#"请求url：{}
    请求头 :
{}
    错误信息 {}
                "#,
                ur,
                head.join("\n"),
                err
            );
            return Err(anyhow!(msg));
        }
    };

    let body = match resp.status() {
        //只有在请求200的情况下，才会执行
        StatusCode::OK => resp.text().await?,

        u => {
            let msg = format!(
                r#"请求url：{}
    请求头 :
{}
    状态码：{}
    错误信息 {}
                "#,
                ur,
                head.join("\n"),
                u,
                resp.text().await?
            );
            return Err(anyhow!(msg));
        }
    };

    Ok(body)
}

/// 生成 ClientBuilder
pub fn gen_client_builder(proxy: &Option<String>, timeout: Duration) -> ClientBuilder {
    let cli = ClientBuilder::new();
    with_proxy(cli, proxy, timeout)
}

/// 加载并配置代理
fn with_proxy(mut cli: ClientBuilder, proxy: &Option<String>, timeout: Duration) -> ClientBuilder {
    cli = cli
        .timeout(timeout)
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
                            let u = url_info_decode(url.username());
                            let pass = url_info_decode(url.password().unwrap_or_default());

                            p = p.basic_auth(&u, &pass);
                        }

                        cli.proxy(p)
                    }

                    _ => cli,
                }
            }
        },
    }
}

// url 解码
pub fn url_info_decode(text: &str) -> String {
    urlencoding::decode(text)
        .map(|decoded| decoded.to_string())
        .unwrap_or_else(|_| text.to_string())
}
