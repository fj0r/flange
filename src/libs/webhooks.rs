use super::settings::{AssetsVariant, Webhook};
use minijinja::Value;
use reqwest::Error;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;

pub async fn webhook_post<T>(wh: &Webhook, msg: T) -> Result<T, Error>
where
    T: Debug + Serialize + DeserializeOwned,
{
    let client = reqwest::Client::new();
    let r = client.post(&wh.endpoint).json(&msg).send().await?;
    r.json::<T>().await
}

pub async fn webhook_get(wh: &AssetsVariant, msg: &Value) -> Result<String, Error> {
    let client = reqwest::Client::new();
    if let AssetsVariant::Webhook {
        endpoint,
        accept: _,
    } = wh
    {
        let r = client.get(endpoint).json(&msg).send().await?;
        r.text().await
    } else {
        // TODO: reqwest Error
        Ok("{}".to_string())
    }
}
