use super::message::ChatMessage;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, mpsc};

#[derive(Debug, Clone)]
pub struct Shared {
    pub sender: HashMap<String, Arc<mpsc::Sender<ChatMessage>>>,
}

impl Shared {
    pub fn init() -> Self {
        Shared {
            sender: HashMap::new(),
        }
    }
}

pub type SharedState = Arc<RwLock<Shared>>;
