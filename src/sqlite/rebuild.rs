use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::Path,
};

use rusqlite::{params, Connection};

use crate::{sqlite::olp, transform::org};

#[allow(clippy::too_many_arguments)]
pub fn insert_node(
    con: &mut Connection,
    id: &str,
    file: &str,
    level: u64,
    todo: bool,
    priority: usize,
    scheduled: &str,
    deadline: &str,
    title: &str,
    olp: &[String],
) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO nodes (id, file, level, todo, priority, scheduled, deadline, title)\n",
        "VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);"
    );
    con.execute(
        STMNT,
        params![id, file, level, todo, priority, scheduled, deadline, title],
    )?;

    olp::insert_olp(con, id, olp)?;

    Ok(())
}

pub fn insert_tag(con: &mut Connection, id: &str, tag: &str) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO tags (node_id, tag)\n",
        "VALUES (?1, ?2);"
    );
    con.execute(STMNT, params![id, tag])?;
    Ok(())
}

pub fn insert_link(con: &mut Connection, source: &str, dest: &str) -> anyhow::Result<()> {
    const TYPE: &str = "id";
    const PROPERTIES: &str = "";
    const POS: usize = 0;
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO links (pos, source, dest, type, properties)\n",
        "VALUES (?1, ?2, ?3, ?4, ?5);"
    );
    con.execute(STMNT, params![POS, source, dest, TYPE, PROPERTIES])?;
    Ok(())
}

#[derive(Default)]
pub(crate) struct IterFilesStats {
    pub num_files: usize,
    pub num_nodes: usize,
    pub num_links: usize,
    pub num_tags: usize,
}

#[derive(thiserror::Error, Debug)]
pub struct IterFilesError {
    entry: DirEntry,
    error: Box<dyn std::error::Error>,
}

impl IterFilesError {
    pub fn new(entry: DirEntry, error: Box<dyn std::error::Error>) -> Self {
        Self { entry, error }
    }
}

impl std::fmt::Display for IterFilesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} has produced error: {}", self.entry, self.error)
    }
}

pub(super) fn iter_files<P: AsRef<Path>>(
    con: &mut Connection,
    roam_path: P,
    stats: &mut IterFilesStats,
) -> Result<(), IterFilesError> {
    let inc = |n: &mut usize| *n += 1;

    for entry in fs::read_dir(roam_path).unwrap() {
        let entry = entry.unwrap();

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => return Err(IterFilesError::new(entry, Box::new(err))),
        };

        if metadata.is_dir() {
            iter_files(con, entry.path(), stats)?;
        }

        if metadata.is_file() && entry.path().extension() == Some(OsStr::new("org")) {
            inc(&mut stats.num_files);

            let nodes = match org::get_nodes_from_file(entry.path()) {
                Ok(nodes) => nodes,
                Err(err) => return Err(IterFilesError::new(entry, err.into())),
            };

            for node in nodes {
                inc(&mut stats.num_nodes);
                let res = self::insert_node(
                    con,
                    &node.uuid,
                    node.file.as_str(),
                    node.level,
                    false,
                    0,
                    "",
                    "",
                    node.title.as_str(),
                    &node.actual_olp,
                );

                if let Err(err) = res {
                    return Err(IterFilesError::new(entry, err.into()));
                }

                for tag in node.tags {
                    inc(&mut stats.num_tags);
                    if let Err(err) = insert_tag(con, &node.uuid, &tag) {
                        return Err(IterFilesError::new(entry, err.into()));
                    }
                }
                for link in node.links {
                    inc(&mut stats.num_links);
                    if let Err(err) = insert_link(con, &node.uuid, &link.0) {
                        return Err(IterFilesError::new(entry, err.into()));
                    }
                }
                // TODO: add files. For this title is required, which is the
                // toplevel `#+title` tile.
            }
        }
    }

    Ok(())
}
