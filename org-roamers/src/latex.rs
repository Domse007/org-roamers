use std::env;
use std::io::Read;
use std::process::Command;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::bail;
use tracing::info;

use crate::file::OrgFile;
use crate::transform::org;

fn get_temp_path() -> PathBuf {
    let mut temp_dir = env::temp_dir();
    temp_dir.push("org-roamers/");
    temp_dir
}

const PREAMBLE: &str = concat!(
    "\\documentclass{article}\n",
    "\\usepackage[T1]{fontenc}\n",
    "\\usepackage[active,tightpage]{preview}\n",
    "\\usepackage{amsmath}\n",
    "\\usepackage{amssymb}\n",
    "[PACKAGES]\n",
    "\\usepackage{xcolor}\n"
);

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
    let mut path = PathBuf::from(get_temp_path());
    std::fs::create_dir_all(path.as_path())?;

    // let's check if the file already exists.
    let mut existing_path = path.clone();
    existing_path.push(format!("{}.svg", hash));
    if let Ok(mut file) = OrgFile::open(existing_path.as_path()) {
        let contents = file.read_to_string()?;
        info!("Found preexisting content for {hash} ({:?})", existing_path);
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

    let output = Command::new("latex")
        .args([
            "-interaction",
            "nonstopmode",
            in_file.as_path().to_str().unwrap(),
        ])
        .current_dir(path.as_path())
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                tracing::error!("STDOUT :: {}", String::from_utf8_lossy(&output.stdout));
                tracing::error!("STDERR :: {}", String::from_utf8_lossy(&output.stderr));
                bail!("Failed to execute latex");
            }
        }
        Err(err) => {
            tracing::error!("latex command failed: {}", err);
            bail!("Failed to execute latex");
        }
    }

    let mut in_file = path.clone();
    in_file.push(format!("{}.dvi", hash));
    let output = Command::new("dvisvgm")
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
        .output();

    match output {
        Ok(output) => {
            if !output.status.success() {
                tracing::error!("STDOUT :: {}", String::from_utf8_lossy(&output.stdout));
                tracing::error!("STDERR :: {}", String::from_utf8_lossy(&output.stderr));
                bail!("Failed to execute dvisvgm");
            }
        }
        Err(err) => {
            tracing::error!("dvisvgm command failed: {}", err);
            bail!("Failed to execute dvisvgm");
        }
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
