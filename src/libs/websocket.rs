use super::message::{Event, Session};
use super::settings::{AssetsList, WebhookMap};
use super::webhooks::handle_webhook;
use anyhow::Ok as Okk;
use axum::extract::ws::WebSocket;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::LazyLock;
use tera::{Context, Tera};
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;

pub async fn handle_ws<T>(
    socket: WebSocket,
    event_tx: Option<UnboundedSender<T>>,
    state: SharedState<UnboundedSender<T>>,
    webhooks: Arc<RwLock<WebhookMap>>,
    greet: AssetsList,
) where
    T: Event
        + for<'a> Deserialize<'a>
        + Serialize
        + From<(Session, Value)>
        + Clone
        + Debug
        + Send
        + 'static,
{
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();
    let sid: Session;

    {
        let s1 = state.clone();
        let mut s = s1.write().await;
        s.count += 1;
        sid = s.count.into();
        s.sender.insert(sid.clone(), tx.clone());
    }

    tracing::info!("Connection opened for {}", &sid);
    let mut context = Context::new();
    context.insert("session_id", &sid);
    static TERA: LazyLock<Tera> = LazyLock::new(|| Tera::new("assets/*").unwrap());
    for g in greet.iter() {
        if !g.enable {
            continue;
        }
        let s = TERA.render(&g.path, &context).unwrap();
        let v: Value = from_str(&s).unwrap();
        let msg: T = (Session(0), v).into();
        if let Ok(text) = serde_json::to_string(&msg) {
            let _ = sender
                .send(axum::extract::ws::Message::Text(text.into()))
                .await;
        }
    }

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
        Okk(())
    });

    let sid_cloned = sid.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // text protocol of ws
            let text = msg.to_text()?;
            let value = serde_json::from_str(text)?;
            let chat_msg: T = (sid_cloned.clone(), value).into();

            let mut is_webhook: bool = false;
            if let Some(ev) = chat_msg.event() {
                let whs = webhooks.read().await;
                if whs.contains_key(ev) {
                    if let Some(wh) = whs.get(ev) {
                        if wh.enable {
                            is_webhook = true;
                            if let Ok(r) = handle_webhook(wh, chat_msg.clone()).await {
                                let _ = tx.send(r);
                            } else {
                                let mut context = Context::new();
                                context.insert("session_id", &sid_cloned);
                                context.insert("event", &ev);
                                let t = TERA.render("webhook_error.json", &context).unwrap();
                                let _ = tx.send(serde_json::from_str(&t)?);
                            }
                        }
                    }
                }
            }

            // send to event MQ
            if !is_webhook {
                if let Some(ref m) = event_tx {
                    let _ = m.send(chat_msg.clone());
                }
            }

            tracing::debug!("[ws] {:?}", &chat_msg);
        }
        Okk(())
    });

    tokio::select! {
        _ = &mut recv_task => recv_task.abort(),
        _ = &mut send_task => send_task.abort(),
    };

    tracing::info!("Connection closed for {}", &sid);
    let mut s = state.write().await;
    s.sender.remove(&sid);
}

#[allow(unused)]
trait Client {
    fn on_init() {}
    fn on_message() {}
}

use super::kafka::KafkaManagerPush;
use super::message::{ChatMessage, Envelope, MessageQueuePush};
use super::shared::SharedState;

pub async fn send_to_ws(
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
