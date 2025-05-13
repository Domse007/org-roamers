use std::path::Path;

use anyhow::{bail, Result};
use rebuild::IterFilesStats;
use rusqlite::{Connection, Params};
use tracing::info;

use crate::error::ServerError;

pub mod olp;
pub mod rebuild;

pub struct SqliteConnection {
    connection: Connection,
}

impl SqliteConnection {
    const MIN_VERSION: usize = 20;

    pub fn init<P: AsRef<Path>>(path: Option<P>) -> Result<Self> {
        let this = match path {
            Some(ref path) => Self {
                connection: Connection::open(path)?,
            },
            None => {
                info!("No path supplied. Building own db.");
                let mut connection = Connection::open_in_memory()?;
                rebuild::init_version(&mut connection, Self::MIN_VERSION)?;
                rebuild::init_files_table(&mut connection)?;
                rebuild::init_nodes_table(&mut connection)?;
                rebuild::init_links_table(&mut connection)?;
                rebuild::init_aliases(&mut connection)?;
                rebuild::init_tags(&mut connection)?;
                Self { connection }
            }
        };

        let version: usize = this
            .connection
            .pragma_query_value(None, "user_version", |row| Ok(row.get_unwrap(0)))?;

        if version != Self::MIN_VERSION {
            tracing::error!(
                "DB version does not match: {} (DB) != {} (required)",
                version,
                Self::MIN_VERSION
            );
            bail!(
                "Incompatible version: (MIN) {} != {} (SUPPLIED)",
                Self::MIN_VERSION,
                version
            );
        }

        Ok(this)
    }

    pub fn query_many<P, T, F>(
        &self,
        sql: &str,
        params: P,
        map_fn: F,
    ) -> Result<Vec<T>, ServerError>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
        P: Params,
    {
        let mut stmt = self.connection.prepare(sql)?;
        let rows = stmt.query_map(params, map_fn)?;
        rows.collect::<Result<Vec<T>, rusqlite::Error>>()
            .map_err(Into::into)
    }

    pub fn query_one<P, T, F>(&self, sql: &str, params: P, map_fn: F) -> Result<T, ServerError>
    where
        P: Params,
        F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        self.connection
            .query_row(sql, params, map_fn)
            .map_err(Into::into)
    }

    pub fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }

    pub fn insert_files<P: AsRef<Path>>(&mut self, roam_path: P) -> Result<()> {
        let mut stats = IterFilesStats::default();
        let res = rebuild::iter_files(&mut self.connection, roam_path, &mut stats);
        #[rustfmt::skip]
        tracing::info!(
            "Indexed {} nodes over {} files containing {} links and {} tags",
            stats.num_nodes, stats.num_files, stats.num_links, stats.num_tags
        );
        res
    }
}

pub mod helpers {
    use rusqlite::Connection;

    pub fn get_all_nodes<const PARAMS: usize>(
        con: &Connection,
        params: [&'static str; PARAMS],
    ) -> Vec<[String; PARAMS]> {
        let params = params.join(", ");
        let stmnt = format!("SELECT {} FROM nodes;", params);
        let mut stmnt = con.prepare(&stmnt).unwrap();
        stmnt
            .query_map([], |row| {
                let mut curr: [String; PARAMS] = [const { String::new() }; PARAMS];
                for i in 0..PARAMS {
                    curr[i] = row.get(i).unwrap_or_default();
                }
                Ok(curr)
            })
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: fix:
    // #[test]
    fn _test_into_node_from_org() {
        let res = SqliteConnection::into_node_from_org(
            "67716660-09aa-4cc7-8ce1-61dcfbe70522".to_string(),
            "Test".to_string(),
            "/home/user/org/test.org".to_string(),
            "1".to_string(),
            "(\"test1\" \"test 2\")".to_string(),
            "(\"test1\" \"test 2\")".to_string(),
            ["tag1", "tag2"].iter().map(ToString::to_string).collect(),
            vec!["t1".to_string()],
        );
        assert_eq!(
            res,
            NodeFromOrg {
                uuid: "67716660-09aa-4cc7-8ce1-61dcfbe70522".to_string(),
                title: "Test".to_string(),
                file: "/home/user/org/test.org".to_string(),
                level: 1,
                olp: ["test1", "test2"].iter().map(ToString::to_string).collect(),
                actual_olp: ["test1", "test2"].iter().map(ToString::to_string).collect(),
                tags: ["tag1", "tag2"].iter().map(ToString::to_string).collect(),
                aliases: vec!["t1".to_string()],
                content: "* title\n content".to_string(),
                links: vec![],
                parent: None,
                timestamps: None,
                refs: Vec::new(),
                cites: Vec::new()
            }
        )
    }
}
