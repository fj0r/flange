use axum::{
    routing::get,
    Router,
};

mod libs;
use libs::channel::ws_handler;
use libs::store::Store;

#[tokio::main]
async fn main() {
    let store = Store::init();

    let app = Router::new()
        .route("/channel", get(ws_handler))
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("WebSocket chat server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

