use super::message::MessageQueue;
use super::settings::{KafkaConsumer, KafkaProducer};
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::producer::{Producer, Record, RequiredAcks};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use tokio::task::spawn_blocking;

#[derive(Clone)]
pub struct KafkaManager<T>
where
    T: Send + Serialize + DeserializeOwned,
{
    rx: Option<Arc<Mutex<mpsc::Receiver<T>>>>,
    tx: Option<mpsc::Sender<T>>,
    consumer: KafkaConsumer,
    producer: KafkaProducer,
}

impl<T> KafkaManager<T>
where
    T: Send + Serialize + DeserializeOwned + 'static,
{
    pub fn new(consumer: KafkaConsumer, producer: KafkaProducer) -> Self {
        Self {
            rx: None,
            tx: None,
            consumer,
            producer,
        }
    }
}

impl<T> MessageQueue for KafkaManager<T>
where
    T: Debug + Clone + Send + Serialize + DeserializeOwned + 'static,
{
    type Item = T;

    async fn run(&mut self) {
        let (producer_tx, producer_rx) = mpsc::channel::<Self::Item>();
        let producer_cfg = self.producer.clone();
        spawn_blocking(move || {
            let mut producer = Producer::from_hosts(producer_cfg.broker)
                .with_ack_timeout(Duration::from_secs(1))
                .with_required_acks(RequiredAcks::One)
                .create()
                .expect("Failed to create Kafka producer");

            while let Ok(value) = producer_rx.recv() {
                let value = serde_json::to_string(&value).expect("serde to string");
                if let Err(e) = producer.send(&Record {
                    key: (),
                    value,
                    topic: &producer_cfg.topic,
                    partition: -1,
                }) {
                    eprintln!("Failed to send message to Kafka: {}", e);
                }
            }
        });

        let (consumer_tx, consumer_rx) = mpsc::channel::<Self::Item>();
        let consumer_cfg = self.consumer.clone();
        spawn_blocking(move || {
            let mut consumer = Consumer::from_hosts(consumer_cfg.broker)
                .with_topic(consumer_cfg.topic)
                .with_group(consumer_cfg.group.unwrap_or("default".into()))
                .with_fallback_offset(FetchOffset::Earliest)
                .with_offset_storage(Some(GroupOffsetStorage::Kafka))
                .create()
                .expect("Failed to create Kafka consumer");

            loop {
                if let Ok(c) = consumer.poll() {
                    for ms in c.iter() {
                        for m in ms.messages() {
                            dbg!(&m);
                            if let Ok(value) = serde_json::from_slice::<Self::Item>(m.value) {
                                if let Err(e) = consumer_tx.send(value) {
                                    eprintln!("Failed to send message from consumer: {}", e);
                                }
                            }
                        }
                        consumer.consume_messageset(ms).expect("consume_messageset");
                    }
                }
                consumer.commit_consumed().expect("commit_consumed");
            }
        });

        self.tx = Some(producer_tx);
        self.rx = Some(Arc::new(Mutex::new(consumer_rx)));
    }

    fn get_rx(&self) -> Option<Arc<Mutex<mpsc::Receiver<Self::Item>>>> {
        self.rx.clone()
    }

    fn get_tx(&self) -> Option<mpsc::Sender<Self::Item>> {
        self.tx.clone()
    }
}
