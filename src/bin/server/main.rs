mod cli;
mod conf;

use std::{env, fs, panic, path::PathBuf, process::ExitCode, str::FromStr};

use anyhow::Result;
use cli::run_cli_server;
use conf::Configuration;
use org_roamers::{
    api::APICalls,
    server::{self, start_server},
    ServerState,
};
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() -> Result<ExitCode> {
    tracing_subscriber::registry()
        .with(EnvFilter::new("debug"))
        .with(fmt::layer())
        .try_init()
        .unwrap();

    panic::set_hook(Box::new(|info| error!("Server paniced with {info}")));

    let args = env::args().skip(1).collect::<Vec<String>>();

    let path = match args.first() {
        Some(path) => path,
        None => {
            error!("Could not get path");
            return Ok(ExitCode::FAILURE);
        }
    };

    let html_path = {
        let mut path = PathBuf::from_str(conf::CONFIG_PATH).unwrap();
        path.push("html_settings.json");
        path
    };

    let configuration = Configuration {
        html_export_path: html_path,
        roam_path: path.into(),
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
        let assemble = |s| {
            let mut p = PathBuf::from_str(s).unwrap();
            p.push("server_conf.json");
            p
        };
        let path = assemble(conf::CONFIG_PATH);
        if !path.exists() {
            assemble("./")
        } else {
            path
        }
    };
    info!("Using config path {server_conf_path:?}");
    let server_conf_serialized = fs::read_to_string(server_conf_path).unwrap();
    let server_configuration = serde_json::from_str(server_conf_serialized.as_str()).unwrap();

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

    let runtime = start_server(configuration.get_url(false), calls, global).unwrap();

    info!("Starting CLI...");

    run_cli_server(&configuration, runtime);

    Ok(ExitCode::SUCCESS)
}
