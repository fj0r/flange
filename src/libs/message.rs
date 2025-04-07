use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub user: String,
    pub content: Value,
}

