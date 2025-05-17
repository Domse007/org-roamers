//! org-roamers is in first and for all a server binary that can create the
//! same graph as [org-roam](https://github.com/org-roam/org-roam). It also has
//! routines for a clone of [org-roam-ui](https://github.com/org-roam/org-roam-ui)
//! latex previews and a lot more.
//!
//! <div class="warning">
//! org-roamers is split into a lib/bin architecture to enable customization of
//! the server. This crate most likely is only useful if some server feature
//! does not fit your org-roam usage. Otherwise just use the supplied server.
//! </div>
//!
//! See: the provided server implementation `org_roamers::bin::server::main.rs`.

pub mod api;
#[allow(warnings)]
pub mod database;
pub mod error;
mod latex;
pub mod parser;
mod perf;
pub mod search;
pub mod server;
pub mod sqlite;
pub mod transform;
pub mod watcher;

use serde::Serialize;
use sqlite::SqliteConnection;
use tempfile::TempDir;
use tracing::info;
use transform::export::HtmlExportSettings;

use std::error::Error;
use std::fs;
use std::path::Path;

use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter};

pub struct ServerState {
    // TODO: use tantivy as a search backend.
    _tempdir: TempDir,
    _schema: Schema,
    _index_writer: IndexWriter,
    _index: Index,

    pub sqlite: SqliteConnection,
    pub html_export_settings: HtmlExportSettings,
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

pub fn prepare_internal<P: AsRef<Path>>(
    path: &str,
    sqlite_db_path: Option<&str>,
    html_export_settings_path: P,
) -> Result<ServerState, Box<dyn std::error::Error>> {
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
        Ok(con) => con,
        Err(e) => {
            return Err(format!("ERROR: could not initialize the sqlite connection: {e}").into())
        }
    };

    Ok(ServerState {
        _tempdir: tempdir,
        _index_writer: indexwriter,
        _index: index,
        _schema: schema,
        sqlite: sqlite_con,
        html_export_settings: HtmlExportSettings::new(html_export_settings_path)
            .unwrap_or_default(),
    })
}

// fn add_node_internal(
//     db: &mut ServerState,
//     title: String,
//     id: String,
//     body: String,
//     file: String,
// ) -> Result<(), Box<dyn Error>> {
//     let title_field = db._schema.get_field("title").unwrap();
//     let id_field = db._schema.get_field("id").unwrap();
//     let body_field = db._schema.get_field("body").unwrap();
//     let file_field = db._schema.get_field("file").unwrap();

//     let mut document = TantivyDocument::new();
//     document.add_text(title_field, title.as_str());
//     document.add_text(id_field, id);
//     document.add_text(body_field, body);
//     document.add_text(file_field, file);

//     db._index_writer.add_document(document)?;

//     info!("Written document: {}.", &title);

//     db._index_writer.commit()?;

//     Ok(())
// }

#[derive(Serialize)]
pub struct GetNodesResult {
    id: String,
    title: String,
}

// #[derive(Serialize)]
// pub struct GetNodesResultWrapper {
//     results: Vec<GetNodesResult>,
// }

// fn get_nodes_internal(
//     db: &mut ServerState,
//     search: String,
//     num_results: usize,
// ) -> Result<GetNodesResultWrapper, Box<dyn Error>> {
//     let reader = db.index.reader()?;

//     let searcher = reader.searcher();

//     let title_field = db.schema.get_field("title").unwrap();
//     let id_field = db.schema.get_field("id").unwrap();
//     let body_field = db.schema.get_field("body").unwrap();

//     let query_parser = QueryParser::for_index(&db.index, vec![title_field, id_field, body_field]);

//     let query = query_parser.parse_query(search.as_str())?;

//     let top_docs: Vec<(Score, DocAddress)> =
//         searcher.search(&query, &TopDocs::with_limit(num_results))?;

//     let mut results = Vec::with_capacity(num_results);

//     for (_score, address) in top_docs {
//         let retrieved_doc = searcher.doc::<TantivyDocument>(address)?;
//         let title = retrieved_doc
//             .get_first(title_field)
//             .unwrap()
//             .as_str()
//             .unwrap()
//             .to_string();
//         let id = retrieved_doc
//             .get_first(id_field)
//             .unwrap()
//             .as_str()
//             .unwrap()
//             .to_string();

//         results.push(GetNodesResult { id, title })
//     }

//     Ok(GetNodesResultWrapper { results })
// }
