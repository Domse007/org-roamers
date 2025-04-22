use std::{env, process::ExitCode, time::Duration};

use anyhow::Result;
use org_roamers::{api::APICalls, prepare_internal, server::{self, start_server}, StdOutLogger};

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

    let calls = APICalls {
        default_route: server::default_route_content,
        get_graph_data: server::get_graph_data,
        get_org_as_html: server::get_org_as_html,
        serve_search_results: server::search,
        serve_latex_svg: server::get_latex_svg,
    };

    prepare_internal(logger, path.to_string(), sqlite_path.to_string()).unwrap();

    start_server("localhost:5000".to_string(), "web/".to_string(), calls).unwrap();

    std::thread::sleep(Duration::from_secs(100000));

    Ok(ExitCode::SUCCESS)
}
