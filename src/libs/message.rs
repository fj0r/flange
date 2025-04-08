use std::sync::mpsc::{SendError, Receiver};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub user: String,
    pub content: Value,
}

pub trait MessageQueue<T> {
    fn run (&mut self);
    fn send (&self, value: T) -> Result<(), SendError<T>>;
    fn listen (&self) -> &Option<Receiver<T>>;
}
