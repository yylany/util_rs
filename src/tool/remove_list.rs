use std::collections::HashSet;
use std::time::Duration;

use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::tool::blacklist_detach;

/// 配置文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveList {
    // 发生错误时；调用 bark api
    pub call_count: i32,
    //替换 {} 文本； 并调用
    pub target_url: Vec<String>,

    /// 配置 不同规则加载的url 信息，ruleId 可以重复定义多次；结果会累积起来
    pub ops: Vec<RemoveConfig>,
}

/// 配置文件
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveConfig {
    // 匹配的规则id
    pub rule_id: String,
    // 远程 黑名单url
    pub black_url: Option<String>,
    // 远程 白名单url
    pub white_url: Option<String>,
}

pub async fn url_call(msg: &str, call_urls: &Vec<String>, call_count: i32) {
    for _ in 0..call_count {
        for url in call_urls {
            // 替换成实际请求的url
            let nurl = url.replace("{}", &msg);
            tokio::spawn(async move {
                super::req::get(&nurl, None, &None).await.unwrap();
            });
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

pub async fn load_match_list_and_merge(
    rule_id: &str,
    ops: &Vec<RemoveConfig>,
    target_blacklist: &mut HashSet<String>,
    target_whitelist: &mut HashSet<String>,
    retry: u8,
) -> Result<()> {
    let (blacklist, whitelist) = load_match_list(rule_id, ops, retry).await?;
    hash_set_merge(target_blacklist, blacklist);
    hash_set_merge(target_whitelist, whitelist);
    Ok(())
}

/// 加载匹配 名称的 黑白名单
/// 第一个；黑名单列表。，第二个 白名单列表
pub async fn load_match_list(
    rule_id: &str,
    ops: &Vec<RemoveConfig>,
    retry: u8,
) -> Result<(HashSet<String>, HashSet<String>)> {
    let d = ops
        .iter()
        .filter(|op| op.rule_id.as_str() == rule_id)
        .collect::<Vec<_>>();
    load_all_list(&d, retry).await
}

/// 将 source 数据合并到 target 中
pub fn hash_set_merge(target: &mut HashSet<String>, source: HashSet<String>) {
    for x in source {
        target.insert(x);
    }
}

/// 加载所有 黑白名单
/// 第一个；黑名单列表。，第二个 白名单列表
pub async fn load_all_list(
    ops: &[&RemoveConfig],
    retry: u8,
) -> Result<(HashSet<String>, HashSet<String>)> {
    let mut blacklist: HashSet<String> = HashSet::new();
    let mut whitelist: HashSet<String> = HashSet::new();

    for op_c in ops {
        if let Some(u) = &op_c.black_url {
            merge(u, &mut blacklist, retry).await?;
        }
        if let Some(u) = &op_c.white_url {
            merge(u, &mut whitelist, retry).await?;
        }
    }

    Ok((blacklist, whitelist))
}

//retry 重试次数
pub async fn merge(url: &str, data: &mut HashSet<String>, retry: u8) -> Result<()> {
    let mut er = Ok(());

    for _ in 0..retry {
        let text = match super::req::get(url, None, &None).await {
            Ok(s) => s,
            Err(err) => {
                er = Err(err);
                continue;
            }
        };

        if text.is_empty() {
            er = Err(anyhow!("响应数据为空"));
            continue;
        }

        hash_set_merge(data, blacklist_detach(&text));
        return Ok(());
    }

    er
}
