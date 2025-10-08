#[cfg(not(target_os = "windows"))]
pub mod config_path {
    use org_roamers::config::ENV_VAR_NAME;
    use std::path::PathBuf;
    use std::{env, fs};

    pub fn paths() -> [Option<PathBuf>; 4] {
        [
            env::var(ENV_VAR_NAME).map(|v| PathBuf::from(v)).ok(),
            Some(PathBuf::from("./conf.json")),
            Some(PathBuf::from("~/.config/org-roamers/conf.json")),
            Some(PathBuf::from("/etc/org-roamers/conf.json")),
        ]
    }

    pub fn config_path() -> Option<PathBuf> {
        paths()
            .into_iter()
            .filter(|e| e.is_some())
            .map(|v| v.unwrap())
            .filter(|p| fs::exists(p).unwrap())
            .next()
    }
}

#[cfg(target_os = "windows")]
pub mod config_path {
    pub fn paths() -> [Option<PathBuf>; 1] {
        [std::env::var("APPDATA").map(PathBuf::from).ok()]
    }
    pub fn config_path() -> Option<PathBuf> {
        paths()
    }
}
