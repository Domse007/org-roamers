//! WebSocket integration for real-time org-roam graph updates
//!
//! This module provides WebSocket broadcasting functionality that integrates with the file system
//! watcher to provide real-time updates to connected clients when org files change.
//!
//! ## Message Types
//!
//! - `StatusUpdate`: Notifies clients about the currently visited node and pending changes
//! - `GraphUpdate`: Broadcasts detailed graph changes (new/updated/removed nodes and links)
//! - `NodeVisited`: Notifies when a user visits a specific node (from Emacs integration)
//! - `Ping`/`Pong`: Keep-alive messages for connection health
//!
//! ## Integration Flow
//!
//! 1. File watcher detects changes in org-roam directory
//! 2. Watcher processes changes and updates the database
//! 3. Watcher creates a detailed `GraphUpdate` with specific changes
//! 4. `WebSocketBroadcaster` sends the update to all connected clients
//! 5. Frontend receives updates and can incrementally update the UI
//!
//! ## Usage
//!
//! The broadcaster is created once in `ServerState` and shared across the application:
//! ```rust,ignore
//! let broadcaster = Arc::new(WebSocketBroadcaster::new());
//!
//! // Broadcast a graph update when files change
//! broadcaster.broadcast_graph_update(
//!     new_nodes,
//!     updated_nodes,
//!     new_links,
//!     removed_nodes,
//!     removed_links,
//! ).await;
//! ```

use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::Duration;
use tracing::{error, info, warn};

use crate::server::types::{RoamID, RoamLink, RoamNode};

pub type ClientId = u64;

/// WebSocket message types for real-time org-roam updates
///
/// These messages are serialized as JSON and sent to connected WebSocket clients
/// to provide real-time updates about graph changes, node visits, and system status.

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Search request from client
    #[serde(rename = "search_request")]
    SearchRequest { query: String, request_id: String },
    /// Search results response
    #[serde(rename = "search_response")]
    SearchResponse {
        request_id: String,
        results: Vec<crate::server::types::SearchResponseElement>,
    },
    /// Status update about the current state of the application
    ///
    /// Sent when:
    /// - User visits a node (from Emacs integration)
    /// - Changes are pending (file modifications detected)
    /// - Periodic status updates
    #[serde(rename = "status_update")]
    StatusUpdate {
        /// Currently visited node (if any)
        visited_node: Option<RoamID>,
        /// Whether there are pending changes being processed
        pending_changes: bool,
        /// Nodes that have been updated (for incremental UI updates)
        updated_nodes: Vec<RoamNode>,
        /// Links that have been updated (for incremental UI updates)
        updated_links: Vec<RoamLink>,
    },
    /// Comprehensive graph update with detailed change information
    ///
    /// Sent by the file watcher when org files change. Contains specific
    /// information about what was added, updated, or removed to enable
    /// efficient incremental updates on the frontend.
    #[serde(rename = "graph_update")]
    GraphUpdate {
        /// Newly created nodes
        new_nodes: Vec<RoamNode>,
        /// Existing nodes that have been modified (title changes, etc.)
        updated_nodes: Vec<RoamNode>,
        /// Newly created links between nodes
        new_links: Vec<RoamLink>,
        /// IDs of nodes that have been removed
        removed_nodes: Vec<RoamID>,
        /// Links that have been removed
        removed_links: Vec<RoamLink>,
    },
    /// Notification that a specific node was visited
    ///
    /// Sent when a user opens a buffer in Emacs, allowing other
    /// connected clients to see what node is currently being viewed.
    #[serde(rename = "node_visited")]
    NodeVisited { node_id: RoamID },
    /// Keep-alive ping message
    #[serde(rename = "ping")]
    Ping,
    /// Response to ping message
    #[serde(rename = "pong")]
    Pong,
}

/// WebSocket broadcaster for real-time org-roam updates
///
/// This struct manages WebSocket connections and broadcasts messages to all connected clients.
/// It integrates with the file system watcher to provide real-time updates when org files change.
///
/// ## Features
///
/// - Automatic client connection management
/// - Message broadcasting to all connected clients
/// - Client cleanup for disconnected sessions
/// - Connection health monitoring with ping/pong
///
/// ## Thread Safety
///
/// The broadcaster is designed to be shared across threads using `Arc<WebSocketBroadcaster>`.
/// All operations are thread-safe and can be called from different parts of the application.
pub struct WebSocketBroadcaster {
    sender: broadcast::Sender<WebSocketMessage>,
    clients: Arc<RwLock<HashMap<ClientId, broadcast::Receiver<WebSocketMessage>>>>,
    next_client_id: Arc<RwLock<ClientId>>,
}

