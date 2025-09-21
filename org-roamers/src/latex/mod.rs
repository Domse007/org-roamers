use std::io::Read;
use std::process::Command;
use std::{fs::File, io::Write, path::Path};

use anyhow::bail;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::latex::builder::{LatexBuilder, LatexPathBuilder};
use crate::transform::org;

mod builder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatexConfig {
    latex_cmd: String,
    latex_opt: Vec<String>,
    dvisvgm_cmd: String,
    dvisvgm_opt: Vec<String>,
}

impl Default for LatexConfig {
    fn default() -> Self {
        Self {
            latex_cmd: "latex".to_string(),
            latex_opt: vec!["-interaction".into(), "nonstopmode".into()],
            dvisvgm_cmd: "dvisvgm".to_string(),
            dvisvgm_opt: vec![
                "--optimize".into(),
                "--clipjoin".into(),
                "--relative".into(),
                "--no-fonts".into(),
                "--exact-bbox".into(),
                "--precision=6".into(),
                "--verbosity=0".into(),
            ],
        }
    }
}

pub fn get_image_with_ctx<P: AsRef<Path>>(
    config: &LatexConfig,
    latex: String,
    color: String,
    file: P,
) -> anyhow::Result<Vec<u8>> {
    let headers = org::get_latex_header(file)?;
    get_image(config, latex, color, headers)
}

pub fn get_image(
    config: &LatexConfig,
    latex: String,
    color: String,
    headers: Vec<String>,
) -> anyhow::Result<Vec<u8>> {
    // construct all paths for generated files.
    let (path_tex, path_dvi, path_svg) = LatexPathBuilder::new().build(latex.as_str());
    if let Ok(file) = File::open(path_svg.as_path()) {
        info!("Found preexisting content.");
        return Ok(file.bytes().map(Result::unwrap).collect());
    }

    // build latex file
    let mut latex_builder = LatexBuilder::new();
    latex_builder.headers(&headers);
    latex_builder.body(&[latex.as_str()]);

    let mut file = File::create(path_tex.as_path())?;
    file.write_all(latex_builder.build(&color).as_bytes())?;

    // step 1: compile .tex file to .dvi
    let output = Command::new(&config.latex_cmd)
        .args(config.latex_opt.as_slice())
        .arg(&path_tex)
        .current_dir(path_tex.parent().unwrap())
        .output();

    match output {
        Ok(output) if !output.status.success() => {
            tracing::error!("STDOUT :: {}", String::from_utf8_lossy(&output.stdout));
            tracing::error!("STDERR :: {}", String::from_utf8_lossy(&output.stderr));
            bail!("Failed to execute latex");
        }
        Err(err) => {
            tracing::error!("latex command failed: {}", err);
            bail!("Failed to execute latex");
        }
        _ => {}
    }

    // step 2: compile .dvi to .svg
    let output = Command::new(&config.dvisvgm_cmd)
        .args(config.dvisvgm_opt.as_slice())
        .arg(&path_dvi)
        .arg("-o")
        .arg(&path_svg)
        .current_dir(path_dvi.parent().unwrap())
        .output();

    match output {
        Ok(output) if !output.status.success() => {
            tracing::error!("STDOUT :: {}", String::from_utf8_lossy(&output.stdout));
            tracing::error!("STDERR :: {}", String::from_utf8_lossy(&output.stderr));
            bail!("Failed to execute dvisvgm");
        }
        Err(err) => {
            tracing::error!("latex command failed: {}", err);
            bail!("Failed to execute dvisvgm");
        }
        _ => {}
    }

    // extract svg from file
    info!("Trying to read {}", path_svg.display());
    let file = File::open(path_svg.as_path())?;

    Ok(file.bytes().map(Result::unwrap).collect())
}
