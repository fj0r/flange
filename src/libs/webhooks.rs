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

#[derive(thiserror::Error, Debug)]
pub enum GreetError {
    #[error("reqwest error")]
    Reqwest(#[from] Error),
    #[error("not a webhook")]
    NotWebhook
}

pub async fn greet_post(wh: &AssetsVariant, msg: &Value) -> Result<String, GreetError> {
    let client = reqwest::Client::new();
    match wh {
        AssetsVariant::Webhook {
            endpoint,
            accept: _,
        } => {
            let r = client.post(endpoint).json(&msg).send().await?;
            Ok(r.text().await?)
        }
        _ => {
            Err(GreetError::NotWebhook)
        }
    }
}
