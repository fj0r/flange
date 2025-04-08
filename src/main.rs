use axum::{Router, routing::get};
use libs::message::MessageQueue;
use serde_json::Value;
use std::sync::{Arc, RwLock};

mod libs;
use anyhow::{Ok, Result};
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use libs::admin::admin_router;
use libs::channel::handle_socket;
use libs::kafka::KafkaManager;
use libs::settings::Settings;
use libs::shared::{Shared, SharedState};

#[tokio::main]
async fn main() -> Result<()> {
    let settings = Settings::new()?;

    dbg!(&settings);

    let shared = Arc::new(RwLock::new(Shared::init()));

    let mq = if settings.kafka.enable {
        let mut mq = KafkaManager::<Value>::new(
            settings.kafka.consumer.clone(),
            settings.kafka.producer.clone(),
        );
        mq.run();
        Some(mq)
    } else {
        None
    };

    let app = Router::new()
        .route(
            "/channel",
            get(
                |ws: WebSocketUpgrade, State(state): State<SharedState>| async move {
                    ws.on_upgrade(|socket| handle_socket(socket, state, mq))
                },
            ),
        )
        .nest("/admin", admin_router())
        .with_state(shared);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
