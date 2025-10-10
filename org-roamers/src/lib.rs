//! org-roamers is in first and for all a server binary that can create the
//! same graph as [org-roam](https://github.com/org-roam/org-roam). It also has
//! routines for a clone of [org-roam-ui](https://github.com/org-roam/org-roam-ui)
//! latex previews and a lot more.
//!
//! <div class="warning">
//! org-roamers is split into a lib/bin architecture to enable customization of
//! the server. This crate most likely is only useful if some server feature
//! does not fit your org-roam usage. Otherwise just use the supplied server.
//! </div>
//!
//! See: the provided server implementation `org_roamers::bin::server::main.rs`.

mod cache;
mod latex;

mod client;
pub mod config;
mod search;
mod server;
mod sqlite;
mod transform;
mod watcher;

use sqlx::SqlitePool;

use dashmap::DashMap;
use std::sync::{atomic::AtomicU64, atomic::Ordering, Arc};
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::time::Instant;

use crate::cache::OrgCache;
use crate::client::message::WebSocketMessage;
use crate::config::Config;

pub struct ServerState {
    /// Read-only configuration
    pub config: Config,
    /// SQLite connection pool
    pub sqlite: SqlitePool,
    /// Org cache
    pub cache: OrgCache,
    /// WebSocket connections
    pub websocket_connections: DashMap<u64, UnboundedSender<WebSocketMessage>>,
    /// Atomic counter for connection IDs
    pub next_connection_id: AtomicU64,
}

impl ServerState {
    pub async fn new(conf: Config) -> anyhow::Result<ServerState> {
        let sqlite_con = sqlite::init_db(conf.strict).await?;

        let mut org_cache = OrgCache::new(conf.org_roamers_root.to_path_buf());

        org_cache.rebuild(&sqlite_con).await?;

        Ok(ServerState {
            sqlite: sqlite_con,
            cache: org_cache,
            config: conf,
            websocket_connections: DashMap::new(),
            next_connection_id: AtomicU64::new(1),
        })
    }

    /// Register a new WebSocket connection
    pub fn register_websocket_connection(
        &self,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
    ) -> u64 {
        let connection_id = self.next_connection_id.fetch_add(1, Ordering::SeqCst);
        self.websocket_connections.insert(connection_id, sender);
        connection_id
    }

    /// Unregister a WebSocket connection
    pub fn unregister_websocket_connection(&self, connection_id: u64) {
        self.websocket_connections.remove(&connection_id);
    }

    /// Send a message to all connected WebSocket clients
    pub fn broadcast_to_websockets(&self, message: WebSocketMessage) {
        let mut failed_connections = Vec::new();

        for entry in self.websocket_connections.iter() {
            let (connection_id, sender) = entry.pair();
            if sender.send(message.clone()).is_err() {
                failed_connections.push(*connection_id);
            }
        }

        // Remove failed connections
        for connection_id in failed_connections {
            self.websocket_connections.remove(&connection_id);
        }
    }
}

pub async fn start(state: ServerState) -> anyhow::Result<()> {
    let start = Instant::now();

    tracing::info!(
        "Using server configuration: {:?}",
        serde_json::to_string(&state.config)
    );

    let org_roam_db_path = state.cache.path().to_path_buf();
    let use_fs_watcher = state.config.fs_watcher;

    let host = &state.config.http_server_config.host;
    let port = &state.config.http_server_config.port;
    let url = format!("{}:{}", host, port);

    let app_state = Arc::new(state);

    if use_fs_watcher {
        let app_state_clone = app_state.clone();
        let watch_path = org_roam_db_path.clone();

        watcher::start_watcher_runtime(app_state_clone, watch_path)
            .await
            .unwrap();

        tracing::info!("File watcher enabled");
    }

    let app = server::build_server(app_state.clone()).await;

    tracing::info!("Server listening on {}", url);
    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();

    let end = Instant::now();
    tracing::info!("Startup took {}ms.", (end - start).as_millis());

    axum::serve(listener, app).tcp_nodelay(true).await.unwrap();

    Ok(())
}
