use std::{fs::File, io::Read, path::Path};

use orgize::{
    ast::{Document, Headline},
    Org,
};

use crate::database::datamodel::Timestamps;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodeFromOrg {
    pub(crate) uuid: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) file: String,
    pub(crate) level: u64,
    pub(crate) parent: Option<String>,
    pub(crate) olp: Vec<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) aliases: Vec<String>,
    pub(crate) timestamps: Option<Timestamps>,
    pub(crate) links: Vec<(String, String)>,
    pub(crate) refs: Vec<String>,
}

fn get_orgize<P: AsRef<Path>>(path: P) -> anyhow::Result<Org> {
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;

    Ok(Org::parse(&content))
}

pub fn get_nodes_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<NodeFromOrg>> {
    let org = get_orgize(path)?;
    let document = org.document();

    get_nodes_from_document(document)
}

pub fn get_nodes_from_document(document: Document) -> anyhow::Result<Vec<NodeFromOrg>> {
    let mut nodes = Vec::new();

    let mut parent = None;

    // parsing #+title
    if let Some(title) = document.title() {
        if let Some(properties) = document.properties() {
            if let Some(id) = properties.get("ID") {
                parent = Some(id.to_string());
                let id = id.to_string();
                let content = document.raw();
                let mut node = NodeFromOrg::default();
                node.title = title;
                node.uuid = id;
                node.content = content;
                node.level = 0;
                nodes.push(node);
            }
        }
    }

    // parsing headlines and recursively looking for nodes
    let mut stack = document
        .headlines()
        .map(|h| (h, parent.clone()))
        .collect::<Vec<(Headline, Option<String>)>>();

    while let Some((headline, mut parent)) = stack.pop() {
        if let Some(properties) = headline.properties() {
            if let Some(id) = properties.get("ID") {
                let my_parent = parent.clone();

                let id = id.to_string();
                // TODO: this is wrong.
                let title = headline.title_raw();
                let level = headline.level() as u64;

                // update parent for children.
                parent = Some(id.clone());

                let mut content = match headline.section() {
                    Some(section) => section.raw(),
                    None => String::new(),
                };
                let subheading = headline
                    .headlines()
                    .map(|headline| headline.raw())
                    .collect::<String>();

                content.push_str(&subheading);

                let mut node = NodeFromOrg::default();
                node.title = title;
                node.uuid = id;
                node.content = content;
                node.level = level;
                node.parent = my_parent;

                nodes.push(node);
            }
        }

        // recursively add children to stack
        for hl in headline.headlines() {
            stack.push((hl, parent.clone()));
        }
    }

    Ok(nodes)
}

pub fn get_latex_header<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<String>> {
    let org = get_orgize(path)?;
    get_latex_hader_from_document(org.document())
}

pub fn get_latex_hader_from_document(document: Document) -> anyhow::Result<Vec<String>> {
    let mut headers = vec![];

    for keyword in document.keywords() {
        if keyword.key().to_lowercase() == "latex_header" {
            headers.push(keyword.value().trim().to_string());
        }
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_gatherer_1() {
        const ORG: &'static str = ":PROPERTIES:
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
        let org = Org::parse(ORG);
        let document = org.document();
        let res = get_nodes_from_document(document).unwrap();
        assert_eq!(
            res,
            vec![
                NodeFromOrg {
                    title: "Hello World".to_string(),
                    parent: None,
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: ORG.to_string(),
                    level: 0,
                    ..Default::default()
                },
                NodeFromOrg {
                    title: "testing".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 1,
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn test_node_gatherer_2() {
        const ORG: &'static str = "
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
        let org = Org::parse(ORG);
        let document = org.document();
        let res = get_nodes_from_document(document).unwrap();
        assert_eq!(
            res,
            vec![
                NodeFromOrg {
                    title: "testing".to_string(),
                    parent: None,
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 1,
                    ..Default::default()
                },
                NodeFromOrg {
                    title: "Hello World".to_string(),
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    parent: None,
                    content: "Welcome\n** Hello\n:PROPERTIES:\n:ID:       e655725d-97db-4eec-925a-b80d66ad97e8\n:END:\nWelcome\n".to_string(),
                    level: 1,
                    ..Default::default()
                },
                NodeFromOrg {
                    title: "Hello".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e655725d-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n".to_string(),
                    level: 2,
                    ..Default::default()
                },
            ]
        );
    }

    #[test]
    fn test_node_gatherer_deep() {
        const ORG: &'static str = "
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
        let org = Org::parse(ORG);
        let document = org.document();
        let res = get_nodes_from_document(document).unwrap();
        assert_eq!(
            res,
            vec![
                NodeFromOrg {
                    title: "Hello World".to_string(),
                    parent: None,
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n** Hello\n:PROPERTIES:\n:ID:       e655725d-97db-4eec-925a-b80d66ad97e8\n:END:\nWelcome\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
                    level: 1,
                    ..Default::default()
                },
                NodeFromOrg {
                    title: "Hello".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e655725d-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
                    level: 2,
                    ..Default::default()
                },
                NodeFromOrg {
                    title: "testing".to_string(),
                    parent: Some("e655725d-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 3,
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn test_node_gatherer_skipped_heading() {
        const ORG: &'static str = "
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
        let org = Org::parse(ORG);
        let document = org.document();
        let res = get_nodes_from_document(document).unwrap();
        assert_eq!(
            res,
            vec![
                NodeFromOrg {
                    title: "Hello World".to_string(),
                    parent: None,
                    uuid: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n** Hello\ntest\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
                    level: 1,
                    ..Default::default()
                },
                NodeFromOrg {
                    title: "testing".to_string(),
                    parent: Some("e655725f-97db-4eec-925a-b80d66ad97e8".to_string()),
                    uuid: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 3,
                    ..Default::default()
                }
            ]
        );
    }

    #[test]
    fn test_get_latex_header() {
        const ORG: &'static str = "
#+title: Test
#+subtitle: test
#+author: Joakim Brodén
#+filetags: :test2:test1:
#+options: date:nil author:t num:nil toc:nil
#+latex_header: \\usepackage{mathtools}
#+latex_header: \\setlength\\parindent{0pt}
#+latex_header: \\setlength{\\abovedisplayskip}{0pt}
#+latex_header: \\usepackage{parskip}
#+latex_header: \\usepackage[margin=3cm]{geometry}";
        let org = Org::parse(ORG);
        let document = org.document();
        let res = get_latex_hader_from_document(document);
        assert_eq!(
            res.unwrap(),
            vec![
                "\\usepackage{mathtools}",
                "\\setlength\\parindent{0pt}",
                "\\setlength{\\abovedisplayskip}{0pt}",
                "\\usepackage{parskip}",
                "\\usepackage[margin=3cm]{geometry}"
            ]
        );
    }
}
