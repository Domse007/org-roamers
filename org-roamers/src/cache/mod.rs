//! # Org cache
//! This module is a cache between the file system and the db. It aims to
//! improve full text searching and simplify the checks for the fs watcher.
//! It should reduce the file lookup to just fetching updated files.

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use dashmap::{mapref::multiple::RefMulti, DashMap};
use sqlx::SqlitePool;

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
    pub fn new<P: AsRef<Path>, PP: AsRef<Path>>(root: P, path: PP) -> io::Result<Self> {
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
    lookup: DashMap<RoamID, Arc<OrgCacheEntry>>,
    // TODO: currently not processed. File changes are handled by the watcher.
    invalidated: Mutex<Vec<InvalidatedBy>>,
}

impl OrgCache {
    pub fn new(root: PathBuf) -> Self {
        Self {
            path: root,
            lookup: DashMap::new(),
            invalidated: Mutex::new(Vec::new()),
        }
    }

    pub async fn rebuild(&mut self, con: &SqlitePool) -> anyhow::Result<()> {
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

            if let Err(err) = insert_file(con, cache_entry.path(), cache_entry.get_hash()).await {
                tracing::error!("{err}");
            }

            let file_path = cache_entry.path().to_string_lossy().to_string();
            let nodes = org::get_nodes(cache_entry.content(), &file_path);

            let cache_entry = Arc::new(cache_entry);
            for node in &nodes {
                self.lookup
                    .insert(node.uuid.clone().into(), cache_entry.clone());
            }

            org::insert_nodes(con, nodes).await;
        }

        Ok(())
    }

    pub async fn get_by_name(
        &self,
        con: &SqlitePool,
        name: &str,
    ) -> Option<(RoamID, Arc<OrgCacheEntry>)> {
        let stmnt = r#"
            SELECT id FROM nodes
            WHERE title = ?;
        "#;

        let id: (String,) = sqlx::query_as(stmnt)
            .bind(name)
            .fetch_one(con)
            .await
            .unwrap();

        match self.retrieve(&id.0.as_str().into()) {
            Some(content) => Some((id.0.into(), content)),
            None => None,
        }
    }

    pub fn submit<P: AsRef<Path>>(&mut self, id: RoamID, path: P) -> anyhow::Result<()> {
        let cache_entry = OrgCacheEntry::new(&self.path, path)?;
        let cache_entry_arc = Arc::new(cache_entry);

        tracing::info!("Submitted {:?} into cache.", cache_entry_arc.path());

        // Find all entries that point to the same file and update them all
        let file_path = cache_entry_arc.path();
        let mut ids_to_update = Vec::new();

        let mut iter = self.lookup.iter_mut();
        while let Some(mut ref_tuple) = iter.next() {
            let (existing_id, existing_entry) = ref_tuple.pair_mut();
            if existing_entry.path() == file_path {
                ids_to_update.push(existing_id.clone());
            }
        }

        // Update all entries for this file
        for id_to_update in ids_to_update {
            self.lookup.insert(id_to_update, cache_entry_arc.clone());
        }

        // Also ensure the requested ID is in the cache
        self.lookup.insert(id, cache_entry_arc);

        Ok(())
    }

    pub fn retrieve(&self, id: &RoamID) -> Option<Arc<OrgCacheEntry>> {
        self.lookup.get(id).map(|r| r.value().clone())
    }

    pub fn invalidate<T: Into<InvalidatedBy>>(&self, by: T) {
        self.invalidated.lock().unwrap().push(by.into());
    }

    /// Under most circumstances: DO NOT USE!
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn iter<'a>(&self) -> impl Iterator<Item = RefMulti<'_, RoamID, Arc<OrgCacheEntry>>> {
        self.lookup.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_org_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_submit_updates_all_nodes_from_same_file() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = OrgCache::new(temp_dir.path().to_path_buf());

        // Create an org file with multiple nodes
        let org_content_v1 = r#":PROPERTIES:
:ID: node-1
:END:
#+title: Test File

* First Heading
:PROPERTIES:
:ID: node-2
:END:
Content 1

* Second Heading  
:PROPERTIES:
:ID: node-3
:END:
Content 2
"#;

        let org_file = create_test_org_file(temp_dir.path(), "test.org", org_content_v1);

        // Manually populate cache as if nodes were processed (simulating rebuild)
        let cache_entry_v1 = OrgCacheEntry::new(temp_dir.path(), &org_file).unwrap();
        let cache_arc_v1 = Arc::new(cache_entry_v1);

        // Insert all three nodes pointing to the same cache entry
        cache.lookup.insert("node-1".into(), cache_arc_v1.clone());
        cache.lookup.insert("node-2".into(), cache_arc_v1.clone());
        cache.lookup.insert("node-3".into(), cache_arc_v1.clone());

        // Verify all nodes point to the same cache entry
        let entry1_ptr = Arc::as_ptr(&cache.lookup.get(&"node-1".into()).unwrap());
        let entry2_ptr = Arc::as_ptr(&cache.lookup.get(&"node-2".into()).unwrap());
        let entry3_ptr = Arc::as_ptr(&cache.lookup.get(&"node-3".into()).unwrap());
        assert_eq!(entry1_ptr, entry2_ptr);
        assert_eq!(entry2_ptr, entry3_ptr);

        // Now update the file content
        let org_content_v2 = r#":PROPERTIES:
:ID: node-1
:END:
#+title: Test File UPDATED

* First Heading
:PROPERTIES:
:ID: node-2
:END:
Content 1 UPDATED

* Second Heading  
:PROPERTIES:
:ID: node-3
:END:
Content 2 UPDATED
"#;

        fs::write(&org_file, org_content_v2).unwrap();

        // Submit update for just one node
        cache.submit("node-2".into(), &org_file).unwrap();

        // Verify ALL nodes now point to the NEW cache entry
        let new_entry1_ptr = Arc::as_ptr(&cache.lookup.get(&"node-1".into()).unwrap());
        let new_entry2_ptr = Arc::as_ptr(&cache.lookup.get(&"node-2".into()).unwrap());
        let new_entry3_ptr = Arc::as_ptr(&cache.lookup.get(&"node-3".into()).unwrap());

        // All should point to the same NEW entry
        assert_eq!(new_entry1_ptr, new_entry2_ptr);
        assert_eq!(new_entry2_ptr, new_entry3_ptr);

        // The new entry should be different from the old one
        assert_ne!(new_entry1_ptr, entry1_ptr);

        // Verify the content was actually updated
        let binding = cache.retrieve(&"node-1".into()).unwrap();
        let updated_content = binding.content();
        assert!(updated_content.contains("UPDATED"));
        let binding = cache.retrieve(&"node-2".into()).unwrap();
        let updated_content2 = binding.content();
        assert!(updated_content2.contains("UPDATED"));
        let binding = cache.retrieve(&"node-3".into()).unwrap();
        let updated_content3 = binding.content();
        assert!(updated_content3.contains("UPDATED"));
    }

    #[test]
    fn test_submit_with_new_node_id() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = OrgCache::new(temp_dir.path().to_path_buf());

        let org_content = r#":PROPERTIES:
