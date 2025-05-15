use axum::{
    Router,
    extract::{Json, Request, State},
    http::{StatusCode, header::ACCEPT},
    response::{IntoResponse, Response},
    routing::{get, post},
};

use super::error::AppError;
use super::{message::{Envelope, Session}, shared::StateChat};
use serde_json::{Value, from_str};

async fn send(
    State(state): State<StateChat>,
    Json(payload): Json<Envelope>,
) -> Result<(StatusCode, Json<Vec<Session>>), AppError> {
    let mut succ: Vec<Session> = Vec::new();
    let s = state.read().await;
    if payload.receiver.is_empty() {
        for (n, c) in s.sender.iter() {
            let _ = c.send(payload.message.clone());
            succ.push(n.to_owned());
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

async fn list(State(state): State<StateChat>) -> axum::Json<Vec<Session>> {
    let s = state.read().await;
    Json(s.sender.keys().cloned().collect::<Vec<Session>>())
}

struct Req<'a>(&'a Request);
impl std::fmt::Display for Req<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = writeln!(f, "### {} {}", self.0.method(), self.0.uri());
        for (name, value) in self.0.headers() {
            let _ = writeln!(f, "  | {}: {:?}", name, value);
        }
        Ok(())
    }
}

async fn echo(req: Request) -> Result<Response, AppError> {
    println!("{}", Req(&req));
    match req.headers().get(ACCEPT).map(|x| x.as_bytes()) {
        Some(b"application/json") => {
            let body = req.into_body();
            let limit = 204800usize;
            let by = axum::body::to_bytes(body, limit).await?;
            let s = String::from_utf8(by.to_vec())?;
            Ok(Json(from_str::<Value>(&s)?).into_response())
        }
        _ => Ok(req.into_body().into_response()),
    }
}

pub fn admin_router() -> Router<StateChat> {
    Router::new()
        .route("/users", get(list))
        .route("/message", post(send))
        .route("/echo", post(echo))
}
