use rusqlite::{params, Connection};

use crate::sqlite::olp;

// TODO: remove file. This also requires updating the table def.
#[allow(clippy::too_many_arguments)]
pub fn insert_node(
    con: &mut Connection,
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
        "INSERT OR REPLACE INTO nodes (id, file, level, todo, priority, scheduled, deadline, title)\n",
        "VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);"
    );
    con.execute(
        STMNT,
        params![id, file, level, todo, priority, scheduled, deadline, title],
    )?;

    olp::insert_olp(con, id, olp)?;

    Ok(())
}

pub fn insert_tag(con: &mut Connection, id: &str, tag: &str) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO tags (node_id, tag)\n",
        "VALUES (?1, ?2);"
    );
    con.execute(STMNT, params![id, tag])?;
    Ok(())
}

pub fn insert_link(con: &mut Connection, source: &str, dest: &str) -> anyhow::Result<()> {
    const TYPE: &str = "id";
    const PROPERTIES: &str = "";
    const POS: usize = 0;
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO links (pos, source, dest, type, properties)\n",
        "VALUES (?1, ?2, ?3, ?4, ?5);"
    );
    con.execute(STMNT, params![POS, source, dest, TYPE, PROPERTIES])?;
    Ok(())
}
