use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use teloxide::prelude::AutoSend;
use std::path::PathBuf;
use teloxide::{
    net,
    requests::{Requester, RequesterExt},
    Bot,
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InputFile,
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me,
    },
    utils::command::BotCommands,
};
use teloxide::payloads::SendAnimationSetters;
use tracing::{debug, error, info};

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

pub fn get_boot(token: String) -> AutoSend<Bot> {
    match env::var("HTTP_PROXY") {
        Ok(proxy) => {
            let client = net::default_reqwest_settings()
                .proxy(Proxy::all(&proxy).unwrap())
                .build()
                .expect("Client creation failed");
            Bot::with_client(token, client).auto_send()
        }
        Err(_) => Bot::new(token).auto_send(),
    }
}


impl TgBot {
    pub fn new(config: &Config) -> TgBot {
        TgBot {
            tg_bot: Arc::new(get_boot(config.token.clone())),
            push_list: Arc::new(config.subscribers.clone()),
            debug: config.debug,
        }
    }
    pub fn new_with_bot(bot:Arc<AutoSend<Bot>>,subscribers:Vec<String>,debug:bool) -> TgBot {
        TgBot {
            tg_bot: bot,
            push_list: Arc::new(subscribers),
            debug,
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


    //推送消息
    //button（第一个是：按钮名称，第二个是：回调数据）
    pub async fn notify_with_button(&self, msg: String, button: Vec<(String, String)>) {
        if self.debug {
            info!("tg send msg: {}", msg);
        } else {
            debug!("tg send msg: {}", &msg);
            let bot = self.tg_bot.clone();
            let list = self.push_list.clone();

            let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

            for versions in button.chunks(3) {
                let row = versions
                    .into_iter()
                    .map(|(t, d)| InlineKeyboardButton::callback(t, d))
                    .collect();
                keyboard.push(row);
            }
            let b = InlineKeyboardMarkup::new(keyboard);


            for x in list.iter() {
                match bot.send_message(x.to_string(), msg.clone()).reply_markup(b.clone()).await {
                    Ok(_) => {}
                    Err(_) => {
                        // error!("{:#?}", e)
                    }
                };
            }
        }
    }

    //推送文件
    pub async fn notify_file(&self, file: String) {
        if self.debug {
            debug!("tg send file: {}", file);
        } else {
            debug!("tg send file: {}", &file);
            let bot = self.tg_bot.clone();
            let list = self.push_list.clone();

            for x in list.iter() {
                match bot
                    .send_document(
                        x.to_string(),
                        InputFile::file(file.parse::<PathBuf>().unwrap()),
                    )
                    .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{:#?}", e)
                    }
                };
            }
        }
    }
}
