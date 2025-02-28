use emacs::{defun, Env};
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use crate::{Logger, DB};

#[defun]
pub fn start(env: &Env, path: String) -> emacs::Result<()> {
    let path = PathBuf::from(path);

    start_internal(env, path.as_path()).map_err(|e| emacs::Error::msg(e.to_string()))
}

pub fn start_internal(logger: impl Logger, path: &Path) -> Result<(), Box<dyn Error>> {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let rows = db.sqlite.get_all_nodes(["title", "id", "file"]);

    for row in rows {
        // TODO;
        let [title, id, file] = row;
        let body = String::new();
        crate::add_node_internal(&logger, db, title, id, body, file)?;
    }

    Ok(())
}
