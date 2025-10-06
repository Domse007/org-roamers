use std::{fs, path::PathBuf};

use org_roamers::{ServerState, config::Config, server::ServerRuntime};

use crate::OrgRoamersGUI;

#[cfg(not(target_os = "windows"))]
pub fn config_path() -> PathBuf {
    PathBuf::from("/etc/org-roamers/")
}

#[cfg(target_os = "windows")]
pub fn config_path() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap()
}

fn html_path() -> PathBuf {
    let mut path = config_path();
    path.push("html_settings.json");
    path
}

fn server_conf_path() -> PathBuf {
    let mut path = config_path();
    path.push("server_conf.json");
    if !path.exists() {
        PathBuf::from("./server_conf.json")
    } else {
        path
    }
}

pub fn start_server(ctx: &OrgRoamersGUI) -> anyhow::Result<ServerRuntime> {
    let url = ctx.url()?;

    let mut server_configuration = match fs::read_to_string(server_conf_path()) {
        Ok(content) => serde_json::from_str(content.as_str()).unwrap(),
        Err(err) => {
            tracing::error!("Failed to load config: {err}");
            Config::default()
        }
    };

    server_configuration.fs_watcher = ctx.settings.fs_watcher;

    let mut state = ServerState::new(server_configuration)?;

    Ok(org_roamers::server::start_server(url, state).unwrap())
}
