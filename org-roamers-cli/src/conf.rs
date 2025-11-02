#[cfg(not(target_os = "windows"))]
pub mod config_path {
    use org_roamers::config::ENV_VAR_NAME;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    pub fn paths() -> [Option<PathBuf>; 3] {
        [
            env::var(ENV_VAR_NAME).map(|v| PathBuf::from(v)).ok(),
            Some(PathBuf::from("~/.config/org-roamers/config.json")),
            Some(PathBuf::from("/etc/org-roamers/config.json")),
        ]
    }

    pub fn config_path() -> Option<PathBuf> {
        paths()
            .into_iter()
            .filter(|e| e.is_some())
            .map(|v| v.unwrap())
            .filter(|p| {
                let p = match fs::canonicalize(p) {
                    Ok(p) => p,
                    Err(_) => return false,
                };
                fs::exists(p).unwrap()
            })
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
