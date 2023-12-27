use crossbeam_channel::{bounded, unbounded, SendTimeoutError, Sender};
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};
use telegram_bot::{
    types::ToChatRef, Api, ChatId, ChatRef, InputFileUpload, Message, SendDocument, SendMessage,
    Text,
};

use tracing::{debug, error, info};

/// 加载tg 消息通道
pub fn load_tg(config: &Config) -> Sender<SendType> {
    let notify = TgBot::new(config);

    let (msg_sen, msg_rec) = unbounded::<SendType>();

    thread::spawn(move || {
        let (send, rece) = bounded(1);

        thread::spawn(move || {
            let rn = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            for msg in rece {
                match msg {
                    SendType::Msg(s) => rn.block_on(notify.notify(s)),
                    SendType::File(p) => rn.block_on(notify.notify_file(p)),
                };
            }
        });

        // 先停止 1s 。等待上面线程启动
        thread::sleep(Duration::from_secs(1));

        let mut size = 0;

        for msg in msg_rec {
            match &msg {
                SendType::Msg(msg_body) => {
                    if msg_body.contains("timed out") {
                        size += 1;
                        if size >= 100 {
                            size = 0;
                            if let Err(err) = send.send_timeout(msg, Duration::from_micros(10)) {
                                match err {
                                    SendTimeoutError::Timeout(msg) => {
                                        info!("tg通知异常（通道超时）  msg => {:?}", msg);
                                    }
                                    SendTimeoutError::Disconnected(msg) => {
                                        info!("tg通知异常（通道关闭）  msg => {:?}", msg);
                                    }
                                }
                            }
                        }
                        continue;
                    }

                    if let Err(err) = send.send_timeout(msg, Duration::from_micros(10)) {
                        match err {
                            SendTimeoutError::Timeout(msg) => {
                                info!("tg通知异常（通道超时）  msg => {:?}", msg);
                            }
                            SendTimeoutError::Disconnected(msg) => {
                                info!("tg通知异常（通道关闭）  msg => {:?}", msg);
                            }
                        }
                    }
                }
                SendType::File(_) => {
                    send.send(msg);
                }
            }
        }
    });

    msg_sen
}

pub enum SendType {
    Msg(String),
    File(String),
}

impl Debug for SendType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SendType::Msg(m) => {
                write!(f, "消息： {}", m)
            }
            SendType::File(m) => {
                write!(f, "文件： {}", m)
            }
        }
    }
}

pub struct ThisChatId(i64);

impl ToChatRef for ThisChatId {
    fn to_chat_ref(&self) -> ChatRef {
        ChatRef::Id(ChatId::from(self.0))
    }
}

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
    tg_bot: Api,

    //推送列表
    push_list: Arc<Vec<ThisChatId>>,

    //false: send msg
    //true: println msg
    debug: bool,
}

pub fn get_boot(token: String) -> Api {
    Api::new(token)
}

impl TgBot {
    pub fn new(config: &Config) -> TgBot {
        TgBot {
            tg_bot: get_boot(config.token.clone()),
            push_list: Arc::new(
                config
                    .subscribers
                    .clone()
                    .into_iter()
                    .map(|s| ThisChatId(s.parse().unwrap()))
                    .collect(),
            ),
            debug: config.debug,
        }
    }

    pub fn new_with_bot(bot: Api, subscribers: Vec<String>, debug: bool) -> TgBot {
        TgBot {
            tg_bot: bot,
            push_list: Arc::new(
                subscribers
                    .into_iter()
                    .map(|s| ThisChatId(s.parse().unwrap()))
                    .collect(),
            ),
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
                let msg_req = SendMessage::new(x, msg.clone());
                bot.send(msg_req).await;
            }
        }
    }

    //推送消息
    //button（第一个是：按钮名称，第二个是：回调数据）
    pub async fn notify_with_button(&self, msg: String, button: Vec<(String, String)>) {
        // if self.debug {
        //     info!("tg send msg: {}", msg);
        // } else {
        //     debug!("tg send msg: {}", &msg);
        //     let bot = self.tg_bot.clone();
        //     let list = self.push_list.clone();
        //
        //     let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
        //
        //     for versions in button.chunks(3) {
        //         let row = versions
        //             .into_iter()
        //             .map(|(t, d)| InlineKeyboardButton::callback(t, d))
        //             .collect();
        //         keyboard.push(row);
        //     }
        //     let b = InlineKeyboardMarkup::new(keyboard);
        //
        //     for x in list.iter() {
        //         match bot
        //             .send_message(x.to_string(), msg.clone())
        //             .reply_markup(b.clone())
        //             .await
        //         {
        //             Ok(_) => {}
        //             Err(_) => {
        //                 // error!("{:#?}", e)
        //             }
        //         };
        //     }
        // }
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
                let file = InputFileUpload::with_path(Text::from(file.clone()));
                let msg_req = SendDocument::new(x, file);
                bot.send(msg_req).await;
            }
        }
    }
}
