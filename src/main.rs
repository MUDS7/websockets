use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use axum::{Error, Router};
use axum::routing::get;
use dashmap::{DashMap, DashSet};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use crate::msg::ClientMessage;

pub mod msg;

pub const SERVER_ADD: &'static str = "127.0.0.1:3000";

pub struct AppState {
    pub user: DashMap<String, broadcast::Sender<Vec<u8>>>,
}

pub type UserState = DashMap<String, SplitSink<WebSocket, Message>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app_state = AppState {
        user: Default::default(),
    };
    let user_state: DashMap<String, SplitSink<WebSocket, Message>> = DashMap::new();
    // 创建路由
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/ws/:user", get(handle_websocket))
        // .with_state(Arc::new(app_state))
        .with_state(Arc::new(user_state));
    // 占用端口创建服务
    let lister = tokio::net::TcpListener::bind(SERVER_ADD).await?;
    axum::serve(lister, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

// 测试路由
async fn hello() -> String {
    "hello world".to_string()
}

/// 分发ws连接
async fn handle_websocket(ws: WebSocketUpgrade,
                          ConnectInfo(info): ConnectInfo<SocketAddr>,
                          user: axum::extract::Path<String>,
                          state: State<Arc<UserState>>) -> impl IntoResponse {
    println!("receive ip: {}:{}, user:{} connection", info.ip(), info.port(), &user.0);
    ws.on_upgrade(move |socket| { handle_socket_test(socket, user.0, state.0) })
}

async fn handle_socket_test(mut ws: WebSocket, user: String, state: Arc<UserState>) {
    let (tx, mut rx) = ws.split();
    // 将加入进来的用户添加到app state中
    if !state.contains_key(&user) {
        state.entry(user).or_insert(tx);
    }
    // let (tx,rx) = ws.split();
    // 处理接收消息
    while let Some(msg) = rx.next().await {
        match msg {
            Ok(msg) => {
                match msg {
                    Message::Text(txt) => {
                        dbg!(&txt);
                    }
                    Message::Binary(bin) => {
                        let msg = bincode::deserialize::<ClientMessage>(&bin).unwrap();
                        println!("Server receive msg : {}", msg.message);
                        // 发送给对应的用户
                        if let Some(mut sender) = state.get_mut(&msg.to_user) {
                            sender.send(Message::Binary(bin)).await.unwrap();
                            dbg!("send ok");
                        }
                    }
                    Message::Ping(_) => {
                        println!("receive a ping message");
                    }
                    Message::Pong(_) => {
                        println!("receive a pong message");
                    }
                    Message::Close(_) => {
                        println!("receive a lose message");
                    }
                }
            }
            Err(e) => {
                dbg!(&e.to_string());
            }
        }
    }
}

async fn handle_socket(mut ws: WebSocket, user: String, state: Arc<AppState>) {
    // 将加入进来的用户添加到app state中
    if !state.user.contains_key(&user) {
        let (tx, _rx) = broadcast::channel(100);
        state.user.entry(user).or_insert(tx);
    }
    // let (tx,rx) = ws.split();
    // 处理接收消息
    while let Some(msg) = ws.recv().await {
        match msg {
            Ok(msg) => {
                match msg {
                    Message::Text(txt) => {
                        dbg!(&txt);
                    }
                    Message::Binary(bin) => {
                        let msg = bincode::deserialize::<ClientMessage>(&bin).unwrap();
                        println!("Server receive msg : {}", msg.message);
                        // 发送给对应的用户
                        if let Some(sender) = state.user.get(&msg.to_user) {
                            let mut rx = sender.subscribe();
                            dbg!(&sender.receiver_count());
                            sender.send(bin).unwrap();
                            dbg!("send ok");
                        }
                    }
                    Message::Ping(_) => {
                        println!("receive a ping message");
                    }
                    Message::Pong(_) => {
                        println!("receive a pong message");
                    }
                    Message::Close(_) => {
                        println!("receive a lose message");
                    }
                }
            }
            Err(e) => {
                dbg!(&e.to_string());
            }
        }
    }

    // 创建新的线程用于发送信息
    tokio::spawn(async move {
        // 发送消息
        match ws.send(Message::Pong("Received".into())).await {
            Ok(_) => {}
            Err(e) => {
                dbg!(e.to_string());
            }
        }
    });
}

