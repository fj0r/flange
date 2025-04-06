use axum::{
    routing::get,
    Router,
};

mod libs;
use libs::channel::ws_handler;
use libs::shared::Shared;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    let shared = Arc::new(RwLock::new(Shared::init()));

    let app = Router::new()
        .route("/channel", get(ws_handler))
        .with_state(shared);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("WebSocket chat server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

