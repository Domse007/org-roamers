use std::{path::Path, str::Chars};

use rusqlite::Connection;

use crate::{database::datamodel::Timestamps, org::NodeFromOrg, parser::Parser};

#[derive(thiserror::Error, Debug)]
pub enum OlpError {
    #[error("StringParseError on char. Already extracted: {0:?}")]
    StringParseError(Vec<String>),
    #[error("Character '{0}' was not expected.")]
    InvalidChar(char),
    #[error("No more characters to consume.")]
    IteratorExhaustion,
}

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
                for i in 0..PARAMS {
                    curr[i] = row.get(i).unwrap();
                }
                Ok(curr)
            })
            .unwrap()
            .map(|e| e.unwrap())
            .collect()
    }

    pub fn get_aliases_for_node(&mut self, id: &str) -> Vec<String> {
        let stmnt = format!("SELECT node_id, alias FROM aliases WHERE node_id == '\"{id}\"'");
        let mut stmnt = self.connection.prepare(&stmnt).unwrap();
        stmnt
            .query_map([], |row| Ok(row.get(1).unwrap()))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    pub fn get_tags_for_node(&mut self, id: &str) -> Vec<String> {
        let stmnt = format!("SELECT node_id, tag FROM tags WHERE node_id == '\"{id}\"';");
        let mut stmnt = self.connection.prepare(&stmnt).unwrap();
        stmnt
            .query_map([], |row| Ok(row.get(1).unwrap()))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }

    /// Parse nodes from org-roam database and turn them into `NodeFromOrg` to
    /// embed them into the databases of org-roamers.
    ///
    /// # Future
    /// Depending on the future of this project, it might be beneficial to
    /// remove this function (or the entire file).
    pub fn nodes_from_org(&mut self) -> Vec<NodeFromOrg> {
        // TODO: tags must be fetched from tags table
        self.get_all_nodes(["id", "title", "file", "level", "olp"])
            .into_iter()
            .map(|[uuid, title, file, level, olp]| {
                let tags = self.get_tags_for_node(&uuid);
                let aliases = self.get_aliases_for_node(&uuid);
                Self::into_node_from_org(uuid, title, file, level, olp, tags, aliases)
            })
            .collect()
    }

    pub(super) fn into_node_from_org(
        uuid: String,
        title: String,
        file: String,
        level: String,
        olp: String,
        tags: Vec<String>,
        aliases: Vec<String>,
    ) -> NodeFromOrg {
        let content = String::new();
        let ctime = String::new();
        let mtime = Vec::new();
        let timestamps = Some(Timestamps::new(ctime, mtime));
        let links = Vec::new();
        let level = level.parse::<u64>().unwrap_or(0);
        let olp = Self::parse_olp(olp).unwrap();
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
            parent: None,
            links,
            // TODO: Handle references
            refs: Vec::new(),
        }
    }

    pub(super) fn parse_olp(olp: String) -> anyhow::Result<Vec<String>> {
        let mut parser = Parser::new(&olp);
        let whitespace = |parser: &mut Parser| {
            let mut attempt = parser.attempt();
            attempt.consume_whitespace();
            parser.sync(attempt);
        };

        whitespace(&mut parser);

        let mut attempt = parser.attempt();
        if let None = attempt.consume_char('(') {
            return Err(OlpError::InvalidChar('(').into());
        }
        parser.sync(attempt);

        let mut paths = vec![];

        loop {
            let mut attempt = parser.attempt();
            match attempt.consume_string() {
                Some(path) => paths.push(path),
                None => {
                    whitespace(&mut parser);
                    let mut attempt = parser.attempt();
                    if let Some(_) = attempt.consume_char(')') {
                        return Ok(paths);
                    } else {
                        break;
                    }
                }
            }
            parser.sync(attempt);
            whitespace(&mut parser);
        }

        Err(OlpError::StringParseError(paths).into())
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
                tags: ["tag1", "tag2"].iter().map(ToString::to_string).collect(),
                aliases: vec!["t1".to_string()],
                content: "* title\n content".to_string(),
                links: vec![],
                parent: None,
                timestamps: None,
                refs: Vec::new(),
            }
        )
    }

    #[test]
    fn test_olp_parser_correct() {
        const OLP: &'static str = "(\"This is a test\" \"How about that\")";
        let res = SqliteConnection::parse_olp(OLP.to_string());
        // assert!(res.is_ok(), "An error occured in the parsing process.");
        assert_eq!(
            res.unwrap(),
            vec!["This is a test".to_string(), "How about that".to_string()]
        );
    }
}