:ID: existing-node
:END:
#+title: Test File

Content here.
"#;

        let org_file = create_test_org_file(temp_dir.path(), "test.org", org_content);

        // Submit for a node ID that doesn't exist in cache yet
        cache.submit("new-node-id".into(), &org_file).unwrap();

        // Verify the new node was added
        assert!(cache.lookup.contains_key(&"new-node-id".into()));

        // Verify content is correct
        let entry = cache.retrieve(&"new-node-id".into()).unwrap();
        assert!(entry.content().contains("existing-node"));
    }

    #[test]
    fn test_submit_different_files_dont_interfere() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = OrgCache::new(temp_dir.path().to_path_buf());

        // Create two different org files
        let org_content1 = r#":PROPERTIES:
:ID: file1-node
:END:
#+title: File 1

Content 1
"#;

        let org_content2 = r#":PROPERTIES:
:ID: file2-node
:END:
#+title: File 2

Content 2
"#;

        let org_file1 = create_test_org_file(temp_dir.path(), "test1.org", org_content1);
        let org_file2 = create_test_org_file(temp_dir.path(), "test2.org", org_content2);

        // Add entries for both files
        cache.submit("file1-node".into(), &org_file1).unwrap();
        cache.submit("file2-node".into(), &org_file2).unwrap();

        // Verify they point to different cache entries
        {
            let file1_ref = cache.lookup.get(&"file1-node".into()).unwrap();
            let file2_ref = cache.lookup.get(&"file2-node".into()).unwrap();
            assert_ne!(file1_ref.content(), file2_ref.content());
        }

        // Store old content for comparison
        let file1_old_content = cache
            .retrieve(&"file1-node".into())
            .unwrap()
            .content()
            .to_string();
        let file2_old_content = cache
            .retrieve(&"file2-node".into())
            .unwrap()
            .content()
            .to_string();

        // Update file1 content
        let org_content1_updated = r#":PROPERTIES:
:ID: file1-node
:END:
#+title: File 1 UPDATED

Content 1 UPDATED
"#;

        fs::write(&org_file1, org_content1_updated).unwrap();
        cache.submit("file1-node".into(), &org_file1).unwrap();

        // Verify file1 entry changed but file2 entry remained the same
        let file1_new_content = cache
            .retrieve(&"file1-node".into())
            .unwrap()
            .content()
            .to_string();
        let file2_same_content = cache
            .retrieve(&"file2-node".into())
            .unwrap()
            .content()
            .to_string();

        assert_ne!(file1_old_content, file1_new_content); // file1 changed
        assert_eq!(file2_old_content, file2_same_content); // file2 unchanged

        // Verify content
        assert!(file1_new_content.contains("UPDATED"));
        assert!(!file2_same_content.contains("UPDATED"));
    }

    #[test]
    fn test_submit_preserves_arc_sharing() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = OrgCache::new(temp_dir.path().to_path_buf());

        let org_content = r#":PROPERTIES:
:ID: node-1
:END:
#+title: Multi Node File

* Section A
:PROPERTIES:  
:ID: node-2
:END:

* Section B
:PROPERTIES:
:ID: node-3  
:END:
"#;

        let org_file = create_test_org_file(temp_dir.path(), "multi.org", org_content);

        // Submit multiple nodes from the same file
        cache.submit("node-1".into(), &org_file).unwrap();
        cache.submit("node-2".into(), &org_file).unwrap();
        cache.submit("node-3".into(), &org_file).unwrap();

        // All should share the same Arc
        let ptr1 = cache.lookup.get(&"node-1".into()).unwrap();
        let ptr2 = cache.lookup.get(&"node-2".into()).unwrap();
        let ptr3 = cache.lookup.get(&"node-3".into()).unwrap();

        assert_eq!(ptr1.content(), ptr2.content());
        assert_eq!(ptr2.content(), ptr3.content());

        // Verify Arc reference count (should be 3 - one for each lookup entry in the DashMap)
        // DashMap guards don't increment Arc reference count, they just hold references to the entries
        let arc_strong_count = Arc::strong_count(ptr1.value());
        assert_eq!(arc_strong_count, 3); // 3 entries in the map
    }
}
