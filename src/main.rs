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

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

