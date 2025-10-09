use std::{env, panic, process::ExitCode};

use org_roamers::start;

mod conf;
mod entry;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .with_line_number(true)
        .init();

    panic::set_hook(Box::new(|info| {
        tracing::error!("Server paniced with {info}")
    }));

    let mut args = env::args().skip(1);

    if let Some(cmd) = args.next() {
        match cmd.as_str() {
            "--server" => {
                let state = match entry::init_state().await {
                    Ok(state) => state,
                    Err(err) => {
                        tracing::error!("{err}");
                        return ExitCode::FAILURE;
                    }
                };
                start(state).await.unwrap();
                tracing::info!("Starting CLI...");
                tracing::info!("Successfully shut down runtime.");
            }
            "--dump-db" => {
                let state = match entry::init_state().await {
                    Ok(state) => state,
                    Err(err) => {
                        tracing::error!("{err}");
                        return ExitCode::FAILURE;
                    }
                };
                if let Err(err) = entry::dump_db(state) {
                    tracing::error!("{err}");
                    return ExitCode::FAILURE;
                }
            }
            "--get-config" => {
                entry::print_config();
            }
            _ => {
                eprintln!("Unsupported command: {cmd}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        eprintln!("No command provided. Use --server, --get-config or --dump-db");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
