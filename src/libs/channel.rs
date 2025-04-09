use axum::extract::ws::WebSocket;
//use axum::extract::State;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::{Arc, Mutex, mpsc};

use super::message::{ChatMessage, MessageQueue};
use super::shared::SharedState;

// pub async fn ws_handler(
//     mq: Option<impl MessageQueue + Send>,
// ) -> Box<dyn FnOnce(WebSocketUpgrade, State<SharedState>) -> Response> {
//     Box::new(
//         move |ws: WebSocketUpgrade, State(state): State<SharedState>|  {
//             ws.on_upgrade(|socket| handle_socket(socket, state, mq))
//         }
//     )
// }

pub async fn handle_socket(
    socket: WebSocket,
    state: SharedState,
    mq: Arc<Mutex<Option<impl MessageQueue<Item = ChatMessage> + std::marker::Send + 'static>>>,
) {
    let (mut sender, mut receiver) = socket.split();

    let (tx, rx) = mpsc::channel::<ChatMessage>();
    let username = format!("user_{}", rand::random::<u32>() % 1000);


    if let Ok(mut s) = state.write() {
        s.sender.insert(username.clone(), Arc::new(tx.clone()));
    }

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

                if let Some(ref m) = *mq.lock().unwrap() {
                    let _ = m.send(&chat_msg);
                }

                println!("[ws] {:?}", &chat_msg);
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv() {
            let text = serde_json::to_string(&msg).unwrap();
            // to ws client
            if sender
                .send(axum::extract::ws::Message::Text(text.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort(),
    };

    println!("Connection closed for {}", &username);
    if let Ok(mut s) = state.write() {
        s.sender.remove(&username);
    }
}

#[allow(unused)]
trait Client {
    fn on_init() {}
    fn on_message() {}
}
