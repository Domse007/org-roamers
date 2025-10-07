//! Simple 1:1 WebSocket client implementation
//!
//! This module provides a straightforward WebSocket client that handles
//! individual connections and registers with the server's connection registry.
//!
//! ## Features
//!
//! - Direct 1:1 WebSocket communication
//! - Connection registration with server state
//! - Search request/response handling
//! - Ping/pong keep-alive mechanism
//! - Simple message handling without broadcasting

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::Duration;
use tracing::{error, info, warn};

use crate::{client::message::WebSocketMessage, server::AppState};

pub mod message;

/// Simple WebSocket client that handles a single connection
pub struct WebSocketClient {
    socket: WebSocket,
    client_id: u64,
}

impl WebSocketClient {
    pub fn new(socket: WebSocket, client_id: u64) -> Self {
        Self { socket, client_id }
    }

    /// Handle the WebSocket connection lifecycle
    pub async fn handle_connection(self, app_state: AppState) {
        let (mut sender, mut receiver) = self.socket.split();
        let client_id = self.client_id;

        info!("WebSocket client {} connected", client_id);

        // Create a channel for receiving messages from the server
        let (server_tx, mut server_rx) = mpsc::unbounded_channel::<WebSocketMessage>();

        // Register this connection with the server state
        {
            let mut state_guard = app_state.lock().unwrap();
            state_guard.register_websocket_connection(server_tx);
        }

        // Set up ping interval for keep-alive
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Send initial ping
        if let Err(e) = sender
            .send(Message::Text(
                serde_json::to_string(&WebSocketMessage::Ping).unwrap(),
            ))
            .await
        {
            error!("Failed to send initial ping to client {}: {}", client_id, e);
            return;
        }

        loop {
            tokio::select! {
                // Handle incoming messages from client
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            match serde_json::from_str::<WebSocketMessage>(&text) {
                                Ok(msg) => msg.handle(app_state.clone(), &mut sender, client_id).await,
                                Err(e) => {
                                    warn!("Failed to parse message from client {}: {} - Raw: {}",
                                          client_id, e, text.chars().take(100).collect::<String>());
                                }
                            }
                        }
                        Some(Ok(Message::Close(close_frame))) => {
                            if let Some(frame) = close_frame {
                                info!("Client {} closed connection: {} - {}", client_id, frame.code, frame.reason);
                            } else {
                                info!("Client {} closed connection", client_id);
                            }
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error for client {}: {}", client_id, e);
                            break;
                        }
                        None => {
                            info!("WebSocket stream ended for client {}", client_id);
                            break;
                        }
                        _ => {}
                    }
                }

                // Handle messages from server (via watcher or other parts)
                msg = server_rx.recv() => {
                    match msg {
                        Some(message) => {
                            if let Err(e) = sender.send(Message::Text(
                                serde_json::to_string(&message).unwrap()
                            )).await {
                                error!("Failed to send server message to client {}: {}", client_id, e);
                                break;
                            }
                        }
                        None => {
                            info!("Server message channel closed for client {}", client_id);
                            break;
                        }
                    }
                }

                // Send periodic pings
                _ = ping_interval.tick() => {
                    if let Err(e) = sender.send(Message::Text(
                        serde_json::to_string(&WebSocketMessage::Ping).unwrap()
                    )).await {
                        error!("Failed to send ping to client {}: {}", client_id, e);
                        break;
                    }
                }
            }
        }

        // Unregister this connection when it closes
        {
            let mut state_guard = app_state.lock().unwrap();
            state_guard.unregister_websocket_connection(client_id);
        }

        info!("WebSocket client {} disconnected", client_id);
    }
}

/// Handle a new WebSocket connection with a simple 1:1 approach
pub async fn handle_websocket(socket: WebSocket, app_state: AppState) {
    // Use a simple counter for client IDs - in production you might want something more robust
    static CLIENT_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let client_id = CLIENT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let client = WebSocketClient::new(socket, client_id);
    client.handle_connection(app_state).await;
}
