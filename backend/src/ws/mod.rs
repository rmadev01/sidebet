use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::models::User;

/// Shared broadcast channel for real-time events
pub type WsBroadcast = Arc<broadcast::Sender<WsEvent>>;

#[derive(Clone, Debug, serde::Serialize)]
pub struct WsEvent {
    pub target_user_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: serde_json::Value,
}

pub fn create_broadcast() -> WsBroadcast {
    let (tx, _) = broadcast::channel(256);
    Arc::new(tx)
}

/// WS /api/ws — authenticated WebSocket connection
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(user): Extension<User>,
    Extension(broadcast): Extension<WsBroadcast>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, user, broadcast))
}

async fn handle_socket(socket: WebSocket, user: User, broadcast: WsBroadcast) {
    let (mut sender, mut receiver) = socket.split();
    let user_id = user.id.to_string();
    let mut rx = broadcast.subscribe();

    // Spawn task to forward broadcast events to this user
    let uid = user_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if event.target_user_id == uid {
                let msg = serde_json::to_string(&event).unwrap_or_default();
                if sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Read incoming messages (ping/pong, or bet watch subscriptions)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Text(text) => {
                    tracing::debug!("WS recv from {}: {}", user_id, text);
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
