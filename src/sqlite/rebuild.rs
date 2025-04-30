use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::Path,
};

use anyhow::Result;
use rusqlite::{Connection, Statement};

use crate::org;

use super::SqliteConnection;

pub fn init_version(con: &mut Connection, version: usize) -> Result<()> {
    con.execute(format!("PRAGMA user_version = {}", version).as_str(), [])?;
    Ok(())
}

pub fn init_files_table(con: &mut Connection) -> Result<()> {
    const STMNT: &'static str = concat!(
        "CREATE TABLE files (file UNIQUE PRIMARY KEY, title,",
        "hash NOT NULL, atime NOT NULL, mtime NOT NULL);"
    );
    con.execute(STMNT, [])?;
    Ok(())
}

pub fn init_nodes_table(con: &mut Connection) -> Result<()> {
    const STMNT: &'static str = concat!(
        "CREATE TABLE nodes (id NOT NULL PRIMARY KEY, file NOT NULL,",
        "level NOT NULL, pos NOT NULL, todo, priority, scheduled text,",
        "deadline text, title, properties, olp,",
        "FOREIGN KEY (file) REFERENCES files (file) ON DELETE CASCADE);"
    );
    con.execute(STMNT, [])?;
    Ok(())
}

pub fn init_aliases(con: &mut Connection) -> Result<()> {
    const STMNT_ALIASES: &'static str = concat!(
        "CREATE TABLE aliases (node_id NOT NULL, alias,",
        "FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE);"
    );
    const STMNT_INDEX: &'static str = concat!("CREATE INDEX alias_node_id ON aliases (node_id );");
    con.execute(STMNT_ALIASES, [])?;
    con.execute(STMNT_INDEX, [])?;
    Ok(())
}

pub fn init_tags(con: &mut Connection) -> Result<()> {
    let STMNT_TAGS: &'static str = concat!(
        "CREATE TABLE tags (node_id NOT NULL, tag,",
        "FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE);"
    );
    let STMNT_INDEX: &'static str = concat!("CREATE INDEX tags_node_id ON tags (node_id);");
    con.execute(STMNT_TAGS, [])?;
    con.execute(STMNT_INDEX, [])?;
    Ok(())
}

pub fn insert_node(
    con: &mut Connection,
    id: &str,
    file: &str,
    level: u64,
    pos: usize,
    todo: bool,
    priority: usize,
    scheduled: &str,
    deadline: &str,
    title: &str,
    properties: &str,
    olp: &str,
) -> Result<()> {
    let s = |s: &str| {
        if s.is_empty() {
            "\"\"".to_string()
        } else {
            format!("\"\"\"{}\"\"\"", s)
        }
    };
    #[rustfmt::skip]
    insert_row(
        con,
        "nodes",
        [
            "id", "file", "level", "pos", "todo", "priority", "scheduled",
            "deadline", "title", "properties", "olp",
        ],
        [
            s(id).as_str(), s(file).as_str(), level.to_string().as_str(), pos.to_string().as_str(),
            if todo { "true" } else { "false" }, priority.to_string().as_str(),
            s(scheduled).as_str(), s(deadline).as_str(), s(title).as_str(), s(properties).as_str(), s(olp).as_str(),
        ],
    )?;
    Ok(())
}

pub fn insert_row<const I: usize>(
    con: &mut Connection,
    table: &str,
    cols: [&str; I],
    vals: [&str; I],
) -> Result<()> {
    let stmnt = insert_row_formatter(table, cols, vals);
    println!("{}", stmnt);
    con.execute(&stmnt, [])?;
    Ok(())
}

fn insert_row_formatter<const I: usize>(table: &str, cols: [&str; I], vals: [&str; I]) -> String {
    let formatter = |cols: [&str; I]| -> String {
        let mut s = String::new();
        let mut iter = cols.iter();
        s.push('(');
        s.push_str(iter.next().unwrap());
        for val in iter {
            s.push_str(", ");
            s.push_str(val);
        }
        s.push(')');
        s
    };
    format!(
        "INSERT INTO {table} {}\nVALUES {}",
        formatter(cols),
        formatter(vals)
    )
}

pub(super) fn iter_files<P: AsRef<Path>>(con: &mut Connection, roam_path: P) -> Result<()> {
    for entry in fs::read_dir(roam_path)? {
        let entry = if entry.is_ok() {
            entry.unwrap()
        } else {
            continue;
        };
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            iter_files(con, entry.path())?;
        }

        if metadata.is_file() && entry.path().extension() == Some(OsStr::new("org")) {
            let nodes = org::get_nodes_from_file(entry.path())?;
            for node in nodes {
                self::insert_node(
                    con,
                    &node.uuid,
                    node.file.as_str(),
                    node.level,
                    0,
                    false,
                    0,
                    "",
                    "",
                    node.title.as_str(),
                    "",
                    SqliteConnection::into_olp_string(node.olp).as_str(),
                )?;
                // TODO: add files. For this title is required, which is the
                // toplevel `#+title` tile.
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_insert_row_formatter() {
        let cols = ["id", "file", "title"];
        let vals = ["\"aa\"", "\"t.org\"", "\"test\""];
        assert_eq!(
            insert_row_formatter("nodes", cols, vals),
            concat!(
                "INSERT INTO nodes (id, file, title)\n",
                "VALUES (\"aa\", \"t.org\", \"test\")"
            )
        );
    }
}
