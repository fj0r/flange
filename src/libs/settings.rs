use figment::{providers::{Env, Format, Toml}, Figment, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct QueuePush {
    #[serde(rename = "type")]
    pub kind: String,
    pub broker: Vec<String>,
    pub topic: Vec<String>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct QueueEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub broker: Vec<String>,
    pub topic: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Queue {
    pub enable: bool,
    pub event: QueueEvent,
    pub push: QueuePush,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Webhook {
    pub enable: bool,
    pub event: String,
    pub endpoint: String,
    pub method: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct Settings {
    pub queue: Queue,
    pub webhooks: Vec<Webhook>,
}

impl Settings {
    pub(crate) fn new() -> Result<Self> {
        Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::prefixed("app_").split("_"))
            .extract()
    }
}
