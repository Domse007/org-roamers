use std::path::Path;

use sqlx::{Executor, SqlitePool};

pub async fn init_files_table(con: &SqlitePool) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "CREATE TABLE files (id INTEGER PRIMARY KEY AUTOINCREMENT, ",
        "file TEXT NOT NULL UNIQUE, hash INTEGER NOT NULL);"
    );
    con.execute(STMNT).await?;
    Ok(())
}

pub async fn insert_file<P: AsRef<Path>>(
    con: &SqlitePool,
    filename: P,
    hash: u64,
) -> anyhow::Result<()> {
    let filename = filename.as_ref().to_string_lossy();
    let hash = hash as u32;

    let _ = sqlx::query("INSERT OR REPLACE INTO files (file, hash) VALUES (?, ?);")
        .bind(filename)
        .bind(hash)
        .execute(con)
        .await?;

    Ok(())
}
