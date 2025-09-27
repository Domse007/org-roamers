use std::path::Path;

use rusqlite::{params, Connection, Transaction};

pub fn init_files_table(con: &mut Connection) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "CREATE TABLE files (id INTEGER PRIMARY KEY AUTOINCREMENT, ",
        "file TEXT NOT NULL, hash INTEGER NOT NULL);"
    );
    con.execute(STMNT, [])?;
    Ok(())
}

pub fn insert_file<P: AsRef<Path>>(
    con: &mut Connection,
    filename: P,
    hash: u64,
) -> anyhow::Result<usize> {
    let filename = filename.as_ref().to_string_lossy();
    let hash = hash as u32;

    const STMNT: &str = r#"INSERT OR REPLACE INTO files (file, hash) VALUES (?1, ?2);"#;
    con.execute(STMNT, params![filename, hash])
        .map_err(Into::into)
}

/// Transaction-aware version of insert_file
pub fn insert_file_tx<P: AsRef<Path>>(
    tx: &Transaction,
    filename: P,
    hash: u64,
) -> anyhow::Result<usize> {
    let filename = filename.as_ref().to_string_lossy();
    let hash = hash as u32;

    const STMNT: &str = r#"INSERT OR REPLACE INTO files (file, hash) VALUES (?1, ?2);"#;
    tx.execute(STMNT, params![filename, hash])
        .map_err(Into::into)
}

pub fn get_hash<P: AsRef<Path>>(con: &mut Connection, filename: P) -> anyhow::Result<u32> {
    let filename = filename.as_ref().to_string_lossy();
    const STMNT: &str = r#"SELECT hash FROM files WHERE file = ?1"#;
    con.query_row(STMNT, params![filename], |row| {
        Ok(row.get_unwrap::<usize, u32>(0))
    })
    .map_err(Into::into)
}
