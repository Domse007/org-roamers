use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG: &str = include_str!("../../conf.json");
pub const ENV_VAR_NAME: &str = "ROAMERS_DIR";

#[derive(Serialize, Deserialize, Clone)]
pub struct HttpServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5000,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct EnvAdvice {
    pub on: String,
    pub header: String,
    pub css_style: String,
    pub text_styling: String,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct HtmlExportSettings {
    pub respect_noexport: bool,
    pub env_advices: Vec<EnvAdvice>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LatexConfig {
    pub latex_cmd: String,
    pub latex_opt: Vec<String>,
    pub dvisvgm_cmd: String,
    pub dvisvgm_opt: Vec<String>,
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

#[derive(Serialize, Deserialize, Clone, Default, Copy)]
pub enum AssetPolicy {
    AllowAll,
    ForbidAll,
    #[default]
    AllowChildrenOfRoot,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    /// Path to the root of the org-roamers / org-roam directory.
    pub org_roamers_root: PathBuf,
    /// Settings that configure the webserver.
    pub http_server_config: HttpServerConfig,
    /// HTML settings when exporting org environments to HTML.
    pub org_to_html: HtmlExportSettings,
    /// Root path to the website files. e.g. .js / .html / .css
    pub root: PathBuf,
    /// Use the filesystem watcher
    pub fs_watcher: bool,
    /// LaTeX settings for rendering fragments
    pub latex_config: LatexConfig,
    /// Settings on asset loading restrictions
    pub asset_policy: AssetPolicy,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            org_roamers_root: "~/notes/".into(),
            http_server_config: HttpServerConfig::default(),
            org_to_html: HtmlExportSettings::default(),
            root: "./web/dist/".into(),
            fs_watcher: false,
            latex_config: LatexConfig::default(),
            asset_policy: AssetPolicy::default(),
        }
    }
}
