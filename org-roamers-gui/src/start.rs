use std::{fs, path::PathBuf, thread};

use org_roamers::{ServerState, config::Config};
use tokio::runtime::Runtime;

use crate::{OrgRoamersGUI, settings::Settings};

pub struct ServerHandle {
    handle: Option<thread::JoinHandle<anyhow::Result<()>>>,
}

impl ServerHandle {
    pub fn abort(&mut self) {
        if let Some(handle) = self.handle.take() {
            // We can't gracefully abort a thread, so we'll need to implement
            // proper shutdown signaling in the future
            drop(handle);
        }
    }
}

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
    path.push("conf.json");
    if !path.exists() {
        PathBuf::from("./conf.json")
    } else {
        path
    }
}

pub fn start(ctx: &OrgRoamersGUI) -> ServerHandle {
    let settings = ctx.settings.clone();

    let handle = thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move { start_server(settings).await })
    });

    ServerHandle {
        handle: Some(handle),
    }
}

pub async fn start_server(ctx: Settings) -> anyhow::Result<()> {
    let mut server_configuration = match fs::read_to_string(server_conf_path()) {
        Ok(content) => serde_json::from_str(content.as_str()).unwrap(),
        Err(err) => {
            tracing::error!("Failed to load config: {err}");
            Config::default()
        }
    };

    server_configuration.fs_watcher = ctx.fs_watcher;
    server_configuration.http_server_config.host = ctx.ip_addr;
    server_configuration.http_server_config.port = ctx.port.parse()?;

    let state = ServerState::new(server_configuration).await?;

    org_roamers::start(state).await.unwrap();

    Ok(())
}
