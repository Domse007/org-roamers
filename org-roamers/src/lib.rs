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

mod diff;
pub mod error;
pub mod file;
mod latex;

pub mod search;
pub mod server;
pub mod sqlite;
pub mod transform;
pub mod watcher;
pub mod websocket;

use serde::Deserialize;
use serde::Serialize;
use server::types::RoamID;
use server::types::RoamLink;
use server::types::RoamNode;
use sqlite::SqliteConnection;
use transform::export::HtmlExportSettings;
use websocket::WebSocketBroadcaster;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::latex::LatexConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticServerConfiguration {
    /// Root path to the website files. e.g. .js / .html / .css
    pub root: String,
    /// Use stricter policy like foreign_keys = ON.
    pub strict: bool,
    /// Use the filesystem watcher
    pub fs_watcher: bool,
    /// LaTeX settings for rendering fragments
    pub latex_config: LatexConfig,
}

impl Default for StaticServerConfiguration {
    fn default() -> Self {
        Self {
            root: "./web/dist/".to_string(),
            strict: false,
            fs_watcher: false,
            latex_config: LatexConfig::default(),
        }
    }
}

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
    pub sqlite: SqliteConnection,
    pub html_export_settings: HtmlExportSettings,
    pub org_roam_db_path: PathBuf,
    pub static_conf: StaticServerConfiguration,
    pub dynamic_state: DynamicServerState,
    pub websocket_broadcaster: Arc<WebSocketBroadcaster>,
}

impl ServerState {
    pub fn new<P: AsRef<Path>>(
        html_export_settings_path: P,
        org_roam_db_path: P,
        static_conf: StaticServerConfiguration,
    ) -> Result<ServerState, Box<dyn std::error::Error>> {
        let sqlite_con = match SqliteConnection::init(static_conf.strict) {
            Ok(con) => con,
            Err(e) => {
                return Err(
                    format!("ERROR: could not initialize the sqlite connection: {e}").into(),
                )
            }
        };

        Ok(ServerState {
            sqlite: sqlite_con,
            html_export_settings: HtmlExportSettings::new(html_export_settings_path)
                .unwrap_or_default(),
            org_roam_db_path: org_roam_db_path.as_ref().to_path_buf(),
            static_conf,
            dynamic_state: DynamicServerState::default(),
            websocket_broadcaster: Arc::new(WebSocketBroadcaster::new()),
        })
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
            sqlite,
            html_export_settings: HtmlExportSettings::default(),
            org_roam_db_path: temp_dir.clone(),
            static_conf: StaticServerConfiguration::default(),
            dynamic_state: DynamicServerState::default(),
            websocket_broadcaster: Arc::new(WebSocketBroadcaster::new()),
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
