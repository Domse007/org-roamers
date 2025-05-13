use std::{ffi::OsStr, fs, path::Path};

use anyhow::Result;
use rusqlite::Connection;

use super::olp;
use crate::org;

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
pub fn init_nodes_table(con: &mut Connection) -> Result<()> {
    const STMNT: &'static str = concat!(
        "CREATE TABLE nodes (id NOT NULL PRIMARY KEY, file NOT NULL,",
        "level NOT NULL, pos NOT NULL, todo, priority, scheduled text,",
        "deadline text, title, properties, olp, actual_olp,",
        "FOREIGN KEY (file) REFERENCES files (file) ON DELETE CASCADE);"
    );
    con.execute(STMNT, [])?;
    Ok(())
}

pub fn init_links_table(con: &mut Connection) -> Result<()> {
    const STMNT: &str = concat!(
        "CREATE TABLE links (pos NOT NULL, source NOT NULL, dest NOT NULL,",
        "type NOT NULL, properties NOT NULL, FOREIGN KEY (source)",
        "REFERENCES nodes (id) ON DELETE CASCADE);"
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
    let stmnt_tags: &'static str = concat!(
        "CREATE TABLE tags (node_id NOT NULL, tag,",
        "FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE);"
    );
    let stmnt_index: &'static str = concat!("CREATE INDEX tags_node_id ON tags (node_id);");
    con.execute(stmnt_tags, [])?;
    con.execute(stmnt_index, [])?;
    Ok(())
}

pub fn insert_node(
    con: &mut Connection,
    with_replace: bool,
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
    actual_olp: &str,
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
            "deadline", "title", "properties", "olp", "actual_olp",
        ],
        [
            s(id).as_str(), s(file).as_str(), level.to_string().as_str(), pos.to_string().as_str(),
            if todo { "true" } else { "false" }, priority.to_string().as_str(),
            s(scheduled).as_str(), s(deadline).as_str(), s(title).as_str(), s(properties).as_str(),
            s(olp).as_str(), s(actual_olp).as_str(),
        ],
        with_replace
    )?;
    Ok(())
}

pub fn insert_tag(con: &mut Connection, id: &str, tag: &str, with_replace: bool) -> Result<()> {
    let id = format!("\"\"\"{id}\"\"\"");
    let tag = format!("\"{tag}\"");
    insert_row(con, "tags", ["node_id", "tag"], [&id, &tag], with_replace)?;
    Ok(())
}

pub fn insert_link(
    con: &mut Connection,
    source: &str,
    dest: &str,
    with_replace: bool,
) -> Result<()> {
    const TYPE: &str = "id";
    const PROPERTIES: &str = "";
    const POS: usize = 0;
    let s = |s| format!("\"{}\"", s);
    let quotify = |s| format!("\"\"\"{}\"\"\"", s);
    insert_row(
        con,
        "links",
        ["pos", "source", "dest", "type", "properties"],
        [
            POS.to_string().as_str(),
            quotify(source).as_str(),
            quotify(dest).as_str(),
            quotify(TYPE).as_str(),
            s(PROPERTIES).as_str(),
        ],
        with_replace,
    )?;
    Ok(())
}

pub fn insert_row<const I: usize>(
    con: &mut Connection,
    table: &str,
    cols: [&str; I],
    vals: [&str; I],
    with_replace: bool,
) -> Result<()> {
    let stmnt = insert_row_formatter(table, cols, vals, with_replace);
    con.execute(&stmnt, [])?;
    Ok(())
}

fn insert_row_formatter<const I: usize>(
    table: &str,
    cols: [&str; I],
    vals: [&str; I],
    with_replace: bool,
) -> String {
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
        "INSERT {}INTO {table} {}\nVALUES {}",
        if with_replace { "OR REPLACE " } else { "" },
        formatter(cols),
        formatter(vals)
    )
}

#[derive(Default)]
pub(crate) struct IterFilesStats {
    pub num_files: usize,
    pub num_nodes: usize,
    pub num_links: usize,
    pub num_tags: usize,
}

pub(super) fn iter_files<P: AsRef<Path>>(
    con: &mut Connection,
    roam_path: P,
    stats: &mut IterFilesStats,
) -> Result<()> {
    let inc = |n: &mut usize| *n += 1;

    for entry in fs::read_dir(roam_path)? {
        let entry = if entry.is_ok() {
            entry.unwrap()
        } else {
            continue;
        };
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            iter_files(con, entry.path(), stats)?;
        }

        if metadata.is_file() && entry.path().extension() == Some(OsStr::new("org")) {
            inc(&mut stats.num_files);
            let nodes = org::get_nodes_from_file(entry.path())?;
            for node in nodes {
                inc(&mut stats.num_nodes);
                self::insert_node(
                    con,
                    false,
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
                    olp::into_olp_string(node.olp).as_str(),
                    olp::into_olp_string(node.actual_olp).as_str(),
                )?;
                for tag in node.tags {
                    inc(&mut stats.num_tags);
                    insert_tag(con, &node.uuid, &tag, false)?;
                }
                for link in node.links {
                    inc(&mut stats.num_links);
                    insert_link(con, &node.uuid, &link.0, false)?;
                }
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
            insert_row_formatter("nodes", cols, vals, false),
            concat!(
                "INSERT INTO nodes (id, file, title)\n",
                "VALUES (\"aa\", \"t.org\", \"test\")"
            )
        );
    }
    #[test]
    fn test_insert_row_formatter_with_replace() {
        let cols = ["id", "file", "title"];
        let vals = ["\"aa\"", "\"t.org\"", "\"test\""];
        assert_eq!(
            insert_row_formatter("nodes", cols, vals, true),
            concat!(
                "INSERT OR REPLACE INTO nodes (id, file, title)\n",
                "VALUES (\"aa\", \"t.org\", \"test\")"
            )
        );
    }
}
