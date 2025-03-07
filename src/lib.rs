pub mod database;
mod latex;
mod logger;
mod migrate;
pub mod org;
pub mod parser;
mod server;
pub mod sqlite;

emacs::plugin_is_GPL_compatible!();

pub use logger::{Logger, StdOutLogger};
use serde::Serialize;
use sqlite::SqliteConnection;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tempfile::TempDir;

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use emacs::{defun, Env, Result};
use tantivy::{doc, schema, Index, IndexWriter};
use tantivy::{schema::*, DocAddress, Score};

struct Global {
    _tempdir: TempDir,
    schema: Schema,
    index_writer: IndexWriter,
    index: Index,

    sqlite: SqliteConnection,
}

const INDEX_WRITER_SIZE: usize = 50_000_000;

static DB: Mutex<Option<Global>> = Mutex::new(None);

pub fn init_tantivy(
    logger: impl Logger,
    path: Option<&Path>,
) -> Result<(TempDir, Schema, IndexWriter, Index)> {
    log!(logger, "{}", std::env::current_dir().unwrap().display());

    let index_path = match path {
        Some(path) => TempDir::new_in(path),
        None => TempDir::new(),
    }?;

    log!(
        logger,
        "Creating a new temp dir in {:?}",
        index_path.path().as_os_str()
    );

    let index = Index::create_in_dir(&index_path, schema.clone())?;

    let index_writer: IndexWriter = index.writer(INDEX_WRITER_SIZE)?;

    log!(logger, "Finished initializing DB.");

    Ok((index_path, schema, index_writer, index))
}

#[emacs::module(name = "org-roamers-utils")]
pub fn init(_: &Env) -> Result<()> {
    Ok(())
}

//TODO: use path.
#[defun]
pub fn prepare(logger: &Env, path: String, sqlite_db_path: String) -> emacs::Result<()> {
    let path = Path::new(&path);

    let path = if path.is_file() {
        None
    } else if !path.exists() {
        match fs::create_dir(path) {
            Ok(_) => Some(path),
            Err(_) => None,
        }
    } else {
        None
    };

    if DB.lock().unwrap().is_none() {
        let (tempdir, schema, indexwriter, index) = match init_tantivy(logger, path) {
            Ok(env) => env,
            Err(err) => {
                return Err(emacs::Error::msg(format!(
                    "ERROR: could not initialize tantivy: {:?}",
                    err
                )))
            }
        };
        let sqlite_con = match SqliteConnection::init(sqlite_db_path) {
            Some(con) => con,
            None => {
                return Err(emacs::Error::msg(
                    "ERROR: could not initialize the sqlite connection",
                ))
            }
        };

        let db = &DB;
        let mut access = db.lock().unwrap();
        *access = Some(Global {
            _tempdir: tempdir,
            index_writer: indexwriter,
            index,
            schema,
            sqlite: sqlite_con,
        });
    }

    Ok(())
}

#[defun]
pub fn add_node(logger: &Env, title: String, id: String, body: String, file: String) -> Result<()> {
    let db = &DB;
    let mut db = db.lock().unwrap();
    let db = db.as_mut().unwrap();

    add_node_internal(&logger, db, title, id, body, file)
}

fn add_node_internal(
    logger: &impl Logger,
    db: &mut Global,
    title: String,
    id: String,
    body: String,
    file: String,
) -> Result<()> {
    let title_field = db.schema.get_field("title").unwrap();
    let id_field = db.schema.get_field("id").unwrap();
    let body_field = db.schema.get_field("body").unwrap();
    let file_field = db.schema.get_field("file").unwrap();

    let mut document = TantivyDocument::new();
    document.add_text(title_field, title.as_str());
    document.add_text(id_field, id);
    document.add_text(body_field, body);
    document.add_text(file_field, file);

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
    let results = get_nodes_internal(_logger, search, num_results)?;

    Ok(serde_json::to_string(&results)?)
}

fn get_nodes_internal(
    _logger: impl Logger,
    search: String,
    num_results: usize,
) -> Result<GetNodesResultWrapper> {
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

    Ok(GetNodesResultWrapper { results })
}
