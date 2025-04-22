mod cli;
mod conf;

use crate::cli::run_cli;
use std::{env, process::ExitCode};

use anyhow::Result;
use conf::Configuration;
use org_roamers::{
    api::APICalls,
    prepare_internal,
    server::{self, start_server},
    StdOutLogger,
};

fn main() -> Result<ExitCode> {
    let logger = StdOutLogger;

    let args = env::args().skip(1).collect::<Vec<String>>();

    let path = match args.get(0) {
        Some(path) => path,
        None => {
            println!("Could not get path");
            return Ok(ExitCode::FAILURE);
        }
    };

    let sqlite_path = match args.get(1) {
        Some(path) => path,
        None => {
            println!("Could not get sqlite_path.");
            return Ok(ExitCode::FAILURE);
        }
    };

    let configuration = Configuration {
        sqlite_path: sqlite_path.to_string(),
        roam_path: path.to_string(),
        ip_addr: "localhost".to_string(),
        port: 5000,
    };

    let calls = APICalls {
        default_route: server::default_route_content,
        get_graph_data: server::get_graph_data,
        get_org_as_html: server::get_org_as_html,
        serve_search_results: server::search,
        serve_latex_svg: server::get_latex_svg,
    };

    prepare_internal(
        logger,
        configuration.roam_path.as_str(),
        configuration.sqlite_path.as_str(),
    )
    .unwrap();

    let runtime = start_server(configuration.get_url(false), "web/".to_string(), calls).unwrap();

    println!("Starting CLI...");

    run_cli(&configuration, runtime);

    Ok(ExitCode::SUCCESS)
}
