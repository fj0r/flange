use super::error::AppError;
use super::{
    message::Envelope,
    settings::{AssetsList, Login, WebhookMap},
    shared::{Info, Sender, Session, StateChat},
};
use axum::{
    Router,
    extract::{Json, Path, Request, State},
    http::{StatusCode, header::ACCEPT},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use minijinja::Environment;
use serde::Serialize;
use serde_json::{Map, Value, from_str};

async fn send(
    State(state): State<StateChat<Sender>>,
    Json(payload): Json<Envelope>,
) -> Result<(StatusCode, Json<Vec<Session>>), AppError> {
    let mut succ: Vec<Session> = Vec::new();
    let s = state.read().await;
    if payload.receiver.is_empty() {
        for (n, c) in &s.session {
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

async fn list(State(state): State<StateChat<Sender>>) -> axum::Json<Vec<Session>> {
    let s = state.read().await;
    let mut r = Vec::new();
    for (k, _v) in &s.session {
        r.push(k.clone());
    }
    Json(r)
}

async fn info(
    Path(user): Path<String>,
    State(state): State<StateChat<Sender>>,
) -> axum::Json<Map<String, Value>> {
    let s = state.read().await;
    let u = s.session.get(&user.as_str().into()).and_then(|x| x.info.clone());
    Json(u.unwrap_or_else(|| Map::new()))
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
        .route("/sessions", get(list))
        .route("/info/{user}", get(info))
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
    Json(mut payload): Json<Map<String, Value>>,
) -> Result<Json<(Session, Info)>, AppError> {
    use short_uuid::ShortUuid;
    let uuid = ShortUuid::generate().to_string();
    payload.insert("username".into(), uuid[..6].into());
    Ok(Json((uuid.as_str().into(), Some(payload))))
}

async fn logout(
    State(_state): State<StateChat<Sender>>,
    Json(payload): Json<Map<String, Value>>,
) -> Result<Json<(Session, Info)>, AppError> {
    Ok(Json(("".into(), Some(payload))))
}

async fn health(State(state): State<StateChat<Sender>>) -> Result<Json<Value>, AppError> {
    let mut b = Map::new();
    let count = state.read().await.count as u64;
    b.insert("count".to_string(), count.into());
    Ok(axum::Json(Value::Object(b)))
}

pub fn debug_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/render/{name}", post(render))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/echo", post(echo))
        .route("/health", get(health))
}

#[derive(Serialize)]
struct ConfigList {
    login: Login,
    greet: AssetsList,
    webhook: WebhookMap,
}

async fn list_config(
    State(state): State<StateChat<Sender>>,
) -> Result<(StatusCode, Json<ConfigList>), AppError> {
    let s = state.read().await;
    let s = s.settings.read().await.clone();
    Ok((
        StatusCode::OK,
        Json(ConfigList {
            login: s.login,
            greet: s.greet,
            webhook: s.webhooks,
        }),
    ))
}

async fn update_login(
    State(state): State<StateChat<Sender>>,
    Json(payload): Json<Login>,
) -> Result<(StatusCode, Json<bool>), AppError> {
    let s = state.write().await;
    let mut s = s.settings.write().await;
    s.login = payload;
    Ok((StatusCode::OK, Json(true)))
}

async fn update_greet(
    State(state): State<StateChat<Sender>>,
    Json(payload): Json<AssetsList>,
) -> Result<(StatusCode, Json<bool>), AppError> {
    let s = state.write().await;
    let mut s = s.settings.write().await;
    s.greet = payload;
    Ok((StatusCode::OK, Json(true)))
}

async fn update_webhook(
    State(state): State<StateChat<Sender>>,
    Json(payload): Json<WebhookMap>,
) -> Result<(StatusCode, Json<bool>), AppError> {
    let s = state.write().await;
    let mut s = s.settings.write().await;
    s.webhooks = payload;
    Ok((StatusCode::OK, Json(true)))
}

pub fn config_router() -> Router<StateChat<Sender>> {
    Router::new()
        .route("/list", get(list_config))
        .route("/login", post(update_login))
        .route("/greet", post(update_greet))
        .route("/webhook", post(update_webhook))
}
