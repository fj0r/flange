use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{
    Arc,
    mpsc::{Receiver, SendError},
};
use std::fmt::Debug;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub user: String,
    pub content: Value,
}

pub trait MessageQueue {
    type Item: Debug + Send + Serialize + serde::de::DeserializeOwned;

    #[allow(unused)]
    async fn run(&mut self);

    #[allow(unused)]
    async fn send(&self, value: &Self::Item) -> Result<(), SendError<Self::Item>>;

    #[allow(unused)]
    fn listen(&self) -> Option<Arc<Mutex<Receiver<Self::Item>>>>;
}
