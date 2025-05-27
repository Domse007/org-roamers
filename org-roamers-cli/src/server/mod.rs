mod args;
mod cli;
mod conf;

use std::{env, fs, panic, path::PathBuf, process::ExitCode};

use anyhow::Result;
use args::CliArgs;
use cli::run_cli_server;
use conf::Configuration;
use org_roamers::{
    ServerState, StaticServerConfiguration,
    api::APICalls,
    server::{self, start_server},
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn entry(args: Vec<String>) -> Result<ExitCode> {
    tracing_subscriber::registry()
        .with(EnvFilter::new("debug"))
        .with(fmt::layer())
        .try_init()
        .unwrap();

    panic::set_hook(Box::new(|info| error!("Server paniced with {info}")));

    let cli_args = match CliArgs::parse(&args) {
        Ok(args) => args,
        Err(err) => {
            error!("An error occured while parsing args: {err}");
            return Ok(ExitCode::FAILURE);
        }
    };

    let html_path = {
        let mut path = conf::config_path();
        path.push("html_settings.json");
        path
    };

    let configuration = Configuration {
        html_export_path: html_path,
        roam_path: cli_args.path.into(),
        ip_addr: "0.0.0.0".to_string(),
        port: 5000,
    };

    let calls = APICalls {
        default_route: server::default_route_content,
        get_graph_data: server::get_graph_data,
        get_org_as_html: server::get_org_as_html,
        serve_search_results: server::search,
        serve_latex_svg: server::get_latex_svg,
        get_status_data: server::get_status_data,
    };

    let server_conf_path = {
        let mut path = conf::config_path();
        path.push("server_conf.json");
        if !path.exists() {
            PathBuf::from("./server_conf.json")
        } else {
            path
        }
    };
    info!("Using config path {server_conf_path:?}");

    let server_configuration = match fs::read_to_string(server_conf_path) {
        Ok(content) => serde_json::from_str(content.as_str()).unwrap(),
        Err(err) => {
            tracing::error!("Failed to load config: {err}");
            StaticServerConfiguration::default()
        }
    };

    let mut global = match ServerState::new(
        configuration.html_export_path.as_path(),
        configuration.roam_path.as_path(),
        server_configuration,
    ) {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("An error occured: {e}");
            return Ok(ExitCode::FAILURE);
        }
    };

    if let Err(err) = global.sqlite.insert_files(&configuration.roam_path) {
        tracing::error!("An error occured: {err}");
    }

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

    let runtime = start_server(configuration.get_url(false), calls, global).unwrap();

    info!("Starting CLI...");

    run_cli_server(&configuration, runtime);

    Ok(ExitCode::SUCCESS)
}
