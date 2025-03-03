use std::{fs::File, io::Read, path::Path};

use orgize::{
    ast::{Document, Headline},
    Org,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtractedNode {
    pub(crate) title: String,
    pub(crate) id: String,
    pub(crate) content: String,
    pub(crate) level: usize,
}

pub fn get_nodes_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<ExtractedNode>> {
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;

    let org = Org::parse(&content);
    let document = org.document();

    get_nodes_from_document(document)
}

pub fn get_nodes_from_document(document: Document) -> anyhow::Result<Vec<ExtractedNode>> {
    let mut nodes = Vec::new();

    // parsing #+title
    if let Some(title) = document.title() {
        if let Some(properties) = document.properties() {
            if let Some(id) = properties.get("ID") {
                let id = id.to_string();
                // let content = match document.section() {
                //     Some(section) => section.raw(),
                //     None => String::new(),
                // };
                let content = document.raw();
                nodes.push(ExtractedNode {
                    title,
                    id,
                    content,
                    level: 0,
                })
            }
        }
    }

    // parsing headlines and recursively looking for nodes
    let mut stack = document.headlines().collect::<Vec<Headline>>();

    while let Some(headline) = stack.pop() {
        if let Some(properties) = headline.properties() {
            if let Some(id) = properties.get("ID") {
                let id = id.to_string();
                // TODO: this is wrong.
                let title = headline.title_raw();
                let level = headline.level();

                let mut content = match headline.section() {
                    Some(section) => section.raw(),
                    None => String::new(),
                };
                let subheading = headline
                    .headlines()
                    .map(|headline| headline.raw())
                    .collect::<String>();

                content.push_str(&subheading);

                nodes.push(ExtractedNode {
                    title,
                    id,
                    content,
                    level,
                })
            }
        }

        // recursively add children to stack
        for hl in headline.headlines() {
            stack.push(hl);
        }
    }

    Ok(nodes)
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
                ExtractedNode {
                    title: "Hello World".to_string(),
                    id: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: ORG.to_string(),
                    level: 0,
                },
                ExtractedNode {
                    title: "testing".to_string(),
                    id: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 1,
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
                ExtractedNode {
                    title: "testing".to_string(),
                    id: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "some text\n".to_string(),
                    level: 1,
                },
		ExtractedNode {
                    title: "Hello World".to_string(),
                    id: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n** Hello\n:PROPERTIES:\n:ID:       e655725d-97db-4eec-925a-b80d66ad97e8\n:END:\nWelcome\n".to_string(),
                    level: 1,
                },
                ExtractedNode {
                    title: "Hello".to_string(),
                    id: "e655725d-97db-4eec-925a-b80d66ad97e8".to_string(),
                    content: "Welcome\n".to_string(),
                    level: 2,
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
		ExtractedNode {
		    title: "Hello World".to_string(),
		    id: "e655725f-97db-4eec-925a-b80d66ad97e8".to_string(),
		    content: "Welcome\n** Hello\n:PROPERTIES:\n:ID:       e655725d-97db-4eec-925a-b80d66ad97e8\n:END:\nWelcome\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
		    level: 1,
		},
		ExtractedNode {
		    title: "Hello".to_string(),
		    id: "e655725d-97db-4eec-925a-b80d66ad97e8".to_string(),
		    content: "Welcome\n*** testing\n:PROPERTIES:\n:ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n:END:\nsome text\n".to_string(),
		    level: 2
		},
		ExtractedNode {
		    title: "testing".to_string(),
		    id: "e6557233-97db-4eec-925a-b80d66ad97e8".to_string(),
		    content: "some text\n".to_string(),
		    level: 3
		}
	    ]
	);
    }
}
