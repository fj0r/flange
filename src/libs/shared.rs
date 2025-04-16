use super::message::ChatMessage;
use std::collections::HashMap;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Clone)]
pub struct Shared {
    pub sender: HashMap<String, tokio::sync::mpsc::Sender<ChatMessage>>,
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
pub struct SharedState (Arc<RwLock<Shared>>);

impl SharedState {
    pub fn new() -> Self {
        SharedState(Arc::new(RwLock::new(Shared::new())))
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<Shared>> {
        self.0.read()
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<Shared>> {
        self.0.write()
    }
}
