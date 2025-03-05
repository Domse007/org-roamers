//! # Datamodel
//!

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub type Key = u64;

#[derive(Debug, Clone, PartialEq)]
pub struct NodeFromOrg {
    pub(crate) uuid: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) file: String,
    pub(crate) level: u64,
    pub(crate) olp: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) aliases: Vec<String>,
    pub(crate) timestamps: Timestamps,
    pub(crate) links: Vec<(String, String)>,
    pub(crate) refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    id: Key,
    // TODO: UUID crate
    uuid: String,
    title: String,
    content: String,
    pub file: Key,
    level: Key,
    olp: Vec<String>,
    tags: Vec<Key>,
    aliases: Vec<Key>,
    timestamps: Timestamps,
    links: Vec<Key>,
    refrs: Vec<Key>,
}

impl Node {
    pub fn from_orgnode(id: Key, org: NodeFromOrg, tags: Vec<u64>, links: Vec<u64>, refrs: Vec<u64>, aliases: Vec<u64>, fileId: u64) -> Self {
        Self {
            id,
            uuid: org.uuid,
            title: org.title,
            content: org.content,
            file: fileId,
            level: org.level,
            olp: org.olp,
            timestamps: org.timestamps,
            tags,
            links,
            refrs,
            aliases
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    id: Key,
    refref: String,
    // TODO: Maybe enumify
    reftype: String,
}

impl Reference {
    pub fn new(id: Key, refref: String, reftype: String) -> Self {
        Self {
            id, refref, reftype
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    id: Key,
    path: PathBuf,
    hash: String,
}

impl File {
    pub fn with_hash(id: Key, path: PathBuf, hash: String) -> Self {
        Self {
            id, path, hash
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    id: Key,
    text: String,
}

impl Alias {
    pub fn new(id: Key, text: String) -> Self {
        Self {
            id, text
        }
    }
}

// TODO: Link description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    id: Key,
    src: Key,
    destination: LinkDest,
}

impl Link {
    // TODO: Other link types
    pub fn id_to_id(id: Key, src: Key, dst: Key) -> Self {
        Self {
            id, src, destination: LinkDest::Node(dst)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkDest {
    File(String),
    Node(Key),
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Timestamps {
    ctime: String,
    mtime: Vec<String>,
}

impl Timestamps {
    pub fn new(ctime: String, mtime: Vec<String>) -> Self {
        Self { ctime, mtime }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    id: Key,
    content: String,
}

impl Tag {
    pub fn new(id: Key, content: String) -> Self {
        Self {
            id, content
        }
    }
}
