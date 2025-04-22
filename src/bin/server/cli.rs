use std::io::stdin;
use std::io::stdout;
use std::io::BufRead;
use std::io::Write;
use std::process::Command;

use org_roamers::server::ServerRuntime;

use crate::conf::Configuration;

pub fn run_cli(conf: &Configuration, runtime: ServerRuntime) {
    const PS: &'static str = "cli> ";

    print!("{}", PS);

    let _ = stdout().flush();

    for line in stdin().lock().lines() {
        if let Err(ref e) = line {
            println!("ERROR :: Got error while reading line: {e:?}");
        }
        match line.unwrap().as_str() {
            "exit" => {
                runtime.graceful_shutdown().unwrap();
		println!("INFO :: graceful shutdown success. Exiting...");
                return;
            }
            "open" => {
                let _ = Command::new("xdg-open").arg(conf.get_url(true)).status();
            }
            unknown => {
                println!("ERROR :: Unknown command: {}", unknown);
            }
        }
        print!("{}", PS);
        let _ = stdout().flush();
    }
}
