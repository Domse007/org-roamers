//! # Database

// TODO: Access control?
pub mod datamodel;
pub mod embeddings;
pub mod search;
pub mod store;

use std::{hash::Hash, path::PathBuf};

use anyhow::Result;
use datamodel::*;
use search::Search;
use store::Store;

use crate::org::NodeFromOrg;

pub struct Database {
    store: Store,
    search: Search
}

fn hash<T: Hash>(t: &T) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl Database {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        Ok(Self {
            store: Store::new(path.clone())?,
            search: Search::new(path)?,
        })
    }

    pub fn insert_node_deep(&self, node: NodeFromOrg, filehash: String) -> Result<()> {
        let file_id = hash(&node.file);
        // File doesn't exist yet
        // TODO: Create 'upsert'
        if let Ok(None) = self.store.get_file(file_id) {
            let file = File::with_hash(file_id, node.file.clone().into(), filehash);
            let _ = self.store.put_file(file_id, &file)?;
        }

        let tagids: Vec<(&String, u64)> = node.tags.iter().map(|t| (t, hash(&t))).collect();
        for (tag, tagid) in tagids.iter() {
            let tagid = *tagid;
            if let Ok(None) = self.store.get_tag(tagid) {
                let tag = Tag::new(tagid, tag.to_string());
                let _ = self.store.put_tag(tagid, &tag)?;
            }
        }
        let tagids: Vec<u64> = tagids.iter().map(|(_, id)| *id).collect();

        let refids: Vec<(&String, u64)> = node.refs.iter().map(|t| (t, hash(&t))).collect();
        for (refr, refid) in refids.iter() {
            let refid = *refid;
            if let Ok(None) = self.store.get_ref(refid) {
                // TODO: reftype
                let refr = Reference::new(refid, refr.to_string(), String::new());
                let _ = self.store.put_ref(refid, &refr)?;
            }
        }
        let refids: Vec<u64> = refids.iter().map(|(_, id)| *id).collect();

        let aliasids: Vec<(&String, u64)> = node.aliases.iter().map(|t| (t, hash(&t))).collect();
        for (alias, aliasid) in aliasids.iter() {
            let aliasid = *aliasid;
            if let Ok(None) = self.store.get_alias(aliasid) {
                let alias = Alias::new(aliasid, alias.to_string());
                let _ = self.store.put_alias(aliasid, &alias)?;
            }
        }
        let aliasids: Vec<u64> = aliasids.iter().map(|(_, id)| *id).collect();

        let linkids: Vec<(u64, u64, u64)> = node
            .links
            .iter()
            .map(|(src, dst)| {
                let link = format!("{src}-{dst}");
                (hash(src), hash(dst), hash(&link))
            })
            .collect();
        for (srcid, dstid, linkid) in linkids.iter() {
            let srcid = *srcid;
            let dstid = *dstid;
            let linkid = *linkid;
            if let Ok(None) = self.store.get_node(srcid) {
                // WARN: Source node ID doesn't exist
            }
            if let Ok(None) = self.store.get_node(dstid) {
                // WARN: Destination node ID doesn't exist
            }
            if let Ok(None) = self.store.get_link(linkid) {
                let link = Link::id_to_id(linkid, srcid, dstid);
                let _ = self.store.put_link(linkid, &link)?;
            }
        }
        let linkids: Vec<u64> = linkids.iter().map(|(_, _, id)| *id).collect();

        let citeids: Vec<(&String, u64)> = node.aliases.iter().map(|t| (t, hash(&t))).collect();
        for (cite, citeid) in citeids.iter() {
            let citeid = *citeid;
            if let Ok(None) = self.store.get_cite(citeid) {
                let cite = Citation::new(citeid, cite.to_string());
                let _ = self.store.put_cite(citeid, &cite)?;
            }
        }
        let citeids: Vec<u64> = citeids.iter().map(|(_, id)| *id).collect();

        let id = hash(&node.uuid);
        let node = Node::from_orgnode(id, node, tagids, linkids, refids, aliasids, citeids, file_id);
        let _ = self.store.put_node(id, &node)?;

        Ok(())
    }

    pub fn get_node(&self, key: Key) -> Result<Option<Node>> {
        self.store.get_node(key)
    }

    pub fn get_node_file(&self, node: &Node) -> Result<Option<File>> {
        self.store.get_file(node.file)
    }
}
