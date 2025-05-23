use std::io::{BufRead, BufReader, Write, stdin, stdout};
use std::net::TcpStream;

const PS: &str = "cli> ";

pub fn entry() {
    let mut tcp = match TcpStream::connect("localhost:12568") {
        Ok(tcp) => tcp,
        Err(err) => {
            println!("Could not connect to server: {err}");
            return;
        }
    };

    print!("{PS}");

    let _ = stdout().flush();

    for line in stdin().lock().lines() {
        if let Err(ref e) = line {
            println!("ERROR :: Got error while reading line: {e:?}");
        }

        let line = line.unwrap();

        if line.starts_with(".") {
            handle_internal(&line);
            continue;
        }

        let _ = tcp.write(line.as_bytes());
        let _ = tcp.write("\n".as_bytes());

        for line in BufReader::new(tcp.try_clone().unwrap()).lines() {
            let line = line.unwrap();
            if line.trim().is_empty() {
                break;
            }
            println!("{}", line);
        }

        print!("{}", PS);
        let _ = stdout().flush();
    }
}

fn handle_internal(line: &str) {
    match line.trim() {
        ".help" => help(),
        ".exit" => std::process::exit(0),
        unknown => println!("{unknown} unsupported command."),
    }
    print!("{}", PS);
    let _ = stdout().flush();
}

fn help() {
    println!("Supported internal commands:");
    println!("    .help    Show this message");
    println!("    .exit    Disconnect and exit cli\n");
    println!("Supported server commands:");
    println!("    exit     Exit the server");
    println!("    conf     Print the server configuration");
    println!("    open     Open the website using xdg-open");
}