impl WebSocketBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);

        Self {
            sender,
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn add_client(&self) -> (ClientId, broadcast::Receiver<WebSocketMessage>) {
        let mut next_id = self.next_client_id.write().await;
        let client_id = *next_id;
        *next_id += 1;

        let receiver = self.sender.subscribe();
        let mut clients = self.clients.write().await;
        clients.insert(client_id, self.sender.subscribe());

        info!("Added WebSocket client {}", client_id);
        (client_id, receiver)
    }

    pub async fn remove_client(&self, client_id: ClientId) {
        let mut clients = self.clients.write().await;
        clients.remove(&client_id);
        info!("Removed WebSocket client {}", client_id);
    }

    pub async fn broadcast(&self, message: WebSocketMessage) {
        let message_type = match &message {
            WebSocketMessage::SearchRequest { .. } => "search_request",
            WebSocketMessage::SearchResponse { .. } => "search_response",
            WebSocketMessage::StatusUpdate { .. } => "status_update",
            WebSocketMessage::GraphUpdate { .. } => "graph_update",
            WebSocketMessage::NodeVisited { .. } => "node_visited",
            WebSocketMessage::Ping => "ping",
            WebSocketMessage::Pong => "pong",
        };

        let client_count = self.sender.receiver_count();
        info!(
            "Broadcasting {} message to {} clients",
            message_type, client_count
        );

        if let Err(e) = self.sender.send(message.clone()) {
            warn!("Failed to broadcast {} message: {}", message_type, e);
        } else {
            info!("Successfully queued {} message for broadcast", message_type);
        }

        // Clean up disconnected clients
        let mut clients = self.clients.write().await;
        let mut disconnected_clients = Vec::new();

        for (client_id, receiver) in clients.iter() {
            if receiver.is_closed() {
                disconnected_clients.push(*client_id);
            }
        }

        if !disconnected_clients.is_empty() {
            info!(
                "Cleaning up {} disconnected clients",
                disconnected_clients.len()
            );
            for client_id in disconnected_clients {
                clients.remove(&client_id);
                info!("Cleaned up disconnected WebSocket client {}", client_id);
            }
        }
    }

    pub async fn broadcast_status_update(
        &self,
        visited_node: Option<RoamID>,
        pending_changes: bool,
        updated_nodes: Vec<RoamNode>,
        updated_links: Vec<RoamLink>,
    ) {
        let message = WebSocketMessage::StatusUpdate {
            visited_node,
            pending_changes,
            updated_nodes,
            updated_links,
        };
        self.broadcast(message).await;
    }

    /// Broadcast a comprehensive graph update with detailed change information
    ///
    /// This method is typically called by the file system watcher when it detects
    /// changes to org files. It sends detailed information about what changed,
    /// allowing clients to perform efficient incremental updates.
    ///
    /// # Arguments
    ///
    /// * `new_nodes` - Nodes that were newly created
    /// * `updated_nodes` - Existing nodes that were modified
    /// * `new_links` - Links that were newly created
    /// * `removed_nodes` - Node IDs that were removed
    /// * `removed_links` - Links that were removed
    pub async fn broadcast_graph_update(
        &self,
        new_nodes: Vec<RoamNode>,
        updated_nodes: Vec<RoamNode>,
        new_links: Vec<RoamLink>,
        removed_nodes: Vec<RoamID>,
        removed_links: Vec<RoamLink>,
    ) {
        let message = WebSocketMessage::GraphUpdate {
            new_nodes,
            updated_nodes,
            new_links,
            removed_nodes,
            removed_links,
        };
        self.broadcast(message).await;
    }

    pub async fn broadcast_node_visited(&self, node_id: RoamID) {
        let message = WebSocketMessage::NodeVisited { node_id };
        self.broadcast(message).await;
    }

    pub fn client_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Handle a new WebSocket connection
///
/// This function manages the lifecycle of a WebSocket connection, including:
/// - Setting up bidirectional message handling
/// - Periodic ping/pong for connection health
/// - Proper cleanup when the connection closes
/// - Message broadcasting from the server to client
/// - Basic message handling from client to server
/// - Search request processing
///
/// # Arguments
///
/// * `socket` - The WebSocket connection
/// * `broadcaster` - Shared broadcaster instance for message distribution
pub async fn handle_websocket(
    socket: WebSocket,
    broadcaster: Arc<WebSocketBroadcaster>,
    app_state: Arc<std::sync::Mutex<(crate::ServerState, Arc<std::sync::Mutex<bool>>)>>,
) {
    let (client_id, mut receiver) = broadcaster.add_client().await;
    let (sender, mut ws_receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    info!("WebSocket connection established for client {}", client_id);

    // Spawn task to handle incoming messages from client
    let broadcaster_clone = broadcaster.clone();
    let sender_clone = sender.clone();
    let incoming_task = tokio::spawn(async move {
        let sender = sender_clone;
        info!("Starting incoming message handler for client {}", client_id);
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    info!(
                        "Received text message from client {}: {}",
                        client_id,
                        text.chars().take(100).collect::<String>()
                    );
                    match serde_json::from_str::<WebSocketMessage>(&text) {
                        Ok(WebSocketMessage::Ping) => {
                            info!(
                                "Received ping from client {}, responding with pong",
                                client_id
                            );
                            broadcaster_clone.broadcast(WebSocketMessage::Pong).await;
                        }
                        Ok(WebSocketMessage::Pong) => {
                            info!("Received pong from client {}", client_id);
                        }
                        Ok(WebSocketMessage::SearchRequest { query, request_id }) => {
                            info!(
                                "Received search request from client {}: {}",
                                client_id, query
                            );

                            // Process search in a separate task to avoid blocking
                            let app_state_clone = app_state.clone();
                            let broadcaster_clone = broadcaster_clone.clone();
                            tokio::spawn(async move {
                                let results = {
                                    let mut state_guard = app_state_clone.lock().unwrap();
                                    let (ref mut server_state, _) = *state_guard;
                                    crate::server::services::search_service::search(
                                        server_state,
                                        query,
                                    )
                                };

                                // Extract results from the first provider (usually sqlite)
                                let search_results = results
                                    .providers
                                    .first()
                                    .map(|provider| provider.results.clone())
                                    .unwrap_or_default();

                                let response = WebSocketMessage::SearchResponse {
                                    request_id,
                                    results: search_results,
                                };

                                broadcaster_clone.broadcast(response).await;
                            });
                        }
                        Ok(other) => {
                            info!(
                                "Received other message from client {}: {:?}",
                                client_id, other
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to parse WebSocket message from client {}: {} - Raw message: {}",
                                client_id, e, text.chars().take(200).collect::<String>()
                            );
                        }
                    }
                }
                Ok(Message::Close(close_frame)) => {
                    if let Some(frame) = close_frame {
                        info!(
                            "WebSocket client {} closed connection with code: {} reason: {}",
                            client_id, frame.code, frame.reason
                        );
                    } else {
                        info!(
                            "WebSocket client {} closed connection (no close frame)",
                            client_id
                        );
                    }
                    break;
                }
                Ok(Message::Pong(_)) => {
                    info!("Client {} responded to our ping", client_id);
                }
                Ok(Message::Ping(data)) => {
                    info!("Received ping from client {}, sending pong", client_id);
                    let result = {
                        let mut guard = sender.lock().await;
                        guard.send(Message::Pong(data)).await
                    };
                    if let Err(e) = result {
                        error!("Failed to send pong to client {}: {}", client_id, e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    warn!(
                        "Received unexpected binary message from client {} (length: {})",
                        client_id,
                        data.len()
                    );
                }
                Err(e) => {
                    error!("WebSocket error for client {}: {}", client_id, e);
                    break;
                }
            }
        }
        info!(
            "Incoming message handler for client {} terminated",
            client_id
        );
    });

    // Spawn task to handle outgoing messages to client
    let outgoing_task = tokio::spawn(async move {
        info!("Starting outgoing message handler for client {}", client_id);

        // Send periodic pings to keep connection alive
        let mut ping_interval = tokio::time::interval(Duration::from_secs(30));
        ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Send initial ping to test connection
        let initial_ping = serde_json::to_string(&WebSocketMessage::Ping).unwrap();
        let result = {
            let mut guard = sender.lock().await;
            guard.send(Message::Text(initial_ping)).await
        };
        if let Err(e) = result {
            error!("Failed to send initial ping to client {}: {}", client_id, e);
            return;
        }
        info!("Sent initial ping to client {}", client_id);

        loop {
            tokio::select! {
                // Handle broadcast messages
                msg = receiver.recv() => {
                    match msg {
                        Ok(message) => {
                            let message_type = match &message {
                                WebSocketMessage::SearchRequest { .. } => "search_request",
                                WebSocketMessage::SearchResponse { .. } => "search_response",
                                WebSocketMessage::StatusUpdate { .. } => "status_update",
                                WebSocketMessage::GraphUpdate { .. } => "graph_update",
                                WebSocketMessage::NodeVisited { .. } => "node_visited",
                                WebSocketMessage::Ping => "ping",
                                WebSocketMessage::Pong => "pong",
                            };
                            info!("Sending {} message to client {}", message_type, client_id);

                            let json = match serde_json::to_string(&message) {
                                Ok(json) => json,
                                Err(e) => {
                                    error!("Failed to serialize {} message for client {}: {}", message_type, client_id, e);
                                    continue;
                                }
                            };

                            let result = {
                                let mut guard = sender.lock().await;
                                guard.send(Message::Text(json)).await
                            };
                            if let Err(e) = result {
                                error!("Failed to send {} message to client {}: {}", message_type, client_id, e);
                                break;
                            }
                            info!("Successfully sent {} message to client {}", message_type, client_id);
                        }
                        Err(broadcast::error::RecvError::Lagged(count)) => {
                            warn!("Client {} lagged behind, {} messages were dropped", client_id, count);
                            // Try to continue serving the client
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("Broadcast channel closed for client {}", client_id);
                            break;
                        }
                    }
                }

                // Send periodic pings
                _ = ping_interval.tick() => {
                    info!("Sending periodic ping to client {}", client_id);
                    let ping_msg = serde_json::to_string(&WebSocketMessage::Ping).unwrap();
                    let result = {
                        let mut guard = sender.lock().await;
                        guard.send(Message::Text(ping_msg)).await
                    };
                    if let Err(e) = result {
                        error!("Failed to send periodic ping to client {}: {}", client_id, e);
                        break;
                    }
                    info!("Sent periodic ping to client {}", client_id);
                }
            }
        }
        info!(
            "Outgoing message handler for client {} terminated",
            client_id
        );
    });

    // Wait for either task to complete
    tokio::select! {
        result = incoming_task => {
            info!("Incoming task for client {} completed: {:?}", client_id, result);
        },
        result = outgoing_task => {
            info!("Outgoing task for client {} completed: {:?}", client_id, result);
        },
    }

    // Clean up
    info!("Cleaning up WebSocket connection for client {}", client_id);
    broadcaster.remove_client(client_id).await;
    info!(
        "WebSocket connection cleanup completed for client {}",
        client_id
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::types::{RoamID, RoamLink, RoamNode, RoamTitle};

    #[test]
    fn test_websocket_message_serialization() {
        let graph_update = WebSocketMessage::GraphUpdate {
            new_nodes: vec![RoamNode {
                title: RoamTitle::from("New Node"),
                id: RoamID::from("new-id"),
                parent: RoamID::from(""),
                num_links: 1,
            }],
            updated_nodes: vec![RoamNode {
                title: RoamTitle::from("Updated Node"),
                id: RoamID::from("updated-id"),
                parent: RoamID::from(""),
                num_links: 2,
            }],
            new_links: vec![RoamLink {
                from: RoamID::from("from-id"),
                to: RoamID::from("to-id"),
            }],
            removed_nodes: vec![RoamID::from("removed-id")],
            removed_links: vec![RoamLink {
                from: RoamID::from("old-from-id"),
                to: RoamID::from("old-to-id"),
            }],
        };

        let serialized = serde_json::to_string(&graph_update).expect("Failed to serialize");

        // Verify it contains the expected structure
        assert!(serialized.contains("\"type\":\"graph_update\""));
        assert!(serialized.contains("\"new_nodes\""));
        assert!(serialized.contains("\"updated_nodes\""));
        assert!(serialized.contains("\"new_links\""));
        assert!(serialized.contains("\"removed_nodes\""));
        assert!(serialized.contains("\"removed_links\""));

        // Verify deserialization works
        let _deserialized: WebSocketMessage =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
    }

    #[test]
    fn test_status_update_message() {
        let status_update = WebSocketMessage::StatusUpdate {
            visited_node: Some(RoamID::from("visited-id")),
            pending_changes: true,
            updated_nodes: vec![],
            updated_links: vec![],
        };

        let serialized = serde_json::to_string(&status_update).expect("Failed to serialize");

        assert!(serialized.contains("\"type\":\"status_update\""));
        assert!(serialized.contains("\"visited_node\""));
        assert!(serialized.contains("\"pending_changes\":true"));

        let _deserialized: WebSocketMessage =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
    }
}
