use std::path::Path;

use anyhow::{bail, Result};
use rebuild::{IterFilesError, IterFilesStats};
use rusqlite::{Connection, Params};

use crate::error::ServerError;

pub mod init;
pub mod olp;
pub mod rebuild;

pub struct SqliteConnection {
    connection: Connection,
}

impl SqliteConnection {
    const MIN_VERSION: usize = 20;

    pub fn init(strict: bool) -> Result<Self> {
        let this = {
            let mut connection = Connection::open_in_memory()?;
            init::init_version(&mut connection, Self::MIN_VERSION)?;
            init::init_files_table(&mut connection)?;
            init::init_nodes_table(&mut connection)?;
            init::init_links_table(&mut connection)?;
            init::init_aliases(&mut connection)?;
            init::init_tags(&mut connection)?;
            init::init_olp_table(&mut connection)?;
            Self { connection }
        };

        let version: usize = this
            .connection
            .pragma_query_value(None, "user_version", |row| Ok(row.get_unwrap(0)))?;

        if strict {
            if let Err(err) = this.connection.execute("PRAGMA foreign_keys = ON;", []) {
                tracing::error!("Could not enable foreign_keys: {err}");
            }
        } else {
            if let Err(err) = this.connection.execute("PRAGMA foreign_keys = OFF;", []) {
                tracing::error!("Could not disable foreign_keys: {err}");
            }
        }

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

    pub fn execute<P>(&self, sql: &str, params: P) -> Result<usize, ServerError>
    where
        P: Params,
    {
        self.connection.execute(sql, params).map_err(Into::into)
    }

    pub fn connection(&mut self) -> &mut Connection {
        &mut self.connection
    }

    pub fn insert_files<P: AsRef<Path>>(&mut self, roam_path: P) -> Result<(), IterFilesError> {
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
                for (i, elem) in curr.iter_mut().enumerate().take(PARAMS) {
                    if let Ok(col) = row.get(i) {
                        *elem = col;
                    }
                }
                Ok(curr)
            })
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
    }
}
