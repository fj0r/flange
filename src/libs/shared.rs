use super::message::{ChatMessage, Session, SessionCount};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub struct Shared<T> {
    pub sender: HashMap<Session, T>,
    pub count: SessionCount,
}

impl<T> Shared<T> {
    pub fn new() -> Self {
        Shared {
            sender: HashMap::new(),
            count: SessionCount::default(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct SharedState<T> (Arc<Mutex<Shared<T>>>);

impl<T> SharedState<T> {
    pub fn new() -> Self {
        SharedState::<T>(Arc::new(Mutex::new(Shared::new())))
    }

    pub async fn read(&self) -> MutexGuard<Shared<T>> {
        self.0.lock().await
    }

    pub async fn write(&self) -> MutexGuard<Shared<T>> {
        self.0.lock().await
    }
}


pub type StateChat = SharedState<UnboundedSender<ChatMessage>>;
