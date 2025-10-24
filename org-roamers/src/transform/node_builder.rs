use std::collections::HashSet;

use orgize::{
    ast::{Keyword, Link},
    export::{Container, Event, Traverser},
    Org, SyntaxElement,
};
use sqlx::SqlitePool;

use crate::sqlite::rebuild;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OrgNode {
    pub(crate) uuid: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) level: u64,
    pub(crate) parent: Option<String>,
    pub(crate) olp: Vec<String>,
    pub(crate) actual_olp: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) aliases: Vec<String>,
    pub(crate) links: Vec<(String, String)>,
    pub(crate) refs: Vec<String>,
    pub(crate) cites: Vec<String>,
    pub(crate) file: String,
}

impl OrgNode {
    #[rustfmt::skip]
    pub async fn insert_node(&self, con: &SqlitePool) -> anyhow::Result<()> {
        // this does not insert olp, tags, etc. -- why?
        rebuild::insert_node(
            con, &self.uuid, &self.file, self.level,
            false, 0, "", "", self.title.as_str(), &self.actual_olp
        ).await
    }

    pub async fn insert_tags(&self, con: &SqlitePool) -> anyhow::Result<()> {
        for tag in &self.tags {
            rebuild::insert_tag(con, &self.uuid, &tag).await?;
        }
        Ok(())
    }

    pub async fn insert_aliases(&self, con: &SqlitePool) -> anyhow::Result<()> {
        for alias in &self.aliases {
            rebuild::insert_alias(con, &self.uuid, &alias).await?;
        }
        Ok(())
    }

    pub async fn insert_links(&self, con: &SqlitePool) -> anyhow::Result<()> {
        for link in &self.links {
            rebuild::insert_link(con, &self.uuid, &link.0).await?;
        }
        Ok(())
    }
}

pub async fn insert_nodes(con: &SqlitePool, nodes: Vec<OrgNode>) {
    for node in nodes.iter() {
        // Only insert tags, aliases, and links if the node was successfully inserted
        match node.insert_node(con).await {
            Ok(_) => {
                if let Err(err) = node.insert_tags(con).await {
                    tracing::error!("Failed to insert tags for node {}: {}", node.uuid, err);
                }
                if let Err(err) = node.insert_aliases(con).await {
                    tracing::error!("Failed to insert aliases for node {}: {}", node.uuid, err);
                }
                if let Err(err) = node.insert_links(con).await {
                    tracing::error!("Failed to insert links for node {}: {}", node.uuid, err);
                }
            }
            Err(err) => {
                tracing::error!(
                    "Failed to insert node {}: {} - skipping tags, aliases, and links",
                    node.uuid,
                    err
                );
            }
        }
    }
}

pub fn get_nodes(content: &str, file: &str) -> Vec<OrgNode> {
    let org = Org::parse(content);

    let mut traverser = NodesBuilder::new(file);
    org.traverse(&mut traverser);
    traverser.nodes
}

#[derive(Default)]
pub struct NodesBuilder {
    nodes: Vec<OrgNode>,
    id_stack: Vec<(String, String)>,
    tags_stack: Vec<Vec<String>>,
    olp: Vec<String>,
    actual_olp: Vec<String>,
    file: String,
}

impl NodesBuilder {
    pub fn new(file: &str) -> Self {
        Self {
            file: file.to_string(),
            ..Default::default()
        }
    }

    pub fn current_olp(&self) -> Vec<String> {
        self.olp.clone()
    }

    pub fn current_actual_olp(&self) -> Vec<String> {
        self.actual_olp.clone()
    }

    pub fn get_tags(&self) -> Vec<String> {
        let mut tags = self
            .tags_stack
            .iter()
            .flatten()
            .cloned()
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();
        tags.sort();
        tags
    }
}

