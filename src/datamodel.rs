//! # Datamodel
//!

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;

#[derive(Debug, Default)]
pub struct GraphStore {
    nodes: HashMap<u64, Node>,
    files: HashMap<u64, File>,
    aliases: HashMap<u64, Alias>,
    links: HashMap<u64, Link>,
    tags: HashMap<u64, Tag>,
    embeds: HashMap<u64, Embedding>,
}

impl GraphStore {
    pub fn insert_node(&mut self, orgnode: NodeFromOrg) -> Result<()> {
        // Steps:
        // 1. file exists?
        // 2. tags exist?
        // 3. aliases exist?
        // 4. links exist?
        Ok(())
    }

    pub fn insert_file(&mut self, file: PathBuf, hash: String) -> Result<u64> {
        Ok(0)
    }

    // getters...
}

#[derive(Debug, Clone)]
pub struct Embedding {
    id: u64,
    data: Vec<f32>,
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct Node {
    id: u64,
    // TODO: UUID crate
    uuid: String,
    title: String,
    content: String,
    file: u64,
    level: u64,
    olp: Vec<String>,
    tags: Vec<u64>,
    aliases: Vec<u64>,
    timestamps: Timestamps,
    links: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct File {
    id: u64,
    path: PathBuf,
    hash: String,
}

#[derive(Debug, Clone)]
pub struct Alias {
    id: u64,
    text: String,
}

#[derive(Debug, Clone)]
pub struct Link {
    id: u64,
    src: u64,
    destination: LinkDest,
}

#[derive(Debug, Clone)]
pub enum LinkDest {
    File(String),
    Node(u64),
    Other(String),
}

#[derive(Debug, Clone)]
pub struct Timestamps {
    ctime: String,
    mtime: Vec<String>,
}

impl Timestamps {
    pub fn new(ctime: String, mtime: Vec<String>) -> Self {
        Self { ctime, mtime }
    }
}

#[derive(Debug, Clone)]
pub struct Tag {
    id: u64,
    content: String,
}
