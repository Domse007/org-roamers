pub mod api;
pub mod database;
mod export;
mod latex;
mod migrate;
pub mod org;
pub mod parser;
pub mod server;
pub mod sqlite;

use serde::Serialize;
use sqlite::SqliteConnection;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tempfile::TempDir;
use tracing::info;

use std::error::Error;
use std::fs;
use std::path::Path;

use tantivy::{doc, Index, IndexWriter};
use tantivy::{schema::*, DocAddress, Score};

pub struct Global {
    _tempdir: TempDir,
    schema: Schema,
    index_writer: IndexWriter,
    index: Index,

    sqlite: SqliteConnection,
}

const INDEX_WRITER_SIZE: usize = 50_000_000;

pub fn init_tantivy(
    path: Option<&Path>,
) -> Result<(TempDir, Schema, IndexWriter, Index), Box<dyn Error>> {
    info!("Working in {}", std::env::current_dir().unwrap().display());

    let index_path = match path {
        Some(path) => TempDir::new_in(path),
        None => TempDir::new(),
    }?;

    info!(
        "Creating a new temp dir in {:?}",
        index_path.path().as_os_str()
    );

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("id", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT | STORED);
    schema_builder.add_text_field("file", TEXT | STORED);

    let schema = schema_builder.build();

    let index = Index::create_in_dir(&index_path, schema.clone())?;

    let index_writer: IndexWriter = index.writer(INDEX_WRITER_SIZE)?;

    info!("Finished initializing DB.");

    Ok((index_path, schema, index_writer, index))
}

pub fn prepare_internal(
    path: &str,
    sqlite_db_path: &str,
) -> Result<Global, Box<dyn std::error::Error>> {
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

    let (tempdir, schema, indexwriter, index) = match init_tantivy(path) {
        Ok(env) => env,
        Err(err) => return Err(format!("ERROR: could not initialize tantivy: {:?}", err).into()),
    };
    let sqlite_con = match SqliteConnection::init(sqlite_db_path) {
        Some(con) => con,
        None => return Err("ERROR: could not initialize the sqlite connection".into()),
    };

    Ok(Global {
        _tempdir: tempdir,
        index_writer: indexwriter,
        index,
        schema,
        sqlite: sqlite_con,
    })
}

fn add_node_internal(
    db: &mut Global,
    title: String,
    id: String,
    body: String,
    file: String,
) -> Result<(), Box<dyn Error>> {
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

    info!("Written document: {}.", &title);

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

fn get_nodes_internal(
    db: &mut Global,
    search: String,
    num_results: usize,
) -> Result<GetNodesResultWrapper, Box<dyn Error>> {
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
