use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct KafkaConsumer {
    pub broker: Vec<String>,
    pub topic: Vec<String>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct KafkaProducer {
    pub broker: Vec<String>,
    pub topic: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Kafka {
    pub enable: bool,
    pub consumer: KafkaConsumer,
    pub producer: KafkaProducer,
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
    pub kafka: Kafka,
    pub webhooks: Vec<Webhook>,
}

impl Settings {
    pub(crate) fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config"))
            .add_source(Environment::with_prefix("app").separator("_"))
            .build()?;

        s.try_deserialize()
    }
}
