use sqlx::SqlitePool;

pub async fn insert_olp(con: &SqlitePool, owner_id: &str, olp: &[String]) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO olp (node_id, position, segment)\n",
        "VALUES (?, ?, ?);"
    );

    for (i, elem) in olp.iter().enumerate() {
        sqlx::query(STMNT)
            .bind(owner_id)
            .bind(i as u32)
            .bind(elem)
            .execute(con)
            .await?;
    }

    Ok(())
}

pub async fn get_olp(con: &SqlitePool, owner_id: &str) -> anyhow::Result<Vec<String>> {
    const STMNT: &str = concat!(
        "SELECT segment FROM olp\n",
        "WHERE node_id = ?\n",
        "ORDER BY position ASC;"
    );

    let olp: Vec<(String,)> = sqlx::query_as(STMNT).bind(owner_id).fetch_all(con).await?;

    Ok(olp.into_iter().map(|e| e.0).collect())
}
