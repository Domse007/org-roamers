//! # Full Text Search
//!
//! Using Tantivy (meili would be ideal, but not easy to split into a standalone crate)

use std::path::PathBuf;
use std::sync::RwLock;

use anyhow::{bail, Result};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{doc, schema, Index, IndexReader, IndexWriter};
use tantivy::{schema::*, DocAddress, Score};

use super::Node;

pub struct Search {
    schema: Schema,
    index: Index,
    writer: RwLock<IndexWriter>,
    reader: IndexReader,
    fields: Vec<Field>,
}

impl Search {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let mut schema = Schema::builder();
        let id = schema.add_text_field("id", TEXT);
        let title = schema.add_text_field("title", TEXT);
        let content = schema.add_text_field("content", TEXT);
        // TODO: include filepath in node?
        // schema.add_facet_field("filepath", FacetOptions::default());
        schema.add_u64_field("level", STORED | INDEXED);

        let schema = schema.build();

        let index = match path {
            Some(path) => Index::create_in_dir(path, schema.clone())?,
            None => Index::create_in_ram(schema.clone()),
        };
        // TODO: Config options
        let writer: IndexWriter = index.writer(100_000)?;
        let writer = RwLock::new(writer);
        let reader = index.reader()?;

        let fields = vec![title, content];

        Ok (Self {
            schema, index, writer, reader,
            fields
        })
    }

    pub fn index_one(&self, node: Node) -> Result<()> {
        let doc = node.to_document(&self.schema)?;
        if let Ok(mut writer) = self.writer.write() {
            let _ = writer.add_document(doc)?;
            let _ = writer.commit()?;
        } else {
            bail!("unable to acquire lock, most likely poisoned");
        }
        Ok(())
    }

    pub fn search(&self, q: &str, limit: usize, fields: Option<Vec<Field>>) -> Result<Vec<DocAddress>> {
        let searcher = self.reader.searcher();
        let fields = match fields {
            Some(f) => f,
            None => self.fields.clone(),
        };
        let query = QueryParser::for_index(&self.index, fields);
        let query = query.parse_query(q)?;
        let collector = TopDocs::with_limit(limit);
        let results = searcher.search(&query, &collector)?;
        let out = results.into_iter().map(|(_, id)| id).collect();
        Ok(out)
    }
}