impl Traverser for NodesBuilder {
    fn event(&mut self, event: orgize::export::Event, _ctx: &mut orgize::export::TraversalContext) {
        match event {
            Event::Enter(Container::Document(document)) => {
                if let Some(properties) = document.properties() {
                    if let Some(id) = properties.get("ID") {
                        let title = document.title().unwrap_or_else(String::new);
                        let tags = get_tags_from_keywords(document.keywords());
                        let id = id.to_string();
                        let content = document.raw();
                        let aliases = properties
                            .get("ROAM_ALIASES")
                            .map(parse_aliases)
                            .unwrap_or_default();

                        let node = OrgNode {
                            title: title.clone(),
                            uuid: id.clone(),
                            content,
                            level: 0,
                            tags: tags.clone(),
                            aliases,
                            parent: None,
                            olp: vec![],
                            actual_olp: vec![],
                            file: self.file.clone(),
                            ..Default::default()
                        };

                        self.nodes.push(node);
                        self.tags_stack.push(tags);

                        self.id_stack.push((title, id));
                    }
                }
                // REMARK: org-roam does not use the main title as part of the olp path.
                // only org-roamers has this additional field...
                if let Some(title) = document.title() {
                    self.actual_olp.push(title);
                }
            }
            Event::Leave(Container::Document(_)) => {
                let _ = self.id_stack.pop();
                let _ = self.tags_stack.pop();
                let _ = self.olp.pop();
            }
            Event::Enter(Container::Headline(headline)) => {
                if let Some(properties) = headline.properties() {
                    if let Some(id) = properties.get("ID") {
                        let my_parent = self.id_stack.last().map(|p| p.1.to_string());
                        let aliases = properties
                            .get("ROAM_ALIASES")
                            .map(parse_aliases)
                            .unwrap_or_default();

                        let tags: Vec<String> = headline
                            .tags()
                            .map(|t| t.to_string())
                            .filter(|t| !t.trim().is_empty())
                            .collect();

                        let id = id.to_string();
                        // TODO: this is wrong.
                        let title = headline.title_raw().trim().to_string();
                        let level = headline.level() as u64;
                        let olp = self.current_olp();
                        let actual_olp = self.current_actual_olp();

                        // update parent for children.
                        self.id_stack.push((title.clone(), id.clone()));

                        let mut content = match headline.section() {
                            Some(section) => section.raw(),
                            None => String::new(),
                        };
                        let subheading = headline
                            .headlines()
                            .map(|headline| headline.raw())
                            .collect::<String>();

                        content.push_str(&subheading);

                        // NOTE: this derives from the org-roam implemementation to prevent
                        // additional queries when computing inherited tags.
                        self.tags_stack.push(tags);

                        let node = OrgNode {
                            title,
                            uuid: id,
                            content,
                            level,
                            parent: my_parent,
                            tags: self.get_tags(),
                            olp,
                            actual_olp,
                            aliases,
                            file: self.file.clone(),
                            ..Default::default()
                        };

                        self.nodes.push(node);
                    }
                }
                self.olp.push(headline.title_raw());
                self.actual_olp.push(headline.title_raw());
            }
            Event::Leave(Container::Headline(headline)) => {
                let _ = self.olp.pop();
                let _ = self.actual_olp.pop();
                if let Some(properties) = headline.properties() {
                    if let Some(id) = properties.get("ID") {
                        if let Some((_, id_from_stack)) = self.id_stack.last() {
                            if id == *id_from_stack {
                                let _ = self.id_stack.pop();
                                let _ = self.tags_stack.pop();
                            }
                        }
                    }
                }
            }
            Event::Enter(Container::Link(link)) => {
                if let Some((id, description)) = parse_link(link) {
                    let id_parent = match self.id_stack.last() {
                        Some(parent) => parent,
                        None => return,
                    };
                    let node = self
                        .nodes
                        .iter_mut()
                        .rev()
                        .find(|n| n.title == id_parent.0.trim());
                    if let Some(node) = node {
                        node.links.push((id, description));
                    } else {
                        tracing::error!("Did not find parent for {id}");
                    }
                }
            }
            _ => {}
        }
    }
}

fn parse_aliases(aliases: orgize::ast::Token) -> Vec<String> {
    aliases
        .split(' ')
        .map(|s| s.trim())
        .map(ToString::to_string)
        .collect()
}

fn parse_link(link: Link) -> Option<(String, String)> {
    let path = link.path();

    if let Some((t, id)) = path.split_once(':') {
        if t.to_lowercase() == "id" {
            let desc = link
                .description()
                .map(|s| match s {
                    SyntaxElement::Node(node) => node.text().to_string(),
                    SyntaxElement::Token(token) => token.text().to_string(),
                })
                .collect::<String>();

            return Some((id.to_string(), desc));
        }
    }

    None
}

