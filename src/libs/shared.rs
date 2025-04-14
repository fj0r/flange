use super::message::ChatMessage;
use std::collections::HashMap;
use std::sync::{mpsc, Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Clone)]
pub struct Shared {
    pub sender: HashMap<String, mpsc::Sender<ChatMessage>>,
}

impl Shared {
    pub fn init() -> Self {
        Shared {
            sender: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SharedState (Arc<RwLock<Shared>>);

impl SharedState {
    pub fn new() -> Self {
        SharedState(Arc::new(RwLock::new(Shared::init())))
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<Shared>> {
        self.0.read()
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<Shared>> {
        self.0.write()
    }
}
