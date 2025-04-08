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
    message: message::ChatMessage,
}

async fn send(
    State(state): State<SharedState>,
    Json(payload): Json<Envelope>,
) -> Result<(StatusCode, Json<Vec<String>>), AppError> {
    let s = state.read().unwrap();
    let mut succ: Vec<String> = Vec::new();
    if payload.receiver.is_empty() {
        for (n, c) in s.sender.iter() {
            let c = c.lock().unwrap().clone();
            let _ = c.send(payload.message.clone());
            succ.push(n.into());
        }
    } else {
        for r in payload.receiver {
            if s.sender.contains_key(&r) {
                let x = s.sender.get(&r);
                let x = x.unwrap().lock().unwrap().clone();
                let _ = x.send(payload.message.clone());
                succ.push(r);
            }
        }
    }
    Ok((StatusCode::OK, succ.into()))
}

pub fn admin_router() -> Router<SharedState> {
    async fn list(State(state): State<SharedState>) -> axum::Json<Vec<String>> {
        let s = state.read().unwrap();
        Json(s.sender.keys().cloned().collect::<Vec<String>>())
    }

    Router::new()
        .route("/users", get(list))
        .route("/message", post(send))
}
