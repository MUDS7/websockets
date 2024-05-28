use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use serde::{Deserialize, Serialize};

pub const SERVER_ADD: &'static str = "ws://127.0.0.1:3000/ws";

/// 客户端传送信息的body
#[derive(Serialize, Deserialize)]
pub struct ClientMessage {
    // 发送方的用户名
    pub from_user: String,
    // 接受方的用户名
    pub to_user: String,
    // 发送的消息
    pub message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (stream, response) = connect_async(format!("{}/{}", SERVER_ADD, "1")).await?;
    let (mut sender, mut receiver) = stream.split();
    sender.send(Message::Ping("Hello".into())).await?;
    // 用于接收消息
    tokio::spawn(async move {
        loop {
            // 接收消息
            while let Some(Ok(msg)) = receiver.next().await {
                match msg {
                    Message::Text(t) => {}
                    Message::Binary(b) => {
                        dbg!(b.len());
                    }
                    Message::Ping(ping) => {}
                    Message::Pong(pong) => {}
                    Message::Close(_) => {}
                    Message::Frame(_) => {}
                }
            }
        }
    });
    // 等待用户输入
    loop {
        let mut msg = String::new();
        std::io::stdin().read_line(&mut msg).unwrap();

        let bin = bincode::serialize(&ClientMessage {
            from_user: "1".to_string(),
            to_user: "2".to_string(),
            message: msg,
        }).unwrap();
        // 发送消息
        sender.send(Message::Binary(bin)).await?;
    }
    Ok(())
}