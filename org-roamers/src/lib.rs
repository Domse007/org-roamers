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

use server::types::RoamID;
use server::types::RoamLink;
use server::types::RoamNode;
use sqlx::SqlitePool;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::cache::OrgCache;
use crate::client::message::WebSocketMessage;
use crate::config::Config;

#[derive(Default, Debug)]
pub struct DynamicServerState {
    pub working_id: Option<(RoamID, Option<RoamID>)>,
    pub pending_reload: bool,
    pub updated_links: Vec<RoamLink>,
    pub updated_nodes: Vec<RoamNode>,
    /// Track files currently being processed to avoid watcher conflicts
    pub files_being_processed: HashSet<PathBuf>,
}

impl DynamicServerState {
    pub fn update_working_id(&mut self, new_id: RoamID) {
        match &mut self.working_id {
            Some(working_id) => {
                working_id.1 = Some(working_id.0.clone());
                working_id.0 = new_id;
            }
            None => self.working_id = Some((new_id, None)),
        }
    }

    pub fn get_working_id(&mut self) -> Option<&RoamID> {
        match &mut self.working_id {
            Some((ref current, ref mut last)) => match last {
                Some(last) if *last == *current => None,
                Some(last) => {
                    *last = current.clone();
                    Some(current)
                }
                None => {
                    *last = Some(current.clone());
                    Some(current)
                }
            },
            None => None,
        }
    }

    /// Mark a file as being processed to avoid watcher conflicts
    pub fn mark_file_processing(&mut self, file_path: PathBuf) {
        self.files_being_processed.insert(file_path);
    }

    /// Unmark a file as being processed
    pub fn unmark_file_processing(&mut self, file_path: &PathBuf) {
        self.files_being_processed.remove(file_path);
    }

    /// Check if a file is currently being processed
    pub fn is_file_being_processed(&self, file_path: &PathBuf) -> bool {
        self.files_being_processed.contains(file_path)
    }
}

pub struct ServerState {
    pub config: Config,
    pub sqlite: SqlitePool,
    pub cache: OrgCache,
    pub dynamic_state: DynamicServerState,
    pub websocket_connections: HashMap<u64, mpsc::UnboundedSender<WebSocketMessage>>,
    pub next_connection_id: u64,
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
            dynamic_state: DynamicServerState::default(),
            websocket_connections: HashMap::new(),
            next_connection_id: 1,
        })
    }

    /// Register a new WebSocket connection
    pub fn register_websocket_connection(
        &mut self,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
    ) -> u64 {
        let connection_id = self.next_connection_id;
        self.next_connection_id += 1;
        self.websocket_connections.insert(connection_id, sender);
        connection_id
    }

    /// Unregister a WebSocket connection
    pub fn unregister_websocket_connection(&mut self, connection_id: u64) {
        self.websocket_connections.remove(&connection_id);
    }

    /// Send a message to all connected WebSocket clients
    pub fn broadcast_to_websockets(&mut self, message: WebSocketMessage) {
        let mut failed_connections = Vec::new();

        for (connection_id, sender) in &self.websocket_connections {
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
    tracing::info!(
        "Using server configuration: {:?}",
        serde_json::to_string(&state.config)
    );

    let org_roam_db_path = state.cache.path().to_path_buf();
    let use_fs_watcher = state.config.fs_watcher;

    let host = &state.config.http_server_config.host;
    let port = &state.config.http_server_config.port;
    let url = format!("{}:{}", host, port);

    let app_state = Arc::new(Mutex::new(state));

    if use_fs_watcher {
        let app_state_clone = app_state.clone();
        let watch_path = org_roam_db_path.clone();

        watcher::start_watcher_runtime(app_state_clone, watch_path)
            .await
            .unwrap();

        tracing::info!("File watcher enabled with concurrency conflict resolution");
    }

    let app = server::build_server(app_state.clone()).await;

    tracing::info!("Server listening on {}", url);
    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();

    axum::serve(listener, app).tcp_nodelay(true).await.unwrap();

    Ok(())
}
