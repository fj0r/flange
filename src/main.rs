use axum::{Router, routing::get};
use libs::message::{ChatMessage, Envelope, MessageQueue};

use tracing::info;
use tracing_subscriber;
mod libs;
use anyhow::{Ok, Result};
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use libs::admin::admin_router;
use libs::websocket::handle_socket;
use libs::kafka::KafkaManager;
use libs::settings::Settings;
use libs::shared::{StateChat, SharedState};
use tokio::sync::mpsc::UnboundedSender;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let settings = Settings::new()?;

    dbg!(&settings);

    let shared = SharedState::<UnboundedSender<ChatMessage>>::new();

    let mq = if settings.queue.enable {
        let mut mq: KafkaManager<ChatMessage, Envelope> = KafkaManager::new(
            settings.queue.push.clone(),
            settings.queue.event.clone(),
        );
        mq.run().await;
        let mqrx = mq.get_rx();
        let shared = shared.clone();
        tokio::spawn(async move {
            if let Some(rx) = mqrx {
                let mut rx = rx.lock().await;
                while let Some(x) = rx.recv().await  {
                    if !x.receiver.is_empty() {
                        let s = shared.read().await;
                        for r in x.receiver {
                            if s.sender.contains_key(&r) {
                                if let Some(s) = s.sender.get(&r) {
                                    let _ = s.send(x.message.clone());
                                }
                            }
                        }
                    }
                }
            }
        });
        Some(mq)
    } else {
        None
    };

    let mqtx = mq.as_ref().and_then(|m| m.get_tx());
    let app = Router::new()
        .route(
            "/channel",
            get(
                |ws: WebSocketUpgrade, State(state): State<StateChat>| async move {
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
