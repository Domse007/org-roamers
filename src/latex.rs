use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::{
    fmt::Display,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::bail;
use tracing::info;

use crate::org;

const PREAMBLE: &'static str = "\\documentclass{article}
\\usepackage[T1]{fontenc}
\\usepackage[active,tightpage]{preview}
\\usepackage{amsmath}
\\usepackage{amssymb}
[PACKAGES]
\\usepackage{xcolor}";

fn preamble(headers: Vec<String>) -> String {
    PREAMBLE.replace(
        "[PACKAGES]",
        headers
            .into_iter()
            .map(|mut e| {
                e.push('\n');
                e
            })
            .collect::<String>()
            .as_str(),
    )
}

pub fn get_image_with_ctx<P: AsRef<Path>>(
    latex: String,
    color: String,
    file: P,
) -> anyhow::Result<String> {
    let headers = org::get_latex_header(file)?;
    get_image(latex, color, headers)
}

pub fn get_image(latex: String, color: String, headers: Vec<String>) -> anyhow::Result<String> {
    let hash = hash(latex.as_str());
    // TODO: only works on linux.
    let mut path = PathBuf::from("/tmp/org-roamers/");
    std::fs::create_dir_all(path.as_path())?;

    // let's check if the file already exists.
    let mut existing_path = path.clone();
    existing_path.push(format!("{}.svg", hash));
    if let Ok(mut file) = File::open(existing_path.as_path()) {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        info!("Found preexisting content for {hash}.");
        return Ok(contents);
    }

    let mut in_file = path.clone();
    in_file.push(format!("{}.tex", hash));

    let mut file = File::create(in_file.as_path())?;
    file.write_all(preamble(headers).as_bytes())?;
    file.write_all(format!("\\definecolor{{mycolor}}{{HTML}}{{{color}}}\n").as_bytes())?;
    file.write_all("\n\\begin{document}\n".as_bytes())?;
    file.write_all("\\begin{preview}\n".as_bytes())?;
    file.write_all("\\color{mycolor}\n".as_bytes())?;
    file.write_all(latex.as_bytes())?;
    file.write_all("\n\\end{preview}\n\\end{document}\n".as_bytes())?;

    let status = Command::new("latex")
        .args([
            "-interaction",
            "nonstopmode",
            in_file.as_path().to_str().unwrap(),
        ])
        .current_dir(path.as_path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        bail!("Failed to execute latex");
    }

    let mut in_file = path.clone();
    in_file.push(format!("{}.dvi", hash));
    let status = Command::new("dvisvgm")
        .args([
            "--optimize",
            "--clipjoin",
            "--relative",
            "--bbox=preview",
            "--no-fonts",
            in_file.as_path().to_str().unwrap(),
            "-o",
            format!("{}.svg", hash).as_str(),
        ])
        .current_dir(path.as_path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        bail!("Failed to execute dvisvgm");
    }

    path.push(format!("{}.svg", hash));

    info!("Trying to read {}", path.display());

    let mut file = File::open(path.as_path())?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    Ok(s)
}

fn hash(s: &str) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::default();
    s.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_image() {
        let res = get_image("\\(f(x):=x^2\\)".to_string(), "FFFFFF".to_string(), vec![]);
        println!("{res:#?}");
        assert!(res.is_ok());
    }
}
