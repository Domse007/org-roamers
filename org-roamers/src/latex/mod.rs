use anyhow::bail;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tracing::info;

use crate::config::LatexConfig;
use crate::latex::builder::{LatexBuilder, LatexPathBuilder};

mod builder;

pub async fn get_image(
    config: &LatexConfig,
    latex: String,
    color: String,
    headers: Vec<String>,
) -> anyhow::Result<Vec<u8>> {
    // construct all paths for generated files.
    let (path_tex, path_dvi, path_svg) = LatexPathBuilder::new().build(latex.as_str());
    if let Ok(mut file) = File::open(path_svg.as_path()).await {
        info!("Found preexisting content.");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        return Ok(buffer);
    }

    // build latex file
    let mut latex_builder = LatexBuilder::new();
    latex_builder.headers(&headers);
    latex_builder.body(&[latex.as_str()]);

    let mut file = File::create(path_tex.as_path()).await?;
    file.write_all(latex_builder.build(&color).as_bytes())
        .await?;

    // step 1: compile .tex file to .dvi
    let output = Command::new(&config.latex_cmd)
        .args(config.latex_opt.as_slice())
        .arg(&path_tex)
        .current_dir(path_tex.parent().unwrap())
        .output()
        .await;

    match output {
        Ok(output) if !output.status.success() => {
            tracing::error!("Could not compile: {latex}");
            tracing::error!("STDOUT :: {}", String::from_utf8_lossy(&output.stdout));
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
        .output()
        .await;

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
    let mut file = File::open(path_svg.as_path()).await?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}
