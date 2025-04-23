use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

pub trait Event {
    fn event(&self) -> Option<&str>;
}

impl Event for Value {
    fn event(&self) -> Option<&str> {
        if self.is_object() {
            if let Some(m) = self.as_object() {
                let r = m
                    .get("event")
                    .and_then(|x| x.as_str());
                return r;
            };
        };
        return None;
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Envelope {
    pub receiver: Vec<String>,
    #[serde(flatten)]
    pub message: ChatMessage,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ChatMessage {
    pub sender: String,
    pub content: Value,
}

impl From<(String, Value)> for ChatMessage {
    fn from(value: (String, Value)) -> Self {
        ChatMessage {
            sender: value.0,
            content: value.1,
        }
    }
}

impl Event for ChatMessage {
    fn event(&self) -> Option<&str> {
        self.content.event()
    }
}

pub trait MessageQueueEvent {
    type Item: Debug + Send + Serialize + serde::de::DeserializeOwned;

    #[allow(unused)]
    async fn run(&mut self);

    #[allow(unused)]
    fn get_tx(&self) -> Option<UnboundedSender<Self::Item>>;
}

pub trait MessageQueuePush {
    type Item: Debug + Send + Serialize + serde::de::DeserializeOwned;

    #[allow(unused)]
    async fn run(&mut self);

    #[allow(unused)]
    fn get_rx(&self) -> Option<Arc<Mutex<UnboundedReceiver<Self::Item>>>>;
}
