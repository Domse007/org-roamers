use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use futures_util::{stream::SplitSink, SinkExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    client::WebSocketClient,
    search::{Feeder, SearchProviderList, SearchResultEntry},
    ServerState,
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
    /// Request for search configuration.
    SearchConfigurationRequest,
    /// Mapping between provider_id and name of provider.
    SearchConfigurationResponse { config: Vec<(usize, String)> },
    /// Stop the current search operation.
    SearchStop,

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
        app_state: Arc<ServerState>,
        sender: &mut SplitSink<WebSocket, Message>,
        client: &mut WebSocketClient,
    ) {
        match self {
            Self::Ping => Self::handle_ping(client.client_id, sender).await,
            Self::Pong => Self::handle_pong(client.client_id).await,
            Self::SearchConfigurationRequest => {
                let (mpsc_sender, mpsc_receiver) = mpsc::channel(10000);
                let provider_list = SearchProviderList::new(mpsc_sender);
                let config = provider_list.config();
                client.search = Some((provider_list, mpsc_receiver));
                if let Err(err) = sender
                    .send(Message::Text(
                        serde_json::to_string(&Self::SearchConfigurationResponse { config })
                            .unwrap(),
                    ))
                    .await
                {
                    tracing::error!("Couln't send conf resp: {err}");
                };
            }
            Self::SearchRequest { query, request_id } => {
                Self::handle_search(app_state, sender, client, query, request_id).await
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
        app_state: Arc<ServerState>,
        _sender: &mut SplitSink<WebSocket, Message>,
        client: &mut WebSocketClient,
        query: &str,
        request_id: &str,
    ) {
        let start = std::time::Instant::now();
        tracing::info!(
            "Processing search request from client {}: {}",
            client.client_id,
            query
        );

        let Some((searcher_providers, mpsc_receiver)) = &mut client.search else {
            tracing::error!("Search started without initializing.");
            return;
        };

        // Cancel any ongoing searches before starting a new one
        searcher_providers.cancel();

        // Drain any pending results from the previous search
        while mpsc_receiver.try_recv().is_ok() {
            // Discard old results
        }

        // Store the current request_id so we can use it when sending results
        client.current_request_id = Some(request_id.to_string());

        tracing::info!("Starting search providers (took {:?})", start.elapsed());

        // Start the search (non-blocking)
        searcher_providers
            .feed(app_state, Feeder::new(query.to_string()))
            .await;

        tracing::info!("Search providers started (took {:?})", start.elapsed());

        // Don't block here - results will be received in the main select! loop
        // The mpsc_receiver is polled in the WebSocketClient::handle_connection method
    }
}
