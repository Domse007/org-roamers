use std::{env, process::ExitCode};

mod cli;
mod server;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);

    if let Some(cmd) = args.next() {
        match cmd.as_str() {
            "--server" => server::entry(args.collect()).unwrap(),
            "--cli" => {
                cli::entry();
                ExitCode::SUCCESS
            }
            _ => {
                eprintln!("Unsupported command: {cmd}");
                ExitCode::FAILURE
            }
        }
    } else {
        eprintln!("No command provided. Use --server or --cli");
        ExitCode::FAILURE
    }
}
