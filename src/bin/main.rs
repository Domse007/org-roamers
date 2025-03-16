use std::{env, process::ExitCode, time::Duration};

use anyhow::Result;
use org_roamers::{prepare_internal, server::start_server, StdOutLogger};

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

    prepare_internal(logger, path.to_string(), sqlite_path.to_string()).unwrap();

    start_server("localhost:5000".to_string(), "web/".to_string()).unwrap();

    std::thread::sleep(Duration::from_secs(100000));

    Ok(ExitCode::SUCCESS)
}
