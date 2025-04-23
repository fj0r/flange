use anyhow::Ok;
use axum::extract::ws::WebSocket;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;

pub async fn handle_socket<T>(
    socket: WebSocket,
    state: SharedState<UnboundedSender<T>>,
    mqtx: Option<UnboundedSender<T>>,
) where
    T: for<'a> Deserialize<'a> + Serialize + From<(String, Value)> + Clone + Debug + Send + 'static,
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

        while let Some(std::result::Result::Ok(msg)) = receiver.next().await {
            // text protocol of ws
            let text = msg.to_text()?;
            let value = serde_json::to_value(text)?;
            let chat_msg: T = (un.clone(), value).into();

            // send to MQ
            if let Some(ref m) = mqtx {
                let _ = m.send(chat_msg.clone());
            }

            tracing::debug!("[ws] {:?}", &chat_msg);
        }
        Ok(())
    });

    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg)?;
            // to ws client
            if sender
                .send(axum::extract::ws::Message::Text(text.into()))
                .await
                .is_err()
            {
                break;
            }
        }
        Ok(())
    });

    tokio::select! {
        _ = &mut recv_task => recv_task.abort(),
        _ = &mut send_task => send_task.abort(),
    };

    tracing::info!("Connection closed for {}", &username);
    let mut s = state.write().await;
    s.sender.remove(&username);
}

#[allow(unused)]
trait Client {
    fn on_init() {}
    fn on_message() {}
}

use super::kafka::KafkaManagerPush;
use super::message::{ChatMessage, Envelope, MessageQueuePush};
use super::shared::SharedState;

pub async fn notify(
    push_mq: &KafkaManagerPush<Envelope>,
    shared: &SharedState<UnboundedSender<ChatMessage>>,
) {
    let mqrx = push_mq.get_rx();
    let shared = shared.clone();
    tokio::spawn(async move {
        let rx = mqrx?;
        let mut rx = rx.lock().await;

        while let Some(x) = rx.recv().await {
            if !x.receiver.is_empty() {
                let s = shared.read().await;
                for r in x.receiver {
                    if s.sender.contains_key(&r) {
                        let s = s.sender.get(&r)?;
                        let _ = s.send(x.message.clone());
                    }
                }
            }
        }
        Some(())
    });
}
