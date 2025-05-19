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

use sqlite::SqliteConnection;
use transform::export::HtmlExportSettings;

use std::path::Path;

pub struct ServerState {
    pub sqlite: SqliteConnection,
    pub html_export_settings: HtmlExportSettings,
}

pub fn prepare_internal<P: AsRef<Path>>(
    html_export_settings_path: P,
) -> Result<ServerState, Box<dyn std::error::Error>> {
    let sqlite_con = match SqliteConnection::init() {
        Ok(con) => con,
        Err(e) => {
            return Err(format!("ERROR: could not initialize the sqlite connection: {e}").into())
        }
    };

    Ok(ServerState {
        sqlite: sqlite_con,
        html_export_settings: HtmlExportSettings::new(html_export_settings_path)
            .unwrap_or_default(),
    })
}
