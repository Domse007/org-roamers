use std::io::{stdin, stdout, BufRead, Write, BufReader};
use std::net::TcpStream;

fn main() {
    let mut tcp = match TcpStream::connect("localhost:12568") {
        Ok(tcp) => tcp,
        Err(err) => {
            println!("Could not connect to server: {err}");
            return;
        }
    };

    const PS: &'static str = "cli> ";

    print!("{PS}");

    let _ = stdout().flush();

    for line in stdin().lock().lines() {
        if let Err(ref e) = line {
            println!("ERROR :: Got error while reading line: {e:?}");
        }

        let _ = tcp.write(line.unwrap().as_bytes());
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
