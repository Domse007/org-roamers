use std::{collections::HashSet, time::Instant};

use anyhow::Result;
use rusqlite::Connection;
use tracing::info;

use crate::api::types::SearchResponseElement;

#[derive(PartialEq, Debug)]
struct ForNode<'a> {
    node_search: Vec<&'a str>,
    tag_filters: Vec<&'a str>,
}

impl<'a> ForNode<'a> {
    fn new(search: Vec<&'a str>) -> Self {
        let mut node_search = vec![];
        let mut tag_filters = vec![];
        for token in search {
            if token.starts_with('#') {
                tag_filters.push(&token[1..]);
            } else {
                node_search.push(token);
            }
        }
        Self {
            node_search,
            tag_filters,
        }
    }

    fn search(&self, con: &mut Connection) -> Result<Vec<SearchResponseElement>> {
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
        let mut result = vec![];
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
                        .any(|f| f.to_lowercase() == e[1..e.len() - 1].to_lowercase())
                });
                if p {
                    result.push(SearchResponseElement {
                        display: element.1[1..element.1.len() - 1].to_string(),
                        id: element.0.into(),
                    });
                }
            }
        } else {
            result = elements
                .map(|row| SearchResponseElement {
                    display: row.1[1..row.1.len() - 1].to_string(),
                    id: row.0.into(),
                })
                .collect();
        }
        Ok(result)
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
struct ForTag<'a> {
    tag_search: Vec<&'a str>,
}

impl<'a> ForTag<'a> {
    fn new(search: Vec<&'a str>) -> Self {
        Self { tag_search: search }
    }

    fn search(&self, con: &mut Connection) -> Result<Vec<SearchResponseElement>> {
        let params = format_tag_param(&self.tag_search);
        let stmnt = format!(
            "SELECT node_id, tag FROM tags WHERE LOWER(tag) IN {}",
            params
        );
        let mut stmnt = con.prepare(stmnt.as_str())?;
        let ids = stmnt
            .query_map([], |row| {
                Ok(row.get_unwrap::<usize, String>(0).to_lowercase())
            })?
            .map(Result::unwrap);
        let mut res = HashSet::new();
        const STMNT: &str = "SELECT id, title FROM nodes WHERE id = ?1";
        let mut stmnt = con.prepare(STMNT)?;
        for id in ids {
            let elem = stmnt
                .query_map([id], |row| {
                    let display: String = row.get_unwrap(1);
                    Ok(SearchResponseElement {
                        display: display[1..display.len() - 1].to_string(),
                        id: row.get_unwrap::<usize, String>(0).into(),
                    })
                })?
                .map(Result::unwrap)
                .next();
            if let Some(elem) = elem {
                res.insert(elem);
            }
        }
        Ok(res.into_iter().collect())
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
        match stype.as_ref().map(|s| s.as_str()) {
            Some("node") => Search::ForNode(ForNode::new(search)),
            Some("tag") => Search::ForTag(ForTag::new(search)),
            _ => Search::ForNode(ForNode::new(search)),
        }
    }

    pub fn search(&self, con: &mut Connection) -> Result<Vec<SearchResponseElement>> {
        let before = Instant::now();
        let res = match self {
            Self::ForNode(node) => node.search(con),
            Self::ForTag(tag) => tag.search(con),
        };
        let delta = Instant::now() - before;
        info!("Search query took {}ms.", delta.as_millis());
        res
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
