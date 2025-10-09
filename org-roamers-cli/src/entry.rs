use std::fs;

use anyhow::Result;
use org_roamers::{
    ServerState,
    config::{Config, DEFAULT_CONFIG, ENV_VAR_NAME},
};
use tracing::info;

use crate::conf;

pub fn print_config() {
    eprintln!("Install the file by calling");
    eprintln!("    org-roamers-cli --get-config > DEST");
    eprintln!("The supported destinations are:");
    for p in conf::config_path::paths() {
        if let Some(p) = p {
            eprintln!("  - {}", p.display());
        }
    }
    eprintln!(
        "Alternatively you can set the environment variable {}.\n\n",
        ENV_VAR_NAME
    );
    println!("{}", DEFAULT_CONFIG);
}

pub async fn init_state() -> Result<ServerState> {
    let Some(server_conf_path) = conf::config_path::config_path() else {
        print_config();
        anyhow::bail!("org-roamers cannot find a config file.");
    };

    info!("Using config path {server_conf_path:?}");

    let server_configuration = match fs::read_to_string(server_conf_path) {
        Ok(content) => serde_json::from_str(content.as_str()).unwrap(),
        Err(err) => {
            tracing::error!("Failed to load config: {err}");
            Config::default()
        }
    };

    let state = match ServerState::new(server_configuration).await {
        Ok(g) => g,
        Err(e) => anyhow::bail!("An error occured: {e}"),
    };

    Ok(state)
}

pub fn dump_db(_state: ServerState) -> anyhow::Result<()> {
    // TODO: Implement database dump functionality for sqlx
    // The previous implementation used rusqlite's backup feature which is not available in sqlx
    anyhow::bail!("Database dump functionality is not yet implemented for sqlx")
}
