mod args;
mod cli;
mod conf;

use std::{env, fs, panic, process::ExitCode};

use anyhow::Result;
use args::CliArgs;
use cli::run_cli_server;
use conf::Configuration;
use org_roamers::{
    ServerState,
    config::{Config, DEFAULT_CONFIG, ENV_VAR_NAME},
    server::start_server,
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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

pub fn entry(args: Vec<String>) -> Result<ExitCode> {
    // tracing_subscriber::registry()
    //     .with(EnvFilter::new("debug"))
    //     .with(fmt::layer())
    //     .try_init()
    //     .unwrap();

    tracing_subscriber::fmt()
        .with_file(true)
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .with_line_number(true)
        .init();

    panic::set_hook(Box::new(|info| error!("Server paniced with {info}")));

    let cli_args = match CliArgs::parse(&args) {
        Ok(args) => args,
        Err(err) => {
            error!("An error occured while parsing args: {err}");
            return Ok(ExitCode::FAILURE);
        }
    };

    let configuration = Configuration {
        roam_path: cli_args.path.into(),
        ip_addr: "0.0.0.0".to_string(),
        port: 5000,
    };

    let Some(server_conf_path) = conf::config_path::config_path() else {
        eprintln!("org-roamers cannot find a config file.");
        print_config();

        return Ok(ExitCode::FAILURE);
    };

    info!("Using config path {server_conf_path:?}");

    let mut server_configuration = match fs::read_to_string(server_conf_path) {
        Ok(content) => serde_json::from_str(content.as_str()).unwrap(),
        Err(err) => {
            tracing::error!("Failed to load config: {err}");
            Config::default()
        }
    };

    server_configuration.fs_watcher = cli_args.fs_watcher;

    let mut global = match ServerState::new(server_configuration) {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("An error occured: {e}");
            return Ok(ExitCode::FAILURE);
        }
    };

    if cli_args.dump {
        let mut dump_path = env::current_dir().unwrap();
        dump_path.push("dump.db");
        if std::fs::exists(&dump_path).unwrap() {
            std::fs::remove_file(&dump_path).unwrap();
        }
        global
            .sqlite
            .connection()
            .backup(rusqlite::DatabaseName::Main, &dump_path, None)?;
        tracing::info!("Saved db dump to {}", dump_path.display());
        return Ok(ExitCode::SUCCESS);
    }

    let runtime = start_server(configuration.get_url(false), global).unwrap();

    info!("Starting CLI...");

    run_cli_server(&configuration, runtime);

    Ok(ExitCode::SUCCESS)
}
