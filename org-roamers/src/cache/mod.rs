//! # Org cache
//! This module is a cache between the file system and the db. It aims to
//! improve full text searching and simplify the checks for the fs watcher.
//! It should reduce the file lookup to just fetching updated files.

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use rusqlite::Connection;

use crate::{
    cache::{file::OrgFile, fileiter::FileIter},
    server::types::RoamID,
    sqlite::files::insert_file,
    transform::org,
};

mod file;
mod fileiter;

#[derive(Debug)]
pub struct OrgCacheEntry {
    path: PathBuf,
    content: String,
}

impl OrgCacheEntry {
    pub fn new<P: AsRef<Path>>(root: P, path: P) -> io::Result<Self> {
        let mut file = OrgFile::open(&path)?;
        Ok(Self {
            path: path.as_ref().strip_prefix(root).unwrap().to_path_buf(),
            content: file.read_to_string()?,
        })
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug)]
pub enum InvalidatedBy {
    Path(PathBuf),
    Id(RoamID),
}

impl From<PathBuf> for InvalidatedBy {
    fn from(value: PathBuf) -> Self {
	Self::Path(value)
    }
}

impl From<RoamID> for InvalidatedBy {
    fn from(value: RoamID) -> Self {
	Self::Id(value)
    }
}

#[derive(Debug)]
pub struct OrgCache {
    /// Path to the root of the org-roamers directory.
    path: PathBuf,
    lookup: HashMap<RoamID, Arc<OrgCacheEntry>>,
    // TODO: currently not processed. diff::diff will be required at some point...
    invalidated: Vec<InvalidatedBy>
}

impl OrgCache {
    pub fn new(root: PathBuf) -> Self {
        Self {
            path: root,
            lookup: HashMap::new(),
	    invalidated: Vec::new()
        }
    }

    pub fn rebuild(&mut self, con: &mut Connection) -> anyhow::Result<()> {
        let file_iter = FileIter::new(&self.path)?;

        for file_or_error in file_iter {
            let file_path = match file_or_error {
                Ok(file_path) => file_path,
                Err(err) => {
                    tracing::error!("{err}");
                    continue;
                }
            };

            let cache_entry = match OrgCacheEntry::new(self.path.as_path(), file_path.as_path()) {
                Ok(entry) => entry,
                Err(err) => {
                    tracing::error!("{err}");
                    continue;
                }
            };

            if let Err(err) = insert_file(con, cache_entry.path(), cache_entry.get_hash()) {
                tracing::error!("{err}");
            }

            let nodes = org::get_nodes(cache_entry.content());

            let cache_entry = Arc::new(cache_entry);
            for node in &nodes {
                self.lookup
                    .insert(node.uuid.clone().into(), cache_entry.clone());
            }

            org::insert_nodes(con, nodes);
        }

        Ok(())
    }

    pub fn get_by_name(
        &self,
        con: &mut Connection,
        name: &str,
    ) -> Option<(RoamID, &OrgCacheEntry)> {
        let stmnt = r#"
            SELECT id FROM nodes
            WHERE title = ?1;
        "#;

        let id = con.query_row(stmnt, [name], |row| Ok(row.get_unwrap::<usize, String>(0)));
        let id = match id {
            Ok(id) => id,
            Err(err) => {
                tracing::error!("{err}");
                return None;
            }
        };

        match self.retrieve(&id.as_str().into()) {
            Some(content) => Some((id.into(), content)),
            None => None,
        }
    }

    // pub fn submit<P: AsRef<Path>>(&mut self, path: P, node: NodeFromOrg) {
    //     let org_file = OrgCacheEntry {
    //         path: path.as_ref().to_path_buf(),
    //         content: node.content,
    //     };

    //     tracing::info!("Submitted {} into cache.", node.uuid);

    //     self.lookup.insert(node.uuid.into(), org_file);
    // }

    pub fn retrieve(&self, id: &RoamID) -> Option<&OrgCacheEntry> {
        self.lookup.get(id).map(|e| e.deref())
    }

    pub fn invalidate<T: Into<InvalidatedBy>>(&mut self, by: T) {
	self.invalidated.push(by.into());
    }

    /// Under most circumstances: DO NOT USE!
    pub fn path(&self) -> &Path {
        &self.path
    }
}
