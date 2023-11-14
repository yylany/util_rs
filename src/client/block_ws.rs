use anyhow::{anyhow, Result};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use websocket::client::Url;
use websocket::sync::{Client, Writer};
use websocket::{ClientBuilder, OwnedMessage};

pub fn connect_to_ws_timeout(
    _ws_url: &str,
    _addr: &str,
    timeout: Duration,
) -> Result<Client<TcpStream>> {
    let (tx, rx) = std::sync::mpsc::channel();

    let ws_url = _ws_url.to_string();
    let addr = _addr.to_string();

    thread::spawn(move || {
        tx.send(connect_to_ws(&ws_url, &addr)).unwrap();
    });

    rx.recv_timeout(timeout)
        .map_err(|err| anyhow!("({}) websocket 连接超时：{err}", _addr))
        .and_then(|s| s)
}
pub fn connect_to_ws(ws_url: &str, addr: &str) -> Result<Client<TcpStream>> {
    let url = Url::parse(ws_url)?;

    let mut t = if addr.is_empty() {
        let hostname = url.with_default_port(|url| {
            const SECURE_PORT: u16 = 443;
            const INSECURE_PORT: u16 = 80;
            const SECURE_WS_SCHEME: &str = "wss";

            Ok(match Some(false) {
                None if url.scheme() == SECURE_WS_SCHEME => SECURE_PORT,
                None => INSECURE_PORT,
                Some(true) => SECURE_PORT,
                Some(false) => INSECURE_PORT,
            })
        })?;
        let mut t = TcpStream::connect(hostname)?;
        t
    } else {
        TcpStream::connect(addr)?
    };

    t.set_write_timeout(Some(Duration::from_secs(2)))?;
    t.set_read_timeout(Some(Duration::from_secs(2)))?;
    t.set_nodelay(true)?;

    let client = ClientBuilder::new(ws_url)
        .unwrap()
        .add_protocol("rust-websocket")
        .connect_on(t)?;

    Ok(client)
}
