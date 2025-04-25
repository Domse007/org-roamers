//! # Datastore

use std::{fmt, marker::PhantomData, path::PathBuf};

use super::{datamodel::{Alias, File, Link, Node, Reference, Tag}, Citation};
use anyhow::Result;
// use arroy::{distances::Euclidean, Database as ArroyDB, Reader, Writer};
use heed::{byteorder::NativeEndian, Database, Env, EnvOpenOptions};
use heed_types::*;

type NodeDB = Database<U64<NativeEndian>, SerdeBincode<Node>>;
type FileDB = Database<U64<NativeEndian>, SerdeBincode<File>>;
type LinkDB = Database<U64<NativeEndian>, SerdeBincode<Link>>;
type TagDB = Database<U64<NativeEndian>, SerdeBincode<Tag>>;
type AliasDB = Database<U64<NativeEndian>, SerdeBincode<Alias>>;
type RefDB = Database<U64<NativeEndian>, SerdeBincode<Reference>>;
type CiteDB = Database<U64<NativeEndian>, SerdeBincode<Citation>>;


pub struct Store {
    env: Env,
    nodes: NodeDB,
    files: FileDB,
    links: LinkDB,
    tags: TagDB,
    aliases: AliasDB,
    references: RefDB,
    cites: CiteDB,
}

impl Store {
    // TODO: Path
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let path = match path {
            Some(path) => path,
            None => {
                // This is probably very bad & unsafe (maybe?) but because mmap its also maybe fine
                let temp = tempfile::tempdir()?;
                temp.path().to_path_buf()
            },
        };
        let env = unsafe { EnvOpenOptions::new().max_dbs(16).open(path)? };

        let mut wtx = env.write_txn()?;
        let nodes: NodeDB =
            env.create_database(&mut wtx, Some("nodes"))?;
        let files: FileDB =
            env.create_database(&mut wtx, Some("links"))?;
        let links: LinkDB =
            env.create_database(&mut wtx, Some("files"))?;
        let tags: TagDB =
            env.create_database(&mut wtx, Some("aliases"))?;
        let aliases: AliasDB =
            env.create_database(&mut wtx, Some("tags"))?;
        let references: RefDB =
            env.create_database(&mut wtx, Some("refs"))?;
        let cites: CiteDB = env.create_database(&mut wtx, Some("cites"))?;

        // TODO: Arroy maybe version mismatch? Wrong return type from create_db
        // let embeds: ArroyDB<Euclidean> = env.create_database(&mut wtx, Some("embeds"))?;

        wtx.commit()?;

