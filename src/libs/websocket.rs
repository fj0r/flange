use super::message::Event;
use super::settings::{Assets, AssetsVariant, Settings};
use super::shared::{Info, Session, StateChat};
use super::template::Tmpls;
use super::webhooks::{greet_post, login_post, webhook_post};
use anyhow::Result;
use anyhow::{Context, Ok as Okk};
use axum::extract::ws::WebSocket;
use futures::{sink::SinkExt, stream::StreamExt};
use minijinja::context;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{Mutex, RwLock};

/* TODO:
use std::async_iter;

struct GreetIter<'a> {
    index: usize,
    greet: &'a AssetsList,
    context: &'a minijinja::Value,
}

impl<'a> AsyncIterator for GreetIter<'a> {
}
*/

static TMPL: LazyLock<Tmpls> = LazyLock::new(|| Tmpls::new("assets").unwrap());

async fn handle_greet<T>(asset: &Assets, context: &minijinja::Value) -> Result<String>
where
    T: Event + Serialize + From<(Session, Value)>,
{
    if !asset.enable {
        return Ok("disabled".into());
    }
    let content = match &asset.variant {
        AssetsVariant::Path { path } => TMPL.get_template(path).unwrap().render(&context).ok(),
        wh @ AssetsVariant::Webhook { .. } => greet_post(&wh, context).await.ok(),
    };
    let v = from_str(&content.context("not a webhook")?)?;
    let msg: T = (Session::default(), v).into();
    Ok(serde_json::to_string(&msg)?)
}

async fn handle_login(
    settings: &Settings,
    query: &HashMap<String, String>,
) -> Option<(Session, Info)> {
    if settings.login.enable {
        let endpoint = &settings.login.endpoint.clone()?;
        let r = login_post(endpoint, query).await.ok()?;
        return Some((r.0.into(), r.1));
    }
    None
}

pub async fn handle_ws<T>(
    socket: WebSocket,
    event_tx: Option<UnboundedSender<T>>,
    state: StateChat<UnboundedSender<T>>,
    settings: Arc<RwLock<Settings>>,
    query: HashMap<String, String>,
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
    let settings = settings.read().await;
    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<T>();
    let sid: Session;

    if let Some((s, info)) = handle_login(&settings, &query).await {
        sid = s;
        let mut s = state.write().await;
        s.session.insert(
            sid.clone(),
            super::shared::Client {
                sender: tx.clone(),
                info,
            },
        );
    } else {
        let mut s = state.write().await;
        s.count += 1;
        sid = s.count.into();
        s.session.insert(
            sid.clone(),
            super::shared::Client {
                sender: tx.clone(),
                info: None,
            },
        );
    }

    tracing::info!("Connection opened for {}", &sid);

    let context = context! { session_id => &sid };

    for g in settings.greet.iter() {
        if let Ok(text) = handle_greet::<T>(g, &context).await {
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
    let webhooks = settings.webhooks.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            // text protocol of ws
            let text = msg.to_text()?;
            let value = serde_json::from_str(text)?;
            let chat_msg: T = (sid_cloned.clone(), value).into();

            let mut is_webhook: bool = false;
            if let Some(ev) = chat_msg.event() {
                if webhooks.contains_key(ev) {
                    if let Some(wh) = webhooks.get(ev) {
                        if wh.enable {
                            is_webhook = true;
                            if let Ok(r) = webhook_post(wh, chat_msg.clone()).await {
                                let _ = tx.send(r);
                            } else {
                                let context = context! {
                                    session_id => &sid_cloned,
                                    event => &ev
                                };
                                let t = TMPL
                                    .get_template("webhook_error.json")
                                    .unwrap()
                                    .render(&context)
                                    .unwrap();
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
    s.session.remove(&sid);
}

#[allow(unused)]
trait Client {
    fn on_init() {}
    fn on_message() {}
}

use super::message::{ChatMessage, Envelope};

pub async fn send_to_ws(
    mqrx: Arc<Mutex<UnboundedReceiver<Envelope>>>,
    shared: &StateChat<UnboundedSender<ChatMessage>>,
) {
    let shared = shared.clone();
    tokio::spawn(async move {
        let mut rx = mqrx.lock().await;

        while let Some(x) = rx.recv().await {
            if !x.receiver.is_empty() {
                let s = shared.read().await;
                for r in x.receiver {
                    if s.session.contains_key(&r) {
                        let s = s.session.get(&r)?;
                        let _ = s.send(x.message.clone());
                    }
                }
            }
        }
        Some(())
    });
}
