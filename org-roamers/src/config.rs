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
pub struct AuthConfig {
    /// Enable authentication system
    pub enabled: bool,

    /// List of authorized users with plaintext passwords
    /// WARNING: Ensure config file has restricted permissions (chmod 600)
    pub users: Vec<User>,

    /// Session configuration
    #[serde(default)]
    pub session: SessionConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    /// Username for login
    pub username: String,

    /// Plaintext password (hashed on server startup)
    /// WARNING: Keep config file secure
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionConfig {
    /// Session expiry mode: "OnInactivity" or "BrowserSession"
    pub expiry_mode: SessionExpiryMode,

    /// Duration in hours before session expires (for OnInactivity mode)
    pub expiry_duration_hours: u64,

    /// Enable secure cookie flag (requires HTTPS in production)
    pub secure_cookie: bool,

    /// Interval in minutes to run cleanup of expired sessions
    pub cleanup_interval_minutes: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            expiry_mode: SessionExpiryMode::default(),
            expiry_duration_hours: 24,
            secure_cookie: bool::default(),
            cleanup_interval_minutes: 60,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub enum SessionExpiryMode {
    /// Session expires after period of inactivity
    #[default]
    OnInactivity,
    /// Session expires on browser close or after 2 weeks
    BrowserSession,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            users: Vec::new(),
            session: SessionConfig::default(),
        }
    }
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
    /// Authentication configuration (optional - defaults to disabled)
    #[serde(default)]
    pub authentication: Option<AuthConfig>,
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
            authentication: None,
        }
    }
}
