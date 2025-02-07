mod logger;

emacs::plugin_is_GPL_compatible!();

pub use logger::{Logger, StdOutLogger};

use std::sync::OnceLock;

use emacs::{Env, Result, defun};
use tantivy::schema::{*};
use tantivy::{doc, Index, IndexWriter};
use tempfile::TempDir;

struct Global {
    schema: Schema,
    tempdir: TempDir,
    index_writer: IndexWriter,
    index: Index,
}

const INDEX_WRITER_SIZE: usize = 50_000_000;

const DB: OnceLock<Global> = OnceLock::new();

pub fn init_db(logger: impl Logger) -> Result<()> {
    let index_path = TempDir::new()?;
    logger.log(format!(
        "Creating a new temp dir in {:?}",
        index_path.path().as_os_str()
    ))?;

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("id", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);

    let schema = schema_builder.build();

    let index = Index::create_in_dir(&index_path, schema.clone())?;

    let index_writer: IndexWriter = index.writer(INDEX_WRITER_SIZE)?;

    DB.get_or_init(move || Global {
        tempdir: index_path,
        index_writer,
        index,
	schema,
    });

    Ok(())
}

#[emacs::module(name = "org-roam-utils")]
pub fn init(env: &Env) -> Result<()> {
    init_db(env)
}

#[defun]
pub fn add_node(title: String, id: String, body: String) -> Result<()> {
    let db = DB;
    let db = db.get().unwrap();

    let title_field = db.schema.get_field("title").unwrap();
    let id_field = db.schema.get_field("id").unwrap();
    let body_field = db.schema.get_field("body").unwrap();

    let mut document = TantivyDocument::new();
    document.add_text(title_field, title);
    document.add_text(id_field, id);
    document.add_text(body_field, body);

    db.index_writer.add_document(document)?;

    Ok(())
}
