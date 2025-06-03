#[cfg(not(target_os = "linux"))]
fn main() {}

#[cfg(target_os = "linux")]
fn main() {
    use std::fs::{self, File};
    use std::io::Write;
    use std::env;

    println!("VARS: {}", env::vars().map(|e|e.0).collect::<Vec<String>>().join(", "));

    fn write_desktop_file<F: Write>(mut file: F) {
        let name = env::var("CARGO_PKG_NAME").unwrap();
	writeln!(file, "[Desktop Entry]").unwrap();
	writeln!(file, "Version={}", env::var("CARGO_PKG_VERSION").unwrap()).unwrap();
	writeln!(file, "Type=Application").unwrap();
	writeln!(file, "Name={name}").unwrap();
	writeln!(file, "Comment={}", env::var("CARGO_PKG_DESCRIPTION").unwrap()).unwrap();
	writeln!(file, "Exec=/usr/local/bin/{name}").unwrap();
	writeln!(file, "Icon={name}").unwrap();
	writeln!(file, "Terminal=false").unwrap();
	writeln!(file, "Categories=Utility;").unwrap();
    }

    let _ = fs::create_dir("../target/");
    let name = env::var("CARGO_PKG_NAME").unwrap();
    let path = format!("../target/{name}.desktop");
    let file = File::create(path).unwrap();
    write_desktop_file(file);

    let _ = fs::create_dir("./target/");
    let path = format!("./target/{name}.desktop");
    let file = File::create(path).unwrap();
    write_desktop_file(file);
    
}
