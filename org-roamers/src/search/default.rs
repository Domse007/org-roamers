use anyhow::Result;
use futures_util::StreamExt;
use sqlx::SqlitePool;

use crate::{search::SearchResultSender, server::AppState, transform::title::TitleSanitizer};

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

    async fn search<F: Fn(&str) -> String>(
        &self,
        con: &SqlitePool,
        sender: &mut SearchResultSender,
        title_sanitizer: F,
    ) -> anyhow::Result<()> {
        let param = format_search_param(&self.node_search);
        // Search both node titles and aliases, using DISTINCT to avoid duplicates
        let stmnt = r#"
            SELECT DISTINCT n.id, n.title 
            FROM nodes n
            LEFT JOIN aliases a ON n.id = a.node_id
            WHERE LOWER(n.title) LIKE ? OR LOWER(a.alias) LIKE ?
        "#;
        let elements: Vec<(String, String)> =
            sqlx::query_as(stmnt).bind(&param).bind(&param).fetch_all(con).await?;
        if !self.tag_filters.is_empty() {
            for element in elements {
                let to_query = &element.0;
                let stmnt = "SELECT node_id, tag FROM tags WHERE node_id = ?";
                let tags: Vec<(String,)> =
                    sqlx::query_as(stmnt).bind(to_query).fetch_all(con).await?;
                let p = tags.iter().any(|e| {
                    self.tag_filters
                        .iter()
                        .any(|f| f.to_lowercase() == e.0.to_lowercase())
                });
                if p {
                    if let Err(err) = sender.send(
                        title_sanitizer(&element.1).into(),
                        element.0.into(),
                        tags.into_iter().map(|e| e.0).collect(),
                        None,
                    ) {
                        tracing::error!("Error sending: {err}");
                    };
                }
            }
        } else {
            for row in elements {
                let to_query = &row.0;
                let stmnt = "SELECT node_id, tag FROM tags WHERE node_id = ?";
                let tags: Vec<(String,)> =
                    sqlx::query_as(stmnt).bind(to_query).fetch_all(con).await?;
                let title = if row.1.is_empty() {
                    tracing::error!("Title is empty: {:?}", row);
                    String::new()
                } else {
                    title_sanitizer(&row.1)
                };
                if let Err(err) = sender.send(
                    title.into(),
                    row.0.into(),
                    tags.into_iter().map(|e| e.0).collect(),
                    None,
                ) {
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

    async fn search<F: Fn(&str) -> String>(
        &self,
        con: &SqlitePool,
        sender: &mut SearchResultSender,
        title_sanitizer: F,
    ) -> anyhow::Result<()> {
        let params = format_tag_param(&self.tag_search);
        let stmnt = "SELECT node_id, tag FROM tags WHERE LOWER(tag) IN ?";
        let (ids, tags): (Vec<String>, Vec<String>) = sqlx::query_as(stmnt)
            .bind(params)
            .fetch(con)
            .map(|e| e.unwrap())
            .unzip()
            .await;
        const STMNT: &str = "SELECT id, title FROM nodes WHERE id = ?";
        for id in ids {
            let tags = tags.clone();
            let (id, display): (String, String) =
                sqlx::query_as(STMNT).bind(id).fetch_one(con).await?;
            let (title, id, tags) = (
                title_sanitizer(&display[1..display.len() - 1]),
                id.into(),
                tags.clone(),
            );
            if let Err(err) = sender.send(title.into(), id, tags, None) {
                tracing::error!("Error sending: {err}");
            };
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

    pub async fn search(&self, sender: &mut SearchResultSender, con: AppState) -> Result<()> {
        let title_sanitizer = |title: &str| {
            let sanitier = TitleSanitizer::new();
            sanitier.process(title)
        };

        let sqlite = con.lock().unwrap().sqlite.clone();

        match self {
            Self::ForNode(node) => {
                node.search(&sqlite, sender, title_sanitizer)
                    .await
            }
            Self::ForTag(tag) => {
                tag.search(&sqlite, sender, title_sanitizer)
                    .await
            }
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

    pub async fn feed(&mut self, state: AppState, f: &super::Feeder) -> anyhow::Result<()> {
        let query = f.s.clone();
        let mut sender = self.sender.clone();

        // Wrap the blocking database operation in spawn_blocking
        tokio::spawn(async move {
            let search = Search::new(&query);
            if let Err(e) = search.search(&mut sender, state).await {
                tracing::error!("Search error: {e}");
            }
        });

        Ok(())
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
