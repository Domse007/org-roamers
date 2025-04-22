use std::error::Error;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::net::Shutdown;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;

use org_roamers::server::ServerRuntime;
use tracing::error;
use tracing::info;
use tracing::warn;

use crate::conf::Configuration;

pub fn run_cli_server(configuration: &Configuration, runtime: ServerRuntime) {
    let tcp = match TcpListener::bind("localhost:12568") {
        Ok(tcp) => tcp,
        Err(err) => {
            error!("Could not bind {err}");
            return;
        }
    };

    for stream in tcp.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection established.");
                if handle_connection(stream, configuration) {
                    runtime.graceful_shutdown().unwrap();
                    return;
                }
            }
            Err(err) => {
                error!("Connection not established: {err}");
            }
        }
    }
}

fn handle_connection(stream: TcpStream, configuration: &Configuration) -> bool {
    match handle_connection_intern(stream, configuration) {
        Ok(shutdown) => shutdown,
        Err(err) => {
            warn!("An error occured: {err}");
            false
        }
    }
}

fn handle_connection_intern(
    stream: TcpStream,
    configuration: &Configuration,
) -> Result<bool, Box<dyn Error>> {
    let reader = BufReader::new(stream.try_clone()?);

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                warn!("Failed to read line: {err}");
                return Ok(false);
            }
        };

        let mut writer = BufWriter::new(stream.try_clone()?);

        match line.as_str() {
            "exit" => {
                write!(writer, "INFO :: graceful shutdown success. Exiting...")?;
                if let Err(err) = stream.shutdown(Shutdown::Both) {
                    warn!("Could not properly shutdown stream: {err}");
                }
                return Ok(true);
            }
            "conf" => {
                let json = serde_json::to_string_pretty(&configuration)?;
                write!(writer, "{}", json)?;
            }
            "open" => {
                let url = configuration.get_url(true);
                let status = Command::new("xdg-open").arg(&url).status();
                match status {
                    Ok(code) if code.success() => write!(writer, "Successfully opened {}", url)?,
                    _ => write!(writer, "Failed to open {}", url)?,
                }
            }
            unknown => write!(writer, "ERROR :: Unknown command: {}", unknown)?,
        }

        write!(writer, "\n\n")?;
        drop(writer);
    }

    info!("Client disconnected.");

    return Ok(false);
}
