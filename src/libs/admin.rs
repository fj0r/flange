use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;

use super::error::AppError;
use super::{message, shared::SharedState};

#[derive(Deserialize)]
struct Envelope {
    receiver: Vec<String>,
    #[serde(flatten)]
    message: message::ChatMessage,
}

async fn send(
    State(state): State<SharedState>,
    Json(payload): Json<Envelope>,
) -> Result<(StatusCode, Json<Vec<String>>), AppError> {
    let mut succ: Vec<String> = Vec::new();
    if let Ok(s) = state.read() {
        if payload.receiver.is_empty() {
            for (n, c) in s.sender.iter() {
                let _ = c.send(payload.message.clone());
                succ.push(n.into());
            }
        } else {
            for r in payload.receiver {
                if s.sender.contains_key(&r) {
                    if let Some(x) = s.sender.get(&r) {
                        let _ = x.send(payload.message.clone());
                        succ.push(r);
                    }
                }
            }
        }
    }
    Ok((StatusCode::OK, succ.into()))
}

async fn list(State(state): State<SharedState>) -> axum::Json<Vec<String>> {
    if let Ok(s) = state.read() {
        Json(s.sender.keys().cloned().collect::<Vec<String>>())
    } else {
        vec![].into()
    }
}

pub fn admin_router() -> Router<SharedState> {
    Router::new()
        .route("/users", get(list))
        .route("/message", post(send))
}
