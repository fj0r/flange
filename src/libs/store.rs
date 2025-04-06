use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use super::message::ChatMessage;

#[derive(Debug, Clone)]
pub struct Store {
    pub sender: Arc<Mutex<broadcast::Sender<ChatMessage>>> 
}

impl Store {
    pub fn init() -> Self {

        let (tx, _rx) = broadcast::channel::<ChatMessage>(100);

        Store {
            sender: Arc::new(Mutex::new(tx))
        }
    }
}
