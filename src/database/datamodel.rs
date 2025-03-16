//! # Datamodel
//!

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tantivy::{schema::{Field, Schema}, Document, TantivyDocument};

use crate::org::NodeFromOrg;

pub type Key = u64;

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
    cites: Vec<Key>,
    timestamps: Option<Timestamps>,
    links: Vec<Key>,
    refrs: Vec<Key>,
}

impl Node {
    pub fn from_orgnode(
        id: Key,
        org: NodeFromOrg,
        tags: Vec<u64>,
        links: Vec<u64>,
        refrs: Vec<u64>,
        aliases: Vec<u64>,
        cites: Vec<u64>,
        file_id: u64,
    ) -> Self {
        Self {
            id,
            uuid: org.uuid,
            title: org.title,
            content: org.content,
            file: file_id,
            level: org.level,
            olp: org.olp,
            cites,
            // TODO
            timestamps: None,
            tags,
            links,
            refrs,
            aliases,
        }
    }

    pub fn to_document(&self, schema: &Schema) -> Result<TantivyDocument> {
        let id = schema.get_field("id")?;
        let title = schema.get_field("title")?;
        let content = schema.get_field("content")?;
        let level = schema.get_field("level")?;

        let mut doc = TantivyDocument::default();
        doc.add_text(id, self.uuid.clone());
        doc.add_text(title, self.title.clone());
        doc.add_text(content, self.content.clone());
        doc.add_u64(level, self.level);

        Ok(doc)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    id: Key,
    cite: String,
}

impl Citation {
    pub fn new(id: Key, cite: String) -> Self {
        Self {
            id, cite
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
            id,
            refref,
            reftype,
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
        Self { id, path, hash }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    id: Key,
    text: String,
}

impl Alias {
    pub fn new(id: Key, text: String) -> Self {
        Self { id, text }
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
            id,
            src,
            destination: LinkDest::Node(dst),
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
        Self { id, content }
    }
}
