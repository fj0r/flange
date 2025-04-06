use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::collections::HashMap;
use super::message::ChatMessage;

#[derive(Debug, Clone)]
pub struct Shared {
    pub sender: HashMap<String, Arc<Mutex<mpsc::Sender<ChatMessage>>>>
}

impl Shared {
    pub fn init() -> Self {
        Shared {
            sender: HashMap::new()
        }
    }
}

pub type SharedState = Arc<RwLock<Shared>>;
