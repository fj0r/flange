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
use libs::settings::Settings;
use libs::shared::{SharedState, StateChat};
use libs::websocket::{handle_socket, notify};
use tokio::sync::mpsc::UnboundedSender;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    dbg!(&settings);

    let shared = SharedState::<UnboundedSender<ChatMessage>>::new();

    let event_mq = if settings.queue.enable {
        let mut push_mq: KafkaManagerPush<Envelope> =
            KafkaManagerPush::new(settings.queue.push.clone());
        push_mq.run().await;
        let shared = shared.clone();
        notify(&push_mq, &shared).await;

        let mut event_mq: KafkaManagerEvent<ChatMessage> =
            KafkaManagerEvent::new(settings.queue.event.clone());
        event_mq.run().await;
        Some(event_mq)
    } else {
        None
    };

    let app = Router::new()
        .route(
            "/channel",
            get(
                |ws: WebSocketUpgrade, State(state): State<StateChat>| async move {
                    let mqtx = event_mq.as_ref().and_then(|m| m.get_tx());
                    ws.on_upgrade(|socket| handle_socket(socket, state, mqtx))
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
