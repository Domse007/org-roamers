use rusqlite::{params, Connection};

pub fn insert_olp(con: &mut Connection, owner_id: &str, olp: &[String]) -> anyhow::Result<()> {
    const STMNT: &str = concat!(
        "INSERT OR REPLACE INTO olp (node_id, position, segment)\n",
        "VALUES (?1, ?2, ?3);"
    );

    let mut stmnt = con.prepare(STMNT)?;

    for (i, elem) in olp.iter().enumerate() {
        stmnt.execute(params![owner_id, i, elem])?;
    }

    Ok(())
}

pub fn get_olp(con: &mut Connection, owner_id: &str) -> anyhow::Result<Vec<String>> {
    const STMNT: &str = concat!(
        "SELECT node_id, position, segment FROM olp\n",
        "WHERE node_id = ?1\n",
        "ORDER BY position ASC;"
    );

    let mut stmnt = con.prepare(STMNT)?;

    let res = stmnt
        .query_map(params![owner_id], |row| {
            Ok(row.get_unwrap::<usize, String>(2))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();

    Ok(res)
}
