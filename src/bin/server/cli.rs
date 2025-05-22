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

trait CLICommand {
    fn name(&self) -> &'static str;
    fn exec(&self, writer: &mut Box<dyn Write>, stream: &TcpStream, conf: &Configuration);
    fn should_exit(&self) -> bool {
        false
    }
}

struct Exit;

impl CLICommand for Exit {
    fn name(&self) -> &'static str {
        "exit"
    }

    fn exec(&self, writer: &mut Box<dyn Write>, stream: &TcpStream, _conf: &Configuration) {
        write!(writer, "INFO :: graceful shutdown success. Exiting...").unwrap();
        if let Err(err) = stream.shutdown(Shutdown::Both) {
            warn!("Could not properly shutdown stream: {err}");
        }
    }

    fn should_exit(&self) -> bool {
        true
    }
}

struct Conf;

impl CLICommand for Conf {
    fn name(&self) -> &'static str {
        "conf"
    }

    fn exec(&self, writer: &mut Box<dyn Write>, _stream: &TcpStream, conf: &Configuration) {
        let json = serde_json::to_string_pretty(conf).unwrap();
        write!(writer, "{}", json).unwrap();
    }
}

struct Open;

impl CLICommand for Open {
    fn name(&self) -> &'static str {
        "open"
    }

    fn exec(&self, writer: &mut Box<dyn Write>, _stream: &TcpStream, conf: &Configuration) {
        let url = conf.get_url(true);
        let status = Command::new("xdg-open").arg(&url).status();
        match status {
            Ok(code) if code.success() => write!(writer, "Successfully opened {}", url).unwrap(),
            _ => write!(writer, "Failed to open {}", url).unwrap(),
        }
    }
}

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

    let commands: Vec<Box<dyn CLICommand>> = vec![Box::new(Exit), Box::new(Conf), Box::new(Open)];

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                warn!("Failed to read line: {err}");
                return Ok(false);
            }
        };

        let mut writer: Box<dyn Write> = Box::new(BufWriter::new(stream.try_clone()?));
        let mut flag = true;

        for cmd in &commands {
            if cmd.name() == line.as_str() {
                flag = false;
                cmd.exec(&mut writer, &stream, configuration);
                if cmd.should_exit() {
                    return Ok(true);
                }
                break;
            }
        }

        if flag {
            write!(writer, "ERROR :: Unknown command: {}", line)?
        }

        write!(writer, "\n\n")?;
        drop(writer);
    }

    info!("Client disconnected.");

    Ok(false)
}
