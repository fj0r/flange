use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub user: String,
    // TODO: dynamic Json
    pub message: String,
}

