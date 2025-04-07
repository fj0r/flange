use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KafkaConsumer {
    pub broker: String,
    pub topic: String,
    pub group: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct KafkaProducer {
    pub broker: String,
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
    pub endpoint: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct Settings {
    pub kafka: Kafka,
    pub webhook: Webhook,
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
