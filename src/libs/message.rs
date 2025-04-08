use std::sync::mpsc::{SendError, Receiver};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub user: String,
    pub content: Value,
}

pub trait MessageQueue {
    type Item;
    fn run (&mut self);
    fn send (&self, value: Self::Item) -> Result<(), SendError<Self::Item>>;
    fn listen (&self) -> &Option<Receiver<Self::Item>>;
}
