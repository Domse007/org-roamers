use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub ip_addr: String,
    pub port: String,
    pub roam_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ip_addr: "localhost".to_string(),
            port: "5000".to_string(),
            roam_path: "".to_string(),
        }
    }
}

impl Settings {
    pub fn write<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
	if let Some(parent) = path.as_ref().parent() {
	    if !parent.exists() {
	        fs::create_dir(parent)?;	
	    }
	}
        let mut file = File::create(path)?;
        let deserialized = serde_json::to_string_pretty(self)?;
        file.write(deserialized.as_bytes())?;
        Ok(())
    }

    pub fn read<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let deserialized = fs::read_to_string(path)?;
        serde_json::from_str(deserialized.as_str()).map_err(Into::into)
    }
}
