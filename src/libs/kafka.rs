use super::message::MessageQueue;
use super::settings::{KafkaConsumer, KafkaProducer};
use futures::lock::Mutex;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use kafka::producer::{Producer, Record, RequiredAcks};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::{Arc, mpsc};
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
    T: Send + Serialize + DeserializeOwned + 'static,
{
    type Item = T;
    fn run(&mut self) {
        let (producer_tx, producer_rx) = mpsc::channel::<T>();

        spawn_blocking(move || {
            let mut producer = Producer::from_hosts(vec!["localhost:9092".to_string()])
                .with_ack_timeout(Duration::from_secs(1))
                .with_required_acks(RequiredAcks::One)
                .create()
                .expect("Failed to create Kafka producer");

            while let Ok(value) = producer_rx.recv() {
                let value = serde_json::to_string(&value).unwrap();
                if let Err(e) = producer.send(&Record {
                    key: (),
                    value,
                    topic: "",
                    partition: -1,
                }) {
                    eprintln!("Failed to send message to Kafka: {}", e);
                }
            }
        });

        let (consumer_tx, consumer_rx) = mpsc::channel::<T>();
        spawn_blocking(move || {
            let mut consumer = Consumer::from_hosts(vec!["localhost:9092".to_string()])
                .with_topic("chat_messages".to_string())
                .with_group("chat_group".to_string())
                .with_fallback_offset(FetchOffset::Earliest)
                .with_offset_storage(Some(GroupOffsetStorage::Kafka))
                .create()
                .expect("Failed to create Kafka consumer");

            loop {
                for ms in consumer.poll().unwrap().iter() {
                    for m in ms.messages() {
                        if let Ok(value) = serde_json::from_slice::<T>(m.value) {
                            if let Err(e) = consumer_tx.send(value) {
                                eprintln!("Failed to send message from consumer: {}", e);
                            }
                        }
                    }
                    consumer.consume_messageset(ms).unwrap();
                }
                consumer.commit_consumed().unwrap();
            }
        });

        self.tx = Some(producer_tx);
        self.rx = Some(Arc::new(Mutex::new(consumer_rx)));
    }

    fn send(&self, value: T) -> Result<(), mpsc::SendError<T>> {
        if let Some(tx) = &self.tx {
            let _ = tx.send(value).unwrap();
        }
        Ok(())
    }

    fn listen(&self) -> &Option<Arc<Mutex<mpsc::Receiver<T>>>> {
        &self.rx
    }
}