fn get_tags_from_keywords(iter: impl Iterator<Item = Keyword>) -> Vec<String> {
    iter.filter(|kw| kw.key().to_lowercase().as_str() == "filetags")
        .map(|kw| kw.value())
        .flat_map(|tags| {
            tags.split(':')
                .map(|e| e.to_string())
                .filter(|t| !t.trim().is_empty())
                .collect::<Vec<String>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_gatherer_1() {
        const ORG: &str = ":PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:END:
#+title: Hello World
Welcome
* testing
:PROPERTIES:
:ID:       e6557233-97db-4eec-925a-b80d66ad97e8
:END:
some text
";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(
            res,
            vec![
                OrgNode {
                    title: "Hello World".to_string(),
                    parent: None,
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: ORG.to_string(),
                    level: 0,
                    file: "test.org".to_string(),
                    ..Default::default()
                },
                OrgNode {
                    title: "testing".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 1,
                    olp: vec![],
                    actual_olp: vec!["Hello World".to_string()],
                    file: "test.org".to_string(),
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn test_node_gatherer_2() {
        const ORG: &str = "
* Hello World
:PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:END:
Welcome
** Hello
:PROPERTIES:
:ID:       e655725d-97db-4eec-925a-b80d66ad97e8
:END:
Welcome
* testing
:PROPERTIES:
:ID:       e6557233-97db-4eec-925a-b80d66ad97e8
:END:
some text
";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(
            res,
            vec![
                OrgNode {
                    title: "Hello World".to_string(),
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    parent: None,
                    content: "Welcome\n** Hello\n:PROPERTIES:\n:ID:       e655725d-97db-4eec-925a-b80d66ad97e8\n:END:\nWelcome\n".to_string(),
                    level: 1,
                    file: "test.org".to_string(),
                    ..Default::default()
                },
                OrgNode {
                    title: "Hello".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e655725d-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n".to_string(),
                    olp: vec!["Hello World".to_string()],
                                        actual_olp: vec!["Hello World".to_string()],
                    level: 2,
                    file: "test.org".to_string(),
                    ..Default::default()
                },
                OrgNode {
                    title: "testing".to_string(),
                    parent: None,
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 1,
                    file: "test.org".to_string(),
                    ..Default::default()
                },
            ]
        );
    }

    #[test]
    fn test_node_gatherer_deep() {
        const ORG: &str = "
* Hello World
:PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:END:
Welcome
** Hello
:PROPERTIES:
:ID:       e655725d-97db-4eec-925a-b80d66ad97e8
:END:
Welcome
*** testing
:PROPERTIES:
:ID:       e6557233-97db-4eec-925a-b80d66ad97e8
:END:
some text
";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(
            res,
            vec![
                OrgNode {
                    title: "Hello World".to_string(),
                    parent: None,
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n** Hello\n:PROPERTIES:\n:ID:       e655725d-97db-4eec-925a-b80d66ad97e8\n:END:\nWelcome\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
                    level: 1,
                    file: "test.org".to_string(),
                    ..Default::default()
                },
                OrgNode {
                    title: "Hello".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e655725d-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
                    olp: vec!["Hello World".to_string()],
                                        actual_olp: vec!["Hello World".to_string()],
                    level: 2,
                    file: "test.org".to_string(),
                    ..Default::default()
                },
                OrgNode {
                    title: "testing".to_string(),
                    parent: Some("e655725d-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    olp: vec!["Hello World".to_string(), "Hello".to_string()],
                    actual_olp: vec!["Hello World".to_string(), "Hello".to_string()],
                    level: 3,
                    file: "test.org".to_string(),
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn test_node_gatherer_skipped_heading() {
        const ORG: &str = "
* Hello World
:PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:END:
Welcome
** Hello
test
*** testing
:PROPERTIES:
:ID:       e6557233-97db-4eec-925a-b80d66ad97e8
:END:
some text
";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(
            res,
            vec![
                OrgNode {
                    title: "Hello World".to_string(),
                    parent: None,
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n** Hello\ntest\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
                    level: 1,
                    file: "test.org".to_string(),
                    ..Default::default()
                },
                OrgNode {
                    title: "testing".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    olp: vec!["Hello World".to_string(), "Hello".to_string()],
                    actual_olp: vec!["Hello World".to_string(), "Hello".to_string()],
                    level: 3,
                    file: "test.org".to_string(),
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn test_get_tags() {
        const ORG: &str = ":PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:END:
#+title: Test
#+filetags: :test1:test2:test3:
* other :test1:test2:
:PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e9
:END:";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(
            res,
            vec![
                OrgNode {
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    title: "Test".to_string(),
                    content: ORG.to_string(),
                    level: 0,
                    parent: None,
                    tags: vec![
                        "test1".to_string(),
                        "test2".to_string(),
                        "test3".to_string()
                    ],
                    file: "test.org".to_string(),
                    ..Default::default()
                },
                OrgNode {
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e9".to_string(),
                    title: "other".to_string(),
                    content: String::new(),
                    level: 1,
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    tags: vec![
                        "test1".to_string(),
                        "test2".to_string(),
                        "test3".to_string(),
                    ],
                    olp: vec![],
                    actual_olp: vec!["Test".to_string()],
                    file: "test.org".to_string(),
                    ..Default::default()
                },
            ]
        );
    }

    #[test]
    fn test_parse_links() {
        const ORG: &str = ":PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:END:
#+title: Test
* other
:PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e9
:END:
Linking to [[id:e655725f-97db-4eec-925a-b80d66ad97e8][Test]]";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(res[0].links, vec![]);
        assert_eq!(
            res[1].links,
            vec![(
                "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                "Test".to_string()
            )]
        );
    }

    #[test]
    fn test_inherited_linking() {
        const ORG: &str = ":PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:END:
#+title: Test
* other
Linking to [[id:e655725f-97db-4eec-925a-b80d66ad97e8][Test]]";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(
            res[0].links,
            vec![(
                "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                "Test".to_string()
            )]
        );
    }

    #[test]
    fn test_aliases() {
        const ORG: &str = ":PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e8
:ROAM_ALIASES: test1 test2
:END:
#+title: Test
* other
:PROPERTIES:
:ID:       e655725f-97db-4eec-925a-b80d66ad97e9
:ROAM_ALIASES: test3 test4
:END:";
        let res = get_nodes(ORG, "test.org");
        assert_eq!(
            res[0].aliases,
            vec!["test1".to_string(), "test2".to_string()]
        );
        assert_eq!(
            res[1].aliases,
            vec!["test3".to_string(), "test4".to_string()]
        );
    }
}
