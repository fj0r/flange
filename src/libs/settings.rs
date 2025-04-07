use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Kafka {
    consumer: Option<String>,
    producer: Option<String>
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub(crate) struct Settings {
    kafka: Option<Kafka>
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
