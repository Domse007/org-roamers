use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
pub const CONFIG_PATH: &'static str = "/etc/org-roamers/";

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub sqlite_path: Option<String>,
    pub html_export_path: PathBuf,
    pub roam_path: String,
    pub ip_addr: String,
    pub port: u16,
}

impl Configuration {
    pub fn get_url(&self, protocol: bool) -> String {
        if protocol {
            format!("http://{}:{}", self.ip_addr, self.port)
        } else {
            format!("{}:{}", self.ip_addr, self.port)
        }
    }
}
