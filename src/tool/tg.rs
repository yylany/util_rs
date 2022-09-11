use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use teloxide::prelude::AutoSend;
use teloxide::{
    net,
    requests::{Requester, RequesterExt},
    Bot,
};
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "debug")]
    pub debug: bool,
    //tg 机器人的 token
    #[serde(rename = "token")]
    pub token: String,
    //推送到特定接收者中
    #[serde(rename = "subscribers")]
    pub subscribers: Vec<String>,
}

//用于tg消息的推送
pub struct TgBot {
    //tg机器人推送
    tg_bot: Arc<AutoSend<Bot>>,

    //推送列表
    push_list: Arc<Vec<String>>,

    //false: send msg
    //true: println msg
    debug: bool,
}

impl TgBot {
    pub fn new(config: &Config) -> TgBot {
        let token = config.token.clone();
        let push_chan = config.subscribers.clone();
        let debug = config.debug;

        match env::var("HTTP_PROXY") {
            Ok(proxy) => {
                let client = net::default_reqwest_settings()
                    .proxy(Proxy::all(&proxy).unwrap())
                    .build()
                    .expect("Client creation failed");
                TgBot {
                    tg_bot: Arc::new(Bot::with_client(token, client).auto_send()),
                    push_list: Arc::new(push_chan.clone()),
                    debug,
                }
            }
            Err(_) => TgBot {
                tg_bot: Arc::new(Bot::new(token).auto_send()),
                push_list: Arc::new(push_chan.clone()),
                debug,
            },
        }
    }

    //推送消息
    pub async fn notify(&self, msg: String) {
        if self.debug {
            info!("tg send msg: {}", msg);
        } else {
            debug!("tg send msg: {}", &msg);
            let bot = self.tg_bot.clone();
            let list = self.push_list.clone();

            for x in list.iter() {
                match bot.send_message(x.to_string(), msg.clone()).await {
                    Ok(_) => {}
                    Err(_) => {
                        // error!("{:#?}", e)
                    }
                };
            }
        }
    }
}
