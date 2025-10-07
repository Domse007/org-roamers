use anyhow::Result;
use rusqlite::Connection;

use crate::{
    search::{Configuration, SearchResultSender},
    server::AppState,
    transform::title::TitleSanitizer,
};

#[derive(PartialEq, Debug)]
pub struct ForNode<'a> {
    node_search: Vec<&'a str>,
    tag_filters: Vec<&'a str>,
}

impl<'a> ForNode<'a> {
    fn new(search: Vec<&'a str>) -> Self {
        let mut node_search = vec![];
        let mut tag_filters = vec![];
        for token in search {
            if let Some(stripped) = token.strip_prefix('#') {
                tag_filters.push(stripped);
            } else {
                node_search.push(token);
            }
        }
        Self {
            node_search,
            tag_filters,
        }
    }

    fn search<F: Fn(&str) -> String>(
        &self,
        con: &mut Connection,
        sender: &mut SearchResultSender,
        title_sanitizer: F,
    ) -> anyhow::Result<()> {
        let param = format_search_param(&self.node_search);
        let stmnt = "SELECT id, title FROM nodes WHERE LOWER(title) LIKE ?1";
        let mut stmnt = con.prepare(stmnt)?;
        let elements = stmnt
            .query_map([param], |row| {
                Ok((
                    row.get::<usize, String>(0).unwrap(),
                    row.get::<usize, String>(1).unwrap(),
                ))
            })?
            .map(Result::unwrap);
        if !self.tag_filters.is_empty() {
            for element in elements {
                let to_query = &element.0;
                let stmnt = "SELECT node_id, tag FROM tags WHERE node_id = ?1";
                let mut stmnt = con.prepare(stmnt)?;
                let mut tags = stmnt
                    .query_map(rusqlite::params![to_query], |row| {
                        Ok(row.get_unwrap::<usize, String>(1))
                    })?
                    .map(Result::unwrap);
                let p = tags.any(|e| {
                    self.tag_filters
                        .iter()
                        .any(|f| f.to_lowercase() == e.to_lowercase())
                });
                if p {
                    if let Err(err) = sender.send(
                        title_sanitizer(&element.1).into(),
                        element.0.into(),
                        tags.collect(),
                        None,
                    ) {
                        tracing::error!("Error sending: {err}");
                    };
                }
            }
        } else {
            for row in elements {
                let to_query = &row.0;
                let stmnt = "SELECT node_id, tag FROM tags WHERE node_id = ?1";
                let mut stmnt = con.prepare(stmnt).unwrap();
                let tags = stmnt
                    .query_map(rusqlite::params![to_query], |row| {
                        Ok(row.get_unwrap::<usize, String>(1))
                    })
                    .unwrap()
                    .map(Result::unwrap)
                    .collect();
                let title = if row.1.is_empty() {
                    tracing::error!("Title is empty: {:?}", row);
                    String::new()
                } else {
                    title_sanitizer(&row.1)
                };
                if let Err(err) = sender.send(title.into(), row.0.into(), tags, None) {
                    tracing::error!("Error sending: {err}");
                };
            }
        }
        Ok(())
    }
}

fn format_search_param(search: &[&str]) -> String {
    let mut s = "%".to_string();
    for t in search {
        s.push_str(t.to_lowercase().replace("\"", "\"\"").as_str());
        s.push('%');
    }
    s
}

#[derive(PartialEq, Debug)]
pub struct ForTag<'a> {
    tag_search: Vec<&'a str>,
}

impl<'a> ForTag<'a> {
    fn new(search: Vec<&'a str>) -> Self {
        Self { tag_search: search }
    }

