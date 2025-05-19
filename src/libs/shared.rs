use super::message::ChatMessage;
use std::{collections::HashMap, ops::Deref};
use std::sync::Arc;
use tokio::sync::{mpsc::UnboundedSender, Mutex, MutexGuard};
use serde_json::{Value, Map};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

pub type SessionCount = u128;
pub type SessionId = String;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash)]
pub struct Session(pub SessionId);

impl From<&str> for Session {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<SessionCount> for Session {
    fn from(value: SessionCount) -> Self {
        Self(value.to_string())
    }
}

impl Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for Session {
    type Target = SessionId;
    fn deref(&self) -> &Self::Target {
        &self.0
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

#[derive(Debug, Clone)]
pub struct Shared<T> {
    pub session: HashMap<Session, T>,
    pub count: SessionCount,
}

impl<T> Shared<T> {
    pub fn new() -> Self {
        Shared {
            session: HashMap::new(),
            count: SessionCount::default(),
        }
    }
}

pub type Info = Option<Map<String, Value>>;

#[derive(Debug, Clone)]
pub struct Client<T> {
    pub sender: T,
    pub info: Info
}

impl<T> Deref for Client<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

pub type Sender = UnboundedSender<ChatMessage>;

pub type StateChat<T> = SharedState<Client<T>>;
