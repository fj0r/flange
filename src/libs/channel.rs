use axum::{
    response::IntoResponse,
    extract::ws::{WebSocket, WebSocketUpgrade}
};

use axum::extract::State;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::{mpsc, Arc, Mutex};

use super::message::ChatMessage;
use super::shared::SharedState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    let (tx, rx) = mpsc::channel::<ChatMessage>();
    let username = format!("user_{}", rand::random::<u32>() % 1000);

    let key = username.clone();
    state.write().unwrap()
        .sender.insert(key, Arc::new(Mutex::new(tx.clone())));

    let msg = ChatMessage {
        user: "System".to_owned(),
        content: format!("Welcome, {}!", &username).into(),
    };
    tx.send(msg).ok();

    let un = username.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // text protocol of ws
            if let Ok(text) = msg.to_text() {
                let chat_msg = ChatMessage {
                    user: un.clone(),
                    content: serde_json::to_value(text).unwrap(),
                };

                if tx.send(chat_msg).is_err() {
                    break;
                }
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv() {
            let text = serde_json::to_string(&msg).unwrap();
            if sender.send(axum::extract::ws::Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    };

    println!("Connection closed for {}", &username);
    state.write().unwrap()
        .sender.remove(&username);
}

trait Client {
    fn on_init() {

    }
    fn on_message() {

    }
}
