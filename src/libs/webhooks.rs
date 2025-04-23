use std::fmt::Debug;
use super::settings::Webhook;
use reqwest::Error;
use serde::{Serialize, de::DeserializeOwned};

pub async fn handle_webhook<'a, T>(wh: &'a Webhook, msg: T) -> Result<T, Error>
where
    T: Debug + Serialize + DeserializeOwned
{
    let client = reqwest::Client::new();
    let r = client.post(&wh.endpoint)
        .json(&msg)
        .send()
        .await?;
    r.json::<T>().await
}
