use axum::{Router, routing::get};
use libs::message::{ChatMessage, Envelope, MessageQueueEvent, MessageQueuePush};

use tracing::info;
use tracing_subscriber;
mod libs;
use anyhow::{Ok, Result};
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use libs::admin::admin_router;
use libs::kafka::{KafkaManagerEvent, KafkaManagerPush};
use libs::settings::{Config, Settings};
use libs::shared::{SharedState, StateChat};
use libs::websocket::{handle_ws, send_to_ws};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut config = Config::new()?;
    let _ = config.listen().await.unwrap();
    dbg!(&config.data);

    let settings = Settings::new()?;

    //dbg!(&settings);

    let shared = SharedState::<UnboundedSender<ChatMessage>>::new();

    let event_mq = if settings.queue.enable {
        let mut push_mq: KafkaManagerPush<Envelope> = match settings.queue.push.kind.as_str() {
            "kafka" => KafkaManagerPush::new(settings.queue.push),
            _ => unreachable!(),
        };
        push_mq.run().await;
        let shared = shared.clone();
        send_to_ws(&push_mq, &shared).await;

        let mut event_mq: KafkaManagerEvent<ChatMessage> = match settings.queue.event.kind.as_str() {
            "kafka" => KafkaManagerEvent::new(settings.queue.event),
            _ => unreachable!(),
        };
        event_mq.run().await;
        Some(event_mq)
    } else {
        None
    };

    let webhooks = Arc::new(RwLock::new(settings.webhooks));
    //let greet = Arc::new(RwLock::new(settings.greet));
    let greet = settings.greet;

    let app = Router::new()
        .route(
            "/channel",
            get(
                |ws: WebSocketUpgrade, State(state): State<StateChat>| async move {
                    let event_tx = event_mq.as_ref().and_then(|m| m.get_tx());
                    ws.on_upgrade(|socket| handle_ws(socket, event_tx, state, webhooks, greet))
                },
            ),
        )
        .nest("/admin", admin_router())
        .with_state(shared);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
