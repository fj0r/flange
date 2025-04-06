use axum::{
    response::IntoResponse,
    extract::ws::{WebSocket, WebSocketUpgrade}
};

use axum::extract::State;
use futures::{sink::SinkExt, stream::StreamExt};

use super::message::ChatMessage;
use super::store::Store;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Store>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Store) {
    let (mut sender, mut receiver) = socket.split();

    let tx = state.sender.lock().unwrap().clone();
    let mut rx = tx.subscribe();

    let username = format!("user_{}", rand::random::<u32>() % 1000);
    let username_log = username.clone();

    let msg = ChatMessage {
        user: "System".to_string(),
        message: format!("Welcome, {}!", username),
    };
    tx.send(msg).ok();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Ok(text) = msg.to_text() {
                let chat_msg = ChatMessage {
                    user: username.clone(),
                    message: text.to_string(),
                };

                if tx.send(chat_msg).is_err() {
                    break;
                }
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let text = format!("{}: {}", msg.user, msg.message);
            if sender.send(axum::extract::ws::Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    };

    println!("Connection closed for {}", username_log);
}
