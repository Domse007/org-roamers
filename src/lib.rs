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

pub mod api;
pub mod error;
mod latex;
pub mod parser;
mod perf;
pub mod search;
pub mod server;
pub mod sqlite;
pub mod transform;
pub mod watcher;

use api::types::RoamID;
use serde::Deserialize;
use serde::Serialize;
use sqlite::SqliteConnection;
use transform::export::HtmlExportSettings;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticServerConfiguration {
    /// Root path to the website files. e.g. .js / .html / .css
    pub root: String,
}

#[derive(Default)]
pub(crate) struct DynamicServerState {
    pub working_id: Option<(RoamID, Option<RoamID>)>,
    pub pending_reload: bool,
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
                    Some(&current)
                }
                None => Some(&current),
            },
            None => None,
        }
    }
}

pub struct ServerState {
    pub sqlite: SqliteConnection,
    pub html_export_settings: HtmlExportSettings,
    pub org_roam_db_path: PathBuf,
    pub static_conf: StaticServerConfiguration,
    dynamic_state: DynamicServerState,
}

impl ServerState {
    pub fn new<P: AsRef<Path>>(
        html_export_settings_path: P,
        org_roam_db_path: P,
        static_conf: StaticServerConfiguration,
    ) -> Result<ServerState, Box<dyn std::error::Error>> {
        let sqlite_con = match SqliteConnection::init() {
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
        })
    }
}
