//! WebSocket handler for real-time communication

#![allow(dead_code)]

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::AppState;

/// WebSocket event channels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventChannel {
    Alerts,
    Metrics,
    AgentStatus,
    Compliance,
    AuditLog,
    SystemHealth,
}

/// Real-time event payload
#[derive(Debug, Clone, Serialize)]
pub struct RealtimeEvent {
    pub channel: EventChannel,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Client messages
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum ClientMessage {
    Subscribe { channels: Vec<EventChannel> },
    Unsubscribe { channels: Vec<EventChannel> },
    Ping,
}

/// Server messages
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Connected { session_id: String },
    Subscribed { channels: Vec<EventChannel> },
    Event(RealtimeEvent),
    Pong,
    Error { message: String },
}


/// Manages WebSocket connections and event broadcasting
#[derive(Clone)]
pub struct RealtimeHub {
    sender: broadcast::Sender<RealtimeEvent>,
    active_connections: Arc<RwLock<u32>>,
}

impl RealtimeHub {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            active_connections: Arc::new(RwLock::new(0)),
        }
    }

    /// Broadcast an event to all connected clients
    pub fn broadcast(&self, event: RealtimeEvent) {
        let _ = self.sender.send(event);
    }

    /// Publish an alert event
    pub fn publish_alert(&self, alert_id: Uuid, title: &str, severity: &str, source: &str) {
        self.broadcast(RealtimeEvent {
            channel: EventChannel::Alerts,
            event_type: "alert.created".to_string(),
            payload: serde_json::json!({
                "id": alert_id,
                "title": title,
                "severity": severity,
                "source": source,
            }),
            timestamp: chrono::Utc::now(),
        });
    }

    /// Publish agent status change
    pub fn publish_agent_status(&self, agent_id: Uuid, hostname: &str, status: &str) {
        self.broadcast(RealtimeEvent {
            channel: EventChannel::AgentStatus,
            event_type: "agent.status_changed".to_string(),
            payload: serde_json::json!({
                "agent_id": agent_id,
                "hostname": hostname,
                "status": status,
            }),
            timestamp: chrono::Utc::now(),
        });
    }

    /// Publish metrics update
    pub fn publish_metrics(&self, agent_id: Uuid, metrics: serde_json::Value) {
        self.broadcast(RealtimeEvent {
            channel: EventChannel::Metrics,
            event_type: "metrics.updated".to_string(),
            payload: serde_json::json!({
                "agent_id": agent_id,
                "metrics": metrics,
            }),
            timestamp: chrono::Utc::now(),
        });
    }

    pub async fn connection_count(&self) -> u32 {
        *self.active_connections.read().await
    }

    fn subscribe(&self) -> broadcast::Receiver<RealtimeEvent> {
        self.sender.subscribe()
    }

    async fn increment_connections(&self) {
        let mut count = self.active_connections.write().await;
        *count += 1;
    }

    async fn decrement_connections(&self) {
        let mut count = self.active_connections.write().await;
        *count = count.saturating_sub(1);
    }
}

/// WebSocket route
pub fn ws_routes() -> Router<AppState> {
    Router::new().route("/ws", get(ws_handler))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

async fn handle_ws_connection(socket: WebSocket, state: AppState) {
    let session_id = Uuid::now_v7().to_string();
    let hub = &state.realtime_hub;

    hub.increment_connections().await;
    info!(session_id = %session_id, "WebSocket client connected");

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut event_rx = hub.subscribe();
    let mut subscribed_channels: Vec<EventChannel> = Vec::new();

    // Send connected message
    let connected_msg = serde_json::to_string(&ServerMessage::Connected {
        session_id: session_id.clone(),
    }).unwrap();
    if ws_sender.send(Message::Text(connected_msg.into())).await.is_err() {
        hub.decrement_connections().await;
        return;
    }

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Subscribe { channels }) => {
                                for ch in &channels {
                                    if !subscribed_channels.contains(ch) {
                                        subscribed_channels.push(ch.clone());
                                    }
                                }
                                let resp = serde_json::to_string(
                                    &ServerMessage::Subscribed { channels: subscribed_channels.clone() }
                                ).unwrap();
                                if ws_sender.send(Message::Text(resp.into())).await.is_err() { break; }
                            }
                            Ok(ClientMessage::Unsubscribe { channels }) => {
                                subscribed_channels.retain(|c| !channels.contains(c));
                                let resp = serde_json::to_string(
                                    &ServerMessage::Subscribed { channels: subscribed_channels.clone() }
                                ).unwrap();
                                if ws_sender.send(Message::Text(resp.into())).await.is_err() { break; }
                            }
                            Ok(ClientMessage::Ping) => {
                                let resp = serde_json::to_string(&ServerMessage::Pong).unwrap();
                                if ws_sender.send(Message::Text(resp.into())).await.is_err() { break; }
                            }
                            Err(_) => {
                                let resp = serde_json::to_string(&ServerMessage::Error {
                                    message: "Invalid message format".to_string(),
                                }).unwrap();
                                let _ = ws_sender.send(Message::Text(resp.into())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => { warn!("WebSocket error: {e}"); break; }
                    _ => {}
                }
            }
            event = event_rx.recv() => {
                match event {
                    Ok(ev) if subscribed_channels.contains(&ev.channel) => {
                        let msg = serde_json::to_string(&ServerMessage::Event(ev)).unwrap();
                        if ws_sender.send(Message::Text(msg.into())).await.is_err() { break; }
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket client lagged by {n} messages");
                    }
                    Err(_) => break,
                }
            }
        }
    }

    hub.decrement_connections().await;
    info!(session_id = %session_id, "WebSocket client disconnected");
}