    fn search<F: Fn(&str) -> String>(
        &self,
        con: &mut Connection,
        sender: &mut SearchResultSender,
        title_sanitizer: F,
    ) -> anyhow::Result<()> {
        let params = format_tag_param(&self.tag_search);
        let stmnt = format!(
            "SELECT node_id, tag FROM tags WHERE LOWER(tag) IN {}",
            params
        );
        let mut stmnt = con.prepare(stmnt.as_str())?;
        let (ids, tags): (Vec<String>, Vec<String>) = stmnt
            .query_map([], |row| {
                Ok((
                    row.get_unwrap::<usize, String>(0).to_lowercase(),
                    row.get_unwrap::<usize, String>(1).to_lowercase(),
                ))
            })?
            .map(Result::unwrap)
            .unzip();
        const STMNT: &str = "SELECT id, title FROM nodes WHERE id = ?1";
        let mut stmnt = con.prepare(STMNT)?;
        for id in ids {
            let tags = tags.clone();
            let elem = stmnt
                .query_map([id], |row| {
                    let display: String = row.get_unwrap(1);
                    Ok((
                        title_sanitizer(&display[1..display.len() - 1]),
                        row.get_unwrap::<usize, String>(0).into(),
                        tags.clone(),
                    ))
                })?
                .map(Result::unwrap)
                .next();
            if let Some((title, id, tags)) = elem {
                if let Err(err) = sender.send(title.into(), id, tags, None) {
                    tracing::error!("Error sending: {err}");
                };
            }
        }

        Ok(())
    }
}

fn format_tag_param(search: &[&str]) -> String {
    let mut s = "(".to_string();
    let mut f = false;
    for tag in search {
        if f {
            s.push_str(", ");
        }
        f = true;
        s.push_str("\"\"\"");
        s.push_str(tag.replace("\"", "\"\"").as_str());
        s.push_str("\"\"\"");
    }
    s.push(')');
    s
}

#[derive(PartialEq, Debug)]
pub enum Search<'a> {
    ForNode(ForNode<'a>),
    ForTag(ForTag<'a>),
}

impl<'a> Search<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut stype = None;
        let mut iter = s.split_whitespace();
        let mut search = vec![];

        while let Some(token) = iter.next() {
            if token.to_lowercase() == ":type" {
                stype = iter.next().map(|t| t.to_lowercase());
            } else {
                search.push(token);
            }
        }
        match stype.as_deref() {
            Some("node") => Search::ForNode(ForNode::new(search)),
            Some("tag") => Search::ForTag(ForTag::new(search)),
            _ => Search::ForNode(ForNode::new(search)),
        }
    }

    pub fn search(&self, sender: &mut SearchResultSender, con: &mut Connection) -> Result<()> {
        let title_sanitizer = |title: &str| {
            let sanitier = TitleSanitizer::new();
            sanitier.process(title)
        };

        match self {
            Self::ForNode(node) => node.search(con, sender, title_sanitizer),
            Self::ForTag(tag) => tag.search(con, sender, title_sanitizer),
        }
    }
}

pub struct DefaultSearch {
    pub(crate) sender: SearchResultSender,
}

impl DefaultSearch {
    pub fn new(sender: SearchResultSender) -> Self {
        Self { sender }
    }

    pub fn id(&self) -> usize {
        self.sender.id()
    }

    pub fn configuration(&self) -> super::Configuration {
        Configuration {
            returns_preview: false,
        }
    }

    pub async fn feed(&mut self, state: AppState, f: &super::Feeder) -> anyhow::Result<()> {
        let query = f.s.clone();
        let mut sender = self.sender.clone();
        
        // Wrap the blocking database operation in spawn_blocking
        tokio::task::spawn_blocking(move || {
            let mut state_guard = state.lock().unwrap();
            let state = &mut *state_guard;
            let mut sqlite_guard = state.sqlite.lock().unwrap();
            let connection = sqlite_guard.connection();

            let search = Search::new(&query);
            search.search(&mut sender, connection)
        })
        .await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_format_search_param() {
        let test = ["Chapter", "noDe", "\"modules\""];
        assert_eq!(format_search_param(&test), "%chapter%node%\"\"modules\"\"%");
    }
    #[test]
    fn test_format_tag_param() {
        let test = ["studies", "compsci"];
        assert_eq!(
            format_tag_param(&test),
            "(\"\"\"studies\"\"\", \"\"\"compsci\"\"\")"
        );
    }
    #[test]
    fn test_search_new_tag() {
        let test = "studies :type tag compsci";
        let expected = Search::ForTag(ForTag {
            tag_search: vec!["studies", "compsci"],
        });
        assert_eq!(Search::new(test), expected);
    }
    #[test]
    fn test_search_new_node() {
        let test = "notes node #compsci commands";
        let expected = Search::ForNode(ForNode {
            node_search: vec!["notes", "node", "commands"],
            tag_filters: vec!["compsci"],
        });
        assert_eq!(Search::new(test), expected);
    }
}
