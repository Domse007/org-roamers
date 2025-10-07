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
pub mod error;
mod latex;

pub mod search;
pub mod server;
pub mod sqlite;
pub mod transform;
pub mod watcher;
mod client;
pub mod config;

use server::types::RoamID;
use server::types::RoamLink;
use server::types::RoamNode;
use sqlite::SqliteConnection;

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
    pub sqlite: Mutex<SqliteConnection>,
    pub cache: OrgCache,
    pub dynamic_state: DynamicServerState,
    pub websocket_connections: HashMap<u64, mpsc::UnboundedSender<WebSocketMessage>>,
    pub next_connection_id: u64,
}

impl ServerState {
    pub fn new(conf: Config) -> anyhow::Result<ServerState> {
        let mut sqlite_con = match SqliteConnection::init(conf.strict) {
            Ok(con) => con,
            Err(e) => {
                anyhow::bail!("ERROR: could not initialize the sqlite connection: {e}");
            }
        };

        let mut org_cache = OrgCache::new(conf.org_roamers_root.to_path_buf());

        org_cache.rebuild(sqlite_con.connection())?;

        Ok(ServerState {
            sqlite: Mutex::new(sqlite_con),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_processing_guard() {
        // Create a mock server state
        let temp_dir = std::env::temp_dir();
        let sqlite = SqliteConnection::init(false).unwrap();
        let server_state = ServerState {
            config: Config::default(),
            sqlite: Mutex::new(sqlite),
            cache: OrgCache::new(temp_dir.clone()),
            dynamic_state: DynamicServerState::default(),
            websocket_connections: HashMap::new(),
            next_connection_id: 1,
        };

        let app_state = Arc::new(Mutex::new((server_state, Arc::new(Mutex::new(false)))));
        let test_file = temp_dir.join("test.org");

        // Test that guard properly tracks files
        {
            let _guard = FileProcessingGuard::new(app_state.clone(), test_file.clone()).unwrap();

            // Check that file is marked as being processed
            let state = app_state.lock().unwrap();
            assert!(state.0.dynamic_state.is_file_being_processed(&test_file));
            drop(state); // Release lock before guard is dropped
        } // Guard is dropped here

        // Check that file is unmarked after guard is dropped
        let state = app_state.lock().unwrap();
        assert!(!state.0.dynamic_state.is_file_being_processed(&test_file));
    }
}

/// RAII guard to automatically track file processing state
/// When dropped, it will automatically unmark the file as being processed
pub struct FileProcessingGuard {
    app_state: Arc<Mutex<(ServerState, Arc<Mutex<bool>>)>>,
    file_path: PathBuf,
}

impl FileProcessingGuard {
    /// Create a new guard and mark the file as being processed
    pub fn new(
        app_state: Arc<Mutex<(ServerState, Arc<Mutex<bool>>)>>,
        file_path: PathBuf,
    ) -> anyhow::Result<Self> {
        // Mark the file as being processed
        {
            let mut state = app_state
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire lock: {}", e))?;
            state
                .0
                .dynamic_state
                .mark_file_processing(file_path.clone());
        } // Lock is dropped here

        Ok(FileProcessingGuard {
            app_state,
            file_path,
        })
    }
}

impl Drop for FileProcessingGuard {
    fn drop(&mut self) {
        if let Ok(mut state) = self.app_state.lock() {
            state
                .0
                .dynamic_state
                .unmark_file_processing(&self.file_path);
        }
    }
}
