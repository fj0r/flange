use axum::{
    routing::get,
    Router,
};
use std::sync::{Arc, RwLock};

mod libs;
use libs::channel::ws_handler;
use libs::admin::admin_router;
use libs::shared::Shared;
use anyhow::{Result, Ok};
use libs::settings::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::new()?;

    dbg!(settings);

    let shared = Arc::new(RwLock::new(Shared::init()));

    let app = Router::new()
        .route("/channel", get(ws_handler))
        .nest("/admin", admin_router())
        .with_state(shared);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

