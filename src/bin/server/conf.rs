use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[cfg(not(target_os = "windows"))]
pub fn config_path() -> PathBuf {
    PathBuf::from("/etc/org-roamers/")
}

#[cfg(target_os = "windows")]
pub fn config_path() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap()
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub html_export_path: PathBuf,
    pub roam_path: PathBuf,
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
