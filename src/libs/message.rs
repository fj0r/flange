use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ChatMessage {
    pub user: String,
    // TODO: dynamic Json
    pub message: String,
}

