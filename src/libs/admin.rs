use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use super::error::AppError;
use super::{message, shared::StateChat};

async fn send(
    State(state): State<StateChat>,
    Json(payload): Json<message::Envelope>,
) -> Result<(StatusCode, Json<Vec<String>>), AppError> {
    let mut succ: Vec<String> = Vec::new();
    let s = state.read().await;
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
    Ok((StatusCode::OK, succ.into()))
}

async fn list(State(state): State<StateChat>) -> axum::Json<Vec<String>> {
    let s = state.read().await;
        Json(s.sender.keys().cloned().collect::<Vec<String>>())
}

pub fn admin_router() -> Router<StateChat> {
    Router::new()
        .route("/users", get(list))
        .route("/message", post(send))
}
