use axum::{
    routing::get,
    Router,
};
use std::sync::{Arc, RwLock};

mod libs;
use libs::channel::ws_handler;
use libs::admin::admin_router;
use libs::shared::Shared;

#[tokio::main]
async fn main() {
    let shared = Arc::new(RwLock::new(Shared::init()));

    let app = Router::new()
        .route("/channel", get(ws_handler))
        .nest("/admin", admin_router())
        .with_state(shared);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("WebSocket chat server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