        Ok(Self {
            env,
            nodes,
            files,
            links,
            tags,
            aliases,
            references,
            cites,
        })
    }

    pub fn get_node(&self, key: u64) -> Result<Option<Node>> {
        let rtx = self.env.read_txn()?;
        let node = self.nodes.get(&rtx, &key)?;
        rtx.commit()?;
        // TODO: Error handling?
        Ok(node)
    }

    pub fn put_node(&self, key: u64, node: &Node) -> Result<()> {
        let mut wtx = self.env.write_txn()?;
        let _ = self.nodes.put(&mut wtx, &key, node)?;
        let _ = wtx.commit()?;
        Ok(())
    }

    pub fn del_node(&self, key: u64) -> Result<bool> {
        let mut wtx = self.env.write_txn()?;
        let r = self.nodes.delete(&mut wtx, &key)?;
        let _ = wtx.commit()?;
        Ok(r)
    }

    pub fn get_file(&self, key: u64) -> Result<Option<File>> {
        let rtx = self.env.read_txn()?;
        let file: Option<File> = self.files.get(&rtx, &key)?;
        rtx.commit()?;
        Ok(file)
    }

    pub fn put_file(&self, key: u64, file: &File) -> Result<()> {
        let mut wtx = self.env.write_txn()?;
        let _ = self.files.put(&mut wtx, &key, file)?;
        let _ = wtx.commit()?;
        Ok(())
    }

    pub fn del_file(&self, key: u64) -> Result<bool> {
        let mut wtx = self.env.write_txn()?;
        let r = self.files.delete(&mut wtx, &key)?;
        let _ = wtx.commit()?;
        Ok(r)
    }

    pub fn get_link(&self, key: u64) -> Result<Option<Link>> {
        let rtx = self.env.read_txn()?;
        let link: Option<Link> = self.links.get(&rtx, &key)?;
        rtx.commit()?;
        Ok(link)
    }

    pub fn put_link(&self, key: u64, link: &Link) -> Result<()> {
        let mut wtx = self.env.write_txn()?;
        let _ = self.links.put(&mut wtx, &key, link)?;
        let _ = wtx.commit()?;
        Ok(())
    }

    pub fn del_link(&self, key: u64) -> Result<bool> {
        let mut wtx = self.env.write_txn()?;
        let r = self.links.delete(&mut wtx, &key)?;
        let _ = wtx.commit()?;
        Ok(r)
    }

    pub fn get_alias(&self, key: u64) -> Result<Option<Alias>> {
        let rtx = self.env.read_txn()?;
        let alias: Option<Alias> = self.aliases.get(&rtx, &key)?;
        rtx.commit()?;
        Ok(alias)
    }

    pub fn put_alias(&self, key: u64, alias: &Alias) -> Result<()> {
        let mut wtx = self.env.write_txn()?;
        let _ = self.aliases.put(&mut wtx, &key, alias)?;
        let _ = wtx.commit()?;
        Ok(())
    }

    pub fn del_alias(&self, key: u64) -> Result<bool> {
        let mut wtx = self.env.write_txn()?;
        let r = self.aliases.delete(&mut wtx, &key)?;
        let _ = wtx.commit()?;
        Ok(r)
    }

    pub fn get_tag(&self, key: u64) -> Result<Option<Tag>> {
        let rtx = self.env.read_txn()?;
        let tag: Option<Tag> = self.tags.get(&rtx, &key)?;
        rtx.commit()?;
        Ok(tag)
    }

    pub fn put_tag(&self, key: u64, tag: &Tag) -> Result<()> {
        let mut wtx = self.env.write_txn()?;
        let _ = self.tags.put(&mut wtx, &key, tag)?;
        let _ = wtx.commit()?;
        Ok(())
    }

    pub fn del_tag(&self, key: u64) -> Result<bool> {
        let mut wtx = self.env.write_txn()?;
        let r = self.tags.delete(&mut wtx, &key)?;
        let _ = wtx.commit()?;
        Ok(r)
    }

    pub fn get_ref(&self, key: u64) -> Result<Option<Reference>> {
        let rtx = self.env.read_txn()?;
        let refer: Option<Reference> = self.references.get(&rtx, &key)?;
        rtx.commit()?;
        Ok(refer)
    }

    pub fn put_ref(&self, key: u64, refr: &Reference) -> Result<()> {
        let mut wtx = self.env.write_txn()?;
        let _ = self.references.put(&mut wtx, &key, refr)?;
        let _ = wtx.commit()?;
        Ok(())
    }

    pub fn del_ref(&self, key: u64) -> Result<bool> {
        let mut wtx = self.env.write_txn()?;
        let r = self.references.delete(&mut wtx, &key)?;
        let _ = wtx.commit()?;
        Ok(r)
    }

    pub fn get_cite(&self, key: u64) -> Result<Option<Citation>> {
        let rtx = self.env.read_txn()?;
        let refer: Option<Citation> = self.cites.get(&rtx, &key)?;
        rtx.commit()?;
        Ok(refer)
    }

    pub fn put_cite(&self, key: u64, cite: &Citation) -> Result<()> {
        let mut wtx = self.env.write_txn()?;
        let _ = self.cites.put(&mut wtx, &key, cite)?;
        let _ = wtx.commit()?;
        Ok(())
    }

    pub fn del_cite(&self, key: u64) -> Result<bool> {
        let mut wtx = self.env.write_txn()?;
        let r = self.cites.delete(&mut wtx, &key)?;
        let _ = wtx.commit()?;
        Ok(r)
    }
}
