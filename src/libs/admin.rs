use axum::{
    Router,
    extract::{Json, Path, Request, State},
    http::{StatusCode, header::ACCEPT},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use super::error::AppError;
use super::{
    message::Envelope,
    shared::{Info, Sender, Session, StateChat},
};
use minijinja::Environment;
use serde_json::{Map, Value, from_str};

async fn send(
    State(state): State<StateChat<Sender>>,
    Json(payload): Json<Envelope>,
) -> Result<(StatusCode, Json<Vec<Session>>), AppError> {
    let mut succ: Vec<Session> = Vec::new();
    let s = state.read().await;
    if payload.receiver.is_empty() {
        for (n, c) in s.session.iter() {
            let _ = c.send(payload.message.clone());
            succ.push(n.to_owned());
        }
    } else {
        for r in payload.receiver {
            if s.session.contains_key(&r) {
                if let Some(x) = s.session.get(&r) {
                    let _ = x.send(payload.message.clone());
                    succ.push(r);
                }
            }
        }
    }
    Ok((StatusCode::OK, succ.into()))
}


#[derive(Debug, Serialize)]
pub struct UserLine {
    id: Session,
    info: Info
}

async fn list(State(state): State<StateChat<Sender>>) -> axum::Json<Vec<UserLine>> {
    let s = state.read().await;
    let mut r = Vec::new();
    for (k, v) in &s.session {
        r .push(UserLine {
            id: k.clone(), info: v.info.clone()
        });
    }
    Json(r)
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

pub fn admin_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/users", get(list))
        .route("/send", post(send))
}

async fn render(
    Path(name): Path<String>,
    Json(payload): Json<Value>,
) -> Result<Response, AppError> {
    let mut env = Environment::new();
    let path = std::path::Path::new("assets");
    let content = async_fs::read_to_string(path.join(&name)).await?;
    let _ = env.add_template_owned(&name, content);
    let r = env.get_template(&name)?.render(payload)?;
    Ok(Response::new(r.into()))
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

async fn login(
    State(_state): State<StateChat<Sender>>,
    Json(payload): Json<Map<String, Value>>,
) -> Result<Json<(Session, Info)>, AppError> {
    use short_uuid::ShortUuid;
    let uuid = ShortUuid::generate().to_string();
    Ok(Json((uuid.as_str().into(), Some(payload))))
}

pub fn debug_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/render/{name}", post(render))
        .route("/login", post(login))
        .route("/echo", post(echo))
}


async fn list_config(
    State(state): State<StateChat<Sender>>,
    Json(payload): Json<Envelope>,
) -> Result<(StatusCode, Json<Vec<Session>>), AppError> {
    let mut succ: Vec<Session> = Vec::new();
    Ok((StatusCode::OK, succ.into()))
}


pub fn config_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/list", get(list_config))
}
