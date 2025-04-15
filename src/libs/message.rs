use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{
    Arc,
    Mutex,
    mpsc::{Sender, Receiver, SendError},
};
use std::fmt::Debug;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub sender: String,
    pub content: Value,
}

pub trait MessageQueue {
    type Item: Debug + Send + Serialize + serde::de::DeserializeOwned;

    #[allow(unused)]
    async fn run(&mut self);

    #[allow(unused)]
    fn get_rx(&self) -> Option<Arc<Mutex<Receiver<Self::Item>>>>;

    #[allow(unused)]
    fn get_tx(&self) -> Option<Sender<Self::Item>>;
}
