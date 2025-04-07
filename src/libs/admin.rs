use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use serde::Deserialize;

use super::{message, shared::SharedState};

#[derive(Deserialize)]
struct Envelope {
    receiver: Vec<String>,
    content: message::ChatMessage
}

async fn send(
    State(state): State<SharedState>,
    Json(payload): Json<Envelope>
) -> (StatusCode, Json<Vec<String>>) {
    let s = state.read().unwrap();
    let mut succ = Vec::new();
    for r in payload.receiver {
        if s.sender.contains_key(&r) {
            let x = s.sender.get(&r);
            let x = x.unwrap().lock().unwrap().clone();
            let _ = x.send(payload.content.clone());
            succ.push(r);
        }
    }
    (StatusCode::OK, succ.into())
}


pub fn admin_router() -> Router<SharedState> {
    async fn list(
        State(state): State<SharedState>
    ) -> axum::Json<Vec<String>> {
        let s = state.read().unwrap();
        Json(s.sender.keys().cloned().collect::<Vec<String>>())
    }

    Router::new()
        .route("/users", get(list))
        .route("/message", post(send))
}

