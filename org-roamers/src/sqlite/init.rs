use sqlx::{Executor, SqlitePool};

pub use super::files::init_files_table;

/// If the table is constructed by org-roamers, actual_olp is added to the
/// table, to simplify the graph construction, because org-roam by default does
/// not consider the toplevel node. Therefore if we have:
///
/// ```org
/// #+title: Maintitle         (with id)
/// * subtitle                 (with id)
/// ```
///
/// The reference org-roam implementation constructs no olp, while actual_olp
/// generates `("Maintitle")`.
pub async fn init_nodes_table(con: &SqlitePool) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "CREATE TABLE nodes (id NOT NULL PRIMARY KEY, file NOT NULL, ",
        "level NOT NULL, todo, priority, scheduled text, ",
        "deadline text, title, properties, ",
        "FOREIGN KEY (file) REFERENCES files (file) ON DELETE CASCADE);"
    );
    con.execute(STMNT).await?;
    Ok(())
}

pub async fn init_links_table(con: &SqlitePool) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "CREATE TABLE links (pos NOT NULL, source NOT NULL, dest NOT NULL,",
        "type NOT NULL, properties NOT NULL, FOREIGN KEY (source)",
        "REFERENCES nodes (id) ON DELETE CASCADE);"
    );
    con.execute(STMNT).await?;
    Ok(())
}

pub async fn init_aliases(con: &SqlitePool) -> anyhow::Result<()> {
    const STMNT_ALIASES: &str = concat!(
        "CREATE TABLE aliases (node_id NOT NULL, alias,",
        "FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE);"
    );
    const STMNT_INDEX: &str = concat!("CREATE INDEX alias_node_id ON aliases (node_id );");
    con.execute(STMNT_ALIASES).await?;
    con.execute(STMNT_INDEX).await?;
    Ok(())
}

pub async fn init_tags(con: &SqlitePool) -> anyhow::Result<()> {
    let stmnt_tags: &'static str = concat!(
        "CREATE TABLE tags (node_id NOT NULL, tag,",
        "FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE);"
    );
    let stmnt_index: &'static str = concat!("CREATE INDEX tags_node_id ON tags (node_id);");
    let stmnt_tag_index: &'static str =
        concat!("CREATE INDEX tags_tag_node ON tags (tag, node_id);");
    con.execute(stmnt_tags).await?;
    con.execute(stmnt_index).await?;
    con.execute(stmnt_tag_index).await?;
    Ok(())
}

pub async fn init_olp_table(con: &SqlitePool) -> anyhow::Result<()> {
    const OLP: &str = concat!(
        "CREATE TABLE olp (\n",
        "    node_id TEXT NOT NULL,\n",
        "    position INTEGER NOT NULL,\n",
        "    segment TEXT NOT NULL,\n",
        "    PRIMARY KEY (node_id, position),\n",
        "    FOREIGN KEY (node_id) REFERENCES nodes(id)\n",
        "        ON DELETE CASCADE\n",
        "        ON UPDATE CASCADE\n",
        ");"
    );

    con.execute(OLP).await?;
    Ok(())
}
