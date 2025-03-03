use std::{mem::MaybeUninit, path::Path};

use rusqlite::Connection;

use crate::{
    datamodel::{NodeFromOrg, Timestamps},
    Logger,
};

pub struct SqliteConnection {
    connection: Connection,
}

impl SqliteConnection {
    pub fn init<P: AsRef<Path>>(path: P) -> Option<Self> {
        match Connection::open(path) {
            Ok(connection) => Some(Self { connection }),
            Err(_) => None,
        }
    }

    /// # Return
    /// It returs Vec of tuples where the first element is the name of the node
    /// pointing to `dest` and the second elemet is the id of the node.
    pub fn get_backlinks(&mut self, dest: String) -> Vec<(String, String)> {
        let stmnt = format!(
            "SELECT DISTINCT source, dest, pos, properties
             FROM links WHERE dest = '\"{}\"'
             AND type = '\"id\"'
             GROUP BY source
             HAVING min(pos);",
            dest
        );

        let mut stmt = self.connection.prepare(&stmnt).unwrap();

        stmt.query_map([], |row| Ok((row.get(0).unwrap(), row.get(3).unwrap())))
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
    }

    pub fn get_all_links(&mut self) -> Vec<(String, String)> {
        const STMNT: &'static str = "
            SELECT links.source, links.dest, links.type
            FROM links
            WHERE links.type = '\"id\"';";
        let mut stmnt = self.connection.prepare(STMNT).unwrap();

        stmnt
            .query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap())))
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
    }

    pub fn get_all_nodes<const PARAMS: usize>(
        &mut self,
        params: [&'static str; PARAMS],
    ) -> Vec<[String; PARAMS]> {
        let params = params.join(", ");
        let stmnt = format!("SELECT {} FROM nodes;", params);
        let mut stmnt = self.connection.prepare(&stmnt).unwrap();
        stmnt
            .query_map([], |row| {
                let mut curr: [String; PARAMS] = [const { String::new() }; PARAMS];
                for i in 0..PARAMS - 1 {
                    curr[i] = row.get(i).unwrap();
                }
                Ok(curr)
            })
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
    }

    /// Parse nodes from org-roam database and turn them into `NodeFromOrg` to
    /// embed them into the databases of org-roamers.
    ///
    /// # Future
    /// Depending on the future of this project, it might be beneficial to
    /// remove this function (or the entire file).
    pub fn nodes_from_org(&mut self) -> Vec<NodeFromOrg> {
        self.get_all_nodes(["id", "title", "file", "level", "olp", "tags"])
            .into_iter()
            .map(|[uuid, title, file, level, olp, tags]| {
                let content = String::new();
                let aliases = Vec::new();
                let ctime = String::new();
                let mtime = Vec::new();
                let timestamps = Timestamps::new(ctime, mtime);
                let links = Vec::new();
                let level = level.parse::<u64>().unwrap_or(0);
                let tags = tags.split(',').map(ToString::to_string).collect();
                let olp = Self::parse_olp(olp);
                NodeFromOrg {
                    uuid,
                    title,
                    content,
                    file,
                    level,
                    olp,
                    tags,
                    aliases,
                    timestamps,
                    links,
                }
            })
            .collect()
    }

    fn parse_olp(olp: String) -> Vec<String> {
        todo!()
    }
}
