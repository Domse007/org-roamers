mod logger;

emacs::plugin_is_GPL_compatible!();

pub use logger::{Logger, StdOutLogger};
use serde::Serialize;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;

use std::sync::{Mutex, OnceLock};

use emacs::{defun, Env, Result};
use tantivy::{doc, Index, IndexWriter};
use tantivy::{schema::*, DocAddress, Score};
use tempfile::TempDir;

struct Global {
    schema: Schema,
    tempdir: TempDir,
    index_writer: IndexWriter,
    index: Index,
}

const INDEX_WRITER_SIZE: usize = 50_000_000;

static DB: Mutex<Option<Global>> = Mutex::new(None);

pub fn init_db(logger: impl Logger) -> Result<()> {
    let index_path = TempDir::new()?;

    log!(
        logger,
        "Creating a new temp dir in {:?}",
        index_path.path().as_os_str()
    );

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("id", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);

    let schema = schema_builder.build();

    let index = Index::create_in_dir(&index_path, schema.clone())?;

    let index_writer: IndexWriter = index.writer(INDEX_WRITER_SIZE)?;

    let db = &DB;
    let mut access = db.lock().unwrap();
    *access = Some(Global {
        tempdir: index_path,
        index_writer,
        index,
        schema,
    });

    log!(logger, "Finished initializing DB.");

    Ok(())
}

#[emacs::module(name = "org-roam-utils")]
pub fn init(_: &Env) -> Result<()> {
    Ok(())
}

//TODO: use path.
#[defun]
pub fn prepare(logger: &Env, path: String) -> Result<()> {
    if DB.lock().unwrap().is_none() {
        return init_db(logger);
    }

    Ok(())
}

#[defun]
pub fn add_node(logger: &Env, title: String, id: String, body: String) -> Result<()> {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let title_field = db.schema.get_field("title").unwrap();
    let id_field = db.schema.get_field("id").unwrap();
    let body_field = db.schema.get_field("body").unwrap();

    let mut document = TantivyDocument::new();
    document.add_text(title_field, title.as_str());
    document.add_text(id_field, id);
    document.add_text(body_field, body);

    db.index_writer.add_document(document)?;

    log!(logger, "Written document: {}.", &title);

    db.index_writer.commit()?;

    Ok(())
}

#[derive(Serialize)]
pub struct GetNodesResult {
    id: String,
    title: String,
}

#[derive(Serialize)]
pub struct GetNodesResultWrapper {
    results: Vec<GetNodesResult>,
}

#[defun]
pub fn get_nodes(_logger: &Env, search: String, num_results: usize) -> Result<String> {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    let reader = db.index.reader()?;

    let searcher = reader.searcher();

    let title_field = db.schema.get_field("title").unwrap();
    let id_field = db.schema.get_field("id").unwrap();
    let body_field = db.schema.get_field("body").unwrap();

    let query_parser = QueryParser::for_index(&db.index, vec![title_field, id_field, body_field]);

    let query = query_parser.parse_query(search.as_str())?;

    let top_docs: Vec<(Score, DocAddress)> =
        searcher.search(&query, &TopDocs::with_limit(num_results))?;

    let mut results = Vec::with_capacity(num_results);

    for (_score, address) in top_docs {
        let retrieved_doc = searcher.doc::<TantivyDocument>(address)?;
        let title = retrieved_doc
            .get_first(title_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let id = retrieved_doc
            .get_first(id_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        results.push(GetNodesResult { id, title })
    }

    Ok(serde_json::to_string(&GetNodesResultWrapper { results })?)
}
