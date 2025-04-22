use std::{
    error::Error,
    path::{Path, PathBuf},
};

use crate::{log, org, Global, Logger, DB};

pub fn start_internal(
    logger: impl Logger,
    db: &mut Global,
    _path: &Path,
) -> Result<(), Box<dyn Error>> {
    let rows = db.sqlite.get_all_nodes(["title", "id", "file"]);

    for row in rows {
        // TODO;
        let [title, id, file] = row;
        let path = PathBuf::from(&file);

        log!(logger, "Adding {} to index: {}", title, file);

        if !path.exists() {
            return Err(format!("File '{}' does not exist.", path.to_str().unwrap()).into());
        }

        let nodes = org::get_nodes_from_file(path.as_path()).map_err(|e| e.to_string())?;

        let mut body = None;

        for node in nodes {
            if node.uuid == id {
                body = Some(node.content);
                break;
            }
        }

        if let Some(body) = body {
            crate::add_node_internal(&logger, db, title, id, body, file)?;
        } else {
            return Err(format!(
                "Could not get file contents for: {}",
                path.to_str().unwrap()
            )
            .into());
        }
    }

    Ok(())
}
