use std::{fs, path::PathBuf};

use org_roamers::{ServerState, config::Config};

use crate::OrgRoamersGUI;

#[cfg(not(target_os = "windows"))]
pub fn config_path() -> PathBuf {
    PathBuf::from("/etc/org-roamers/")
}

#[cfg(target_os = "windows")]
pub fn config_path() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap()
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

pub async fn start_server(ctx: &OrgRoamersGUI) -> anyhow::Result<()> {
    let mut server_configuration = match fs::read_to_string(server_conf_path()) {
        Ok(content) => serde_json::from_str(content.as_str()).unwrap(),
        Err(err) => {
            tracing::error!("Failed to load config: {err}");
            Config::default()
        }
    };

    server_configuration.fs_watcher = ctx.settings.fs_watcher;
    server_configuration.http_server_config.host = ctx.host().to_string();
    server_configuration.http_server_config.port = ctx.port()?;

    let state = ServerState::new(server_configuration)?;

    org_roamers::start(state).await.unwrap();

    Ok(())
}
