use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Debug, Display};
use std::ops::Deref;
use std::sync::Arc;
use time::OffsetDateTime;
use time::serde::rfc3339;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

pub trait Event {
    fn event(&self) -> Option<&str>;
    fn set_time(&mut self, time: Created);
}

impl Event for Value {
    fn event(&self) -> Option<&str> {
        if self.is_object() {
            if let Some(m) = self.as_object() {
                let r = m
                    .get("event")
                    .and_then(|x| x.as_str());
                return r;
            };
        };
        None
    }
    fn set_time(&mut self, _time: Created) {
        unreachable!()
    }
}

pub type SessionCount = u128;
pub type SessionId = String;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, Eq, Hash)]
pub struct Session(pub SessionId);

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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Created(#[serde(with = "rfc3339")] pub OffsetDateTime);

impl Default for Created {
    fn default() -> Self {
        Self (OffsetDateTime::now_utc())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Envelope {
    pub receiver: Vec<Session>,
    #[serde(flatten)]
    pub message: ChatMessage,
}

impl Event for Envelope {
    fn event(&self) -> Option<&str> {
        self.message.event()
    }
    fn set_time(&mut self, time: Created) {
        self.message.set_time(time);
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ChatMessage {
    pub sender: Session,
    pub created: Option<Created>,
    pub content: Value,
}


impl From<(Session, Value)> for ChatMessage {
    fn from(value: (Session, Value)) -> Self {
        ChatMessage {
            sender: value.0,
            created: Some(Created::default()),
            content: value.1,
        }
    }
}

impl Event for ChatMessage {
    fn event(&self) -> Option<&str> {
        self.content.event()
    }

    fn set_time(&mut self, time: Created) {
        self.created = Some(time);
    }
}

pub trait MessageQueueEvent {
    type Item: Debug + Send + Serialize + serde::de::DeserializeOwned;

    #[allow(unused)]
    async fn run(&mut self);

    #[allow(unused)]
    fn get_tx(&self) -> Option<UnboundedSender<Self::Item>>;
}

pub trait MessageQueuePush {
    type Item: Debug + Send + Serialize + serde::de::DeserializeOwned;

    #[allow(unused)]
    async fn run(&mut self);

    #[allow(unused)]
    fn get_rx(&self) -> Option<Arc<Mutex<UnboundedReceiver<Self::Item>>>>;
}
