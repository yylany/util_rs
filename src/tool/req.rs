//! 封装常用请求代码

use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::{ClientBuilder, Method, Response, StatusCode, Url};

pub async fn get(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
) -> Result<String> {
    let resp = exec_req(url, req_head, proxy, Method::GET, None).await;
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
) -> Result<(StatusCode, String)> {
    let resp = exec_req(url, req_head, proxy, Method::GET, None).await?;
    let status = resp.status();
    let text = resp.text().await?;
    Ok((status, text))
}

pub async fn post(
    url: &str,
    req_head: Option<&HashMap<String, String>>,
    proxy: &Option<String>,
    body: String,
) -> Result<String> {
    let resp = exec_req(url, req_head, proxy, Method::POST, Some(body)).await;
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
) -> Result<(StatusCode, String)> {
    let resp = exec_req(url, req_head, proxy, Method::GET, Some(body)).await?;
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
) -> Result<Response> {
    let cli = gen_client_builder(proxy).build()?;

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
pub fn gen_client_builder(proxy: &Option<String>) -> ClientBuilder {
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
