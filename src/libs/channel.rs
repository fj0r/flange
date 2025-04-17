use std::fmt::Debug;
use axum::extract::ws::WebSocket;
use serde::{Serialize, Deserialize};
//use axum::extract::State;
use super::shared::SharedState;
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc::UnboundedSender;
use serde_json::Value;

// pub async fn ws_handler(
//     mq: Option<impl MessageQueue + Send>,
// ) -> Box<dyn FnOnce(WebSocketUpgrade, State<SharedState>) -> Response> {
//     Box::new(
//         move |ws: WebSocketUpgrade, State(state): State<SharedState>|  {
//             ws.on_upgrade(|socket| handle_socket(socket, state, mq))
//         }
//     )
// }

pub async fn handle_socket<T>(
    socket: WebSocket,
    state: SharedState<UnboundedSender<T>>,
    mqtx: Option<UnboundedSender<T>>,
) where
    T: for<'a> Deserialize<'a> + Serialize + From<(String, Value)> + Clone + Debug + Send + 'static
{
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();
    let username: String;

    {
        let s1 = state.clone();
        let mut s = s1.write().await;
        s.count += 1;
        username = format!("user_{}", s.count);
        s.sender.insert(username.clone(), tx.clone());
    }

    let un = username.clone();

    let mut recv_task = tokio::spawn(async move {
        /* FIXME: stuck
        let msg = ChatMessage {
            sender: "system".into(),
            content: format!("Welcome, {}!", un).into(),
        };
        tx.send(msg).ok();
        */

        while let Some(Ok(msg)) = receiver.next().await {
            // text protocol of ws
            if let Ok(text) = msg.to_text() {
                if let Ok(value) = serde_json::to_value(text) {
                    let chat_msg: T = (un.clone(), value).into();

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
        while let Some(msg) = rx.recv().await {
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
    let mut s = state.write().await;
    s.sender.remove(&username);
}

#[allow(unused)]
trait Client {
    fn on_init() {}
    fn on_message() {}
}
