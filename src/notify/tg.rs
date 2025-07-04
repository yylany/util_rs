use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};

use crossbeam_channel::{bounded, unbounded, SendTimeoutError, Sender};
use reqwest::Proxy;
use serde::{Deserialize, Serialize};
use teloxide::payloads::SendAnimationSetters;
use teloxide::prelude::AutoSend;
use teloxide::types::{InputMedia, InputMediaDocument};
use teloxide::{
    net,
    payloads::SendMessageSetters,
    prelude::*,
    requests::{Requester, RequesterExt},
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputFile,
        InputMessageContentText,
    },
    utils::command::BotCommands,
    Bot,
};
use tracing::{debug, error, info, trace};

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
                    SendType::FileWarpMsg((p, msg)) => {
                        rn.block_on(notify.notify_file_with_msg(p, msg))
                    }
                    SendType::FilesWarpMsg(p, msg) => {
                        rn.block_on(notify.notify_files_with_msg(p, msg))
                    }
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
                _ => {
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
    FileWarpMsg((String, Option<String>)),

    FilesWarpMsg(Vec<String>, Option<String>),
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

            SendType::FileWarpMsg((m, msg)) => {
                write!(f, "文件： {}; 附带消息：{:?}", m, msg)
            }
            SendType::FilesWarpMsg(m, msg) => {
                write!(f, "文件： {:?}  附带信息: {:?}", m, msg)
            }
        }
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
    pub fn new_with_bot(bot: Arc<AutoSend<Bot>>, subscribers: Vec<String>, debug: bool) -> TgBot {
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

            let parts = split_message(&msg, 4000);
            let total_pages = parts.len();

            if total_pages > 1 {
                for chat_id in list.iter() {
                    let max_retries = 3;
                    for (index, part) in parts.iter().enumerate() {
                        let page_info = format!("\n\n--- 第 {}/{} 页 ---", index + 1, total_pages);
                        let message_with_page = format!("{}{}", part, page_info);

                        let mut retries = 0;
                        while retries < max_retries {
                            match bot
                                .send_message(chat_id.to_string(), &message_with_page)
                                .await
                            {
                                Ok(_) => break,
                                Err(err) => {
                                    retries += 1;
                                    if retries == max_retries {
                                        return;
                                    }
                                    error!("tg 发送消息失败：{}", err);
                                }
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            } else {
                for x in list.iter() {
                    match bot.send_message(x.to_string(), msg.clone()).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("tg 发送消息失败：{}", err);

                            // error!("{:#?}", e)
                        }
                    };
                }
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
                match bot
                    .send_message(x.to_string(), msg.clone())
                    .reply_markup(b.clone())
                    .await
                {
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

    //推送文件
    pub async fn notify_files_with_msg(&self, files: Vec<String>, msg: Option<String>) {
        if self.debug {
            debug!("tg send file: {:?}", files);
        } else {
            debug!("tg send file: {:?}", &files);
            let bot = self.tg_bot.clone();
            let list = self.push_list.clone();

            let mut send_fs_item = Vec::new();

            let file_size = files.len();

            for (index, file) in files.into_iter().enumerate() {
                let fs = InputFile::file(file.parse::<PathBuf>().unwrap());

                let item = if file_size == (index + 1) {
                    InputMedia::Document(InputMediaDocument::new(fs).caption(msg.as_ref().unwrap()))
                } else {
                    InputMedia::Document(InputMediaDocument::new(fs))
                };

                send_fs_item.push(item);
            }

            for x in list.iter() {
                let s = bot.send_media_group(x.to_string(), send_fs_item.clone());

                let result = s.await;

                match result {
                    Ok(d) => {}
                    Err(e) => {
                        error!("{:#?}", e)
                    }
                };
            }
        }
    }

    //推送文件
    pub async fn notify_file_with_msg(&self, file: String, msg: Option<String>) {
        if self.debug {
            debug!("tg send file: {}", file);
        } else {
            trace!("tg send file: {}", &file);
            let bot = self.tg_bot.clone();
            let list = self.push_list.clone();

            for x in list.iter() {
                loop {
                    let s = bot.send_document(
                        x.to_string(),
                        InputFile::file(file.parse::<PathBuf>().unwrap()),
                    );

                    let result = match &msg {
                        None => s.await,
                        Some(text) => {
                            info!("{}", text);
                            s.caption(text).await
                        }
                    };

                    match result {
                        Ok(d) => {
                            break;
                        }
                        Err(e) => {
                            error!("{:#?}", e)
                        }
                    };
                }
            }
        }
    }
}

fn split_message(text: &str, max_length: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();

    for char in text.chars() {
        if current.chars().count() >= max_length {
            result.push(current);
            current = String::new();
        }
        current.push(char);
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}
