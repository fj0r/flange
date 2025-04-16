use axum::extract::ws::WebSocket;
//use axum::extract::State;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::{Arc, mpsc};

use super::message::ChatMessage;
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
    mqtx: Option<mpsc::Sender<ChatMessage>>,
) {
    let (mut sender, mut receiver) = socket.split();

    let (tx, rx) = mpsc::channel::<ChatMessage>();
    let username = format!("user_{}", rand::random::<u32>() % 1000);


    {
        let s1 = state.clone();
        if let Ok(mut s) = s1.write() {
            s.sender.insert(username.clone(), tx.clone());
        }
    }

    let msg = ChatMessage {
        sender: "system".into(),
        content: format!("Welcome, {}!", &username).into(),
    };
    tx.send(msg).ok();

    let un = username.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // text protocol of ws
            if let Ok(text) = msg.to_text() {
                if let Ok(value) = serde_json::to_value(text) {
                    let chat_msg = ChatMessage {
                        sender: un.clone(),
                        content: value,
                    };

                    // send to MQ
                    if let Some(ref m) = mqtx {
                        let _ = m.send(chat_msg.clone());
                    }

                    println!("[ws] {:?}", &chat_msg);
                }
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv() {
            if let Ok(text) = serde_json::to_string(&msg) {
                // to ws client
                if sender
                    .send(axum::extract::ws::Message::Text(text.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = &mut recv_task => recv_task.abort(),
        _ = &mut send_task => send_task.abort(),
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
