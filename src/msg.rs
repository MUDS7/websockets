use serde::{Deserialize, Serialize};

/// 客户端传送信息的body
#[derive(Serialize,Deserialize)]
pub struct ClientMessage {
    // 发送方的用户名
    pub from_user: String,
    // 接受方的用户名
    pub to_user: String,
    // 发送的消息
    pub message: String,
}
