use super::message::ChatMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub struct Shared {
    pub sender: HashMap<String, UnboundedSender<ChatMessage>>,
    pub count: u128,
}

impl Shared {
    pub fn new() -> Self {
        Shared {
            sender: HashMap::new(),
            count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SharedState (Arc<Mutex<Shared>>);

impl SharedState {
    pub fn new() -> Self {
        SharedState(Arc::new(Mutex::new(Shared::new())))
    }

    pub async fn read(&self) -> MutexGuard<Shared> {
        self.0.lock().await
    }

    pub async fn write(&self) -> MutexGuard<Shared> {
        self.0.lock().await
    }
}
