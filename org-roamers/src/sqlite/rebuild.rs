use sqlx::SqlitePool;

use crate::sqlite::olp;

// TODO: remove file. This also requires updating the table def.
#[allow(clippy::too_many_arguments)]
pub async fn insert_node(
    con: &SqlitePool,
    id: &str,
    file: &str,
    level: u64,
    todo: bool,
    priority: usize,
    scheduled: &str,
    deadline: &str,
    title: &str,
    olp: &[String],
) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO nodes (id, file, level, todo, priority, scheduled, deadline, title, properties)\n",
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);"
    );

    sqlx::query(STMNT)
        .bind(id)
        .bind(file)
        .bind(level as u32)
        .bind(todo)
        .bind(priority as u32)
        .bind(scheduled)
        .bind(deadline)
        .bind(title)
        .bind(Option::<String>::None) // properties - not currently used
        .execute(con)
        .await?;

    olp::insert_olp(con, id, olp).await?;

    Ok(())
}

pub async fn insert_tag(con: &SqlitePool, id: &str, tag: &str) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO tags (node_id, tag)\n",
        "VALUES (?, ?);"
    );
    sqlx::query(STMNT).bind(id).bind(tag).execute(con).await?;
    Ok(())
}

pub async fn insert_alias(con: &SqlitePool, id: &str, alias: &str) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO aliases (node_id, alias)\n",
        "VALUES (?, ?);"
    );
    sqlx::query(STMNT).bind(id).bind(alias).execute(con).await?;
    Ok(())
}

pub async fn insert_link(con: &SqlitePool, source: &str, dest: &str) -> anyhow::Result<()> {
    const TYPE: &str = "id";
    const PROPERTIES: &str = "";
    const POS: u32 = 0;
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO links (pos, source, dest, type, properties)\n",
        "VALUES (?, ?, ?, ?, ?);"
    );
    sqlx::query(STMNT)
        .bind(POS)
        .bind(source)
        .bind(dest)
        .bind(TYPE)
        .bind(PROPERTIES)
        .execute(con)
        .await?;
    Ok(())
}
