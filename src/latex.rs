use std::{
    fmt::Display, fs::File, io::Write, path::{Path, PathBuf}
};

use tempfile::TempDir;

use crate::org;

const PREAMBLE: &'static str =
    "\\documentclass{article}\n[DEFAULT-PACKAGES]\n[PACKAGES]\n\\usepackage{xcolor}";

pub enum LaTeXCompiler {
    LuaLaTeX,
    XeTeX,
    PDFLaTeX,
}

impl Display for LaTeXCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PDFLaTeX => "pdflatex",
                Self::XeTeX => "xetex",
                Self::LuaLaTeX => "lualtex",
            }
        )
    }
}

impl LaTeXCompiler {
    pub fn default_packages(&self) -> Vec<String> {
        let mut packages = vec![];
        if matches!(self, Self::LuaLaTeX | Self::XeTeX) {
            packages.push("\\usepackage{amsmath}".to_string());
            packages.push("\\usepackage{fontspec}".to_string());
        }
        if matches!(self, Self::PDFLaTeX) {
            packages.push("\\usepackage[AUTO]{inputenc}".to_string());
            packages.push("\\usepackage{amsmath}".to_string());
            packages.push("\\usepackage{amssymb}".to_string());
        }

        packages
    }
}

pub struct LaTeXConfig {
    latex: LaTeXCompiler,
    dvisvgm: String,
    preamble: String,
    packages: Option<Vec<String>>,
    working_dir: TempDir,
    file_counter: usize,
}

impl Default for LaTeXConfig {
    fn default() -> Self {
        Self {
            latex: LaTeXCompiler::PDFLaTeX,
            dvisvgm: "dvisvgm".to_string(),
            preamble: PREAMBLE.to_string(),
            packages: None,
            working_dir: TempDir::new().unwrap(),
            file_counter: 0,
        }
    }
}

impl LaTeXConfig {
    pub fn get_commands<P: AsRef<Path>>(&self, outfile: P, infile: P) -> (String, String, String) {
        let compiler = format!(
            "{} -interaction nonstopmode -output-directory {} {}",
            self.latex,
            outfile.as_ref().to_str().unwrap(),
            infile.as_ref().to_str().unwrap()
        );
        let pre_compiler = format!(
            "{} -output-directory %o -ini -jobname=%b \"&%L\" mylatexformat.ltx {}",
            self.latex,
            infile.as_ref().to_str().unwrap()
        );
        let dvisvgm = format!(
            "{} --page=1- --optimize --clipjoin --relative --no-fonts --bbox=preview -o {}.svg {}",
            self.dvisvgm,
            outfile.as_ref().to_str().unwrap(),
            infile.as_ref().to_str().unwrap()
        );

        (compiler, pre_compiler, dvisvgm)
    }

    pub fn get_temp_file(&mut self, extension: &str) -> String {
        let file = format!("preview-{}.{}", self.file_counter, extension);
        self.file_counter += 1;
        file
    }

    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(self.working_dir.path())
    }

    pub fn preamble(&self, headers: Vec<String>) -> String {
        self.preamble
            .clone()
            .replace(
                "[DEFAULT-PACKAGES]",
                &self
                    .latex
                    .default_packages()
                    .into_iter()
                    .map(|mut e| {
                        e.push('\n');
                        e
                    })
                    .collect::<String>(),
            )
            .replace(
                "[PACKAGES]",
                &self
                    .packages
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .chain(headers)
                    .map(|mut e| {
                        e.push('\n');
                        e
                    })
                    .collect::<String>(),
            )
    }
}

pub fn get_image<P: AsRef<Path>>(
    mut config: LaTeXConfig,
    latex: String,
    file: P,
) -> anyhow::Result<Vec<u8>> {
    let headers = org::get_latex_header(file)?;
    let mut path = config.path_buf();
    path.push(config.get_temp_file("tex"));

    let mut file = File::create(path.as_path())?;

    file.write_all(config.preamble(headers).as_bytes())?;
    file.write_all(latex.as_bytes())?;

    unimplemented!()
}
