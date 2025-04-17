use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc::{UnboundedSender, UnboundedReceiver}, Mutex};
use std::fmt::Debug;

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

pub trait MessageQueue {
    type Item: Debug + Send + Serialize + serde::de::DeserializeOwned;

    #[allow(unused)]
    async fn run(&mut self);

    #[allow(unused)]
    fn get_rx(&self) -> Option<Arc<Mutex<UnboundedReceiver<Self::Item>>>>;

    #[allow(unused)]
    fn get_tx(&self) -> Option<UnboundedSender<Self::Item>>;
}

