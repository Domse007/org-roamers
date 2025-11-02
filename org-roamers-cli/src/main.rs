use std::{panic, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use org_roamers::start;

mod conf;
mod entry;

#[derive(Parser)]
#[command(about = "The cli interface for org-roamers.")]
struct Cli {
    /// The subcommand to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Command for server
    Server {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Command for get-config
    GetConfig {
        #[arg(long)]
        with_auth: bool,
    },
    /// Dump the DB after initialisation
    DumpConfig {
        #[arg(long)]
        config: Option<PathBuf>,
    },
}

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

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { config } => {
            let state = match entry::init_state(config).await {
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
        Commands::DumpConfig { config } => {
            let state = match entry::init_state(config).await {
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
        Commands::GetConfig { with_auth } => {
            entry::print_config(with_auth);
        }
    }

    ExitCode::SUCCESS
}
