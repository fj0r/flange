use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router};

use super::{message, shared::SharedState};

async fn send(State(state): State<SharedState>) {
    state;
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

