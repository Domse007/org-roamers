use axum::extract::ws::{Message, WebSocket};
use futures_util::{stream::SplitSink, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    search::{Feeder, SearchProviderList, SearchResultEntry},
    server::AppState,
};

/// WebSocket message types for 1:1 client communication
///
/// These messages are serialized as JSON and sent between the server
/// and individual WebSocket clients without any broadcasting.

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Search request from client
    #[serde(rename = "search_request")]
    SearchRequest { query: String, request_id: String },

    /// Search results response to client
    #[serde(rename = "search_response")]
    SearchResponse {
        request_id: String,
        results: SearchResultEntry,
    },

    /// Status update about file changes
    #[serde(rename = "status_update")]
    StatusUpdate { files_changed: usize },

    /// Node visited notification
    #[serde(rename = "node_visited")]
    NodeVisited {
        node_id: crate::server::types::RoamID,
    },

    /// Buffer modified notification
    #[serde(rename = "buffer_modified")]
    BufferModified,

    /// Keep-alive ping message
    #[serde(rename = "ping")]
    Ping,

    /// Response to ping message
    #[serde(rename = "pong")]
    Pong,
}

impl WebSocketMessage {
    pub async fn handle(
        &self,
        app_state: AppState,
        sender: &mut SplitSink<WebSocket, Message>,
        client_id: u64,
    ) {
        match self {
            Self::Ping => Self::handle_ping(client_id, sender).await,
            Self::Pong => Self::handle_pong(client_id).await,
            Self::SearchRequest { query, request_id } => {
                Self::handle_search(app_state, sender, client_id, query, request_id).await
            }
            unsupported => {
                tracing::error!("Unsupported request: {unsupported:?}");
            }
        }
    }

    async fn handle_ping(client_id: u64, sender: &mut SplitSink<WebSocket, Message>) {
        tracing::info!("Received ping from client {}, sending pong", client_id);
        if let Err(e) = sender
            .send(Message::Text(
                serde_json::to_string(&WebSocketMessage::Pong).unwrap(),
            ))
            .await
        {
            tracing::error!("Failed to send pong to client {}: {}", client_id, e);
        }
    }

    async fn handle_pong(client_id: u64) {
        tracing::info!("Received pong from client {}", client_id);
    }

    async fn handle_search(
        app_state: AppState,
        sender: &mut SplitSink<WebSocket, Message>,
        client_id: u64,
        query: &str,
        request_id: &str,
    ) {
        tracing::info!(
            "Processing search request from client {}: {}",
            client_id,
            query
        );

        let (mpsc_sender, mut mpsc_receiver) = mpsc::channel(100);
        let mut searcher_providers = SearchProviderList::new();

        searcher_providers.feed(app_state, mpsc_sender, Feeder::new(query.to_string())).await;

        while let Some(msg) = mpsc_receiver.blocking_recv() {
            let response = WebSocketMessage::SearchResponse {
                request_id: request_id.to_string(),
                results: msg,
            };
            if let Err(e) = sender
                .send(Message::Text(serde_json::to_string(&response).unwrap()))
                .await
            {
                tracing::error!(
                    "Failed to send search response to client {}: {}",
                    client_id,
                    e
                );
            }
        }
    }
}
