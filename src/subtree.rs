//! Org utils to extract subtrees.

use orgize::{
    export::{Container, Event, TraversalContext, Traverser},
    rowan::ast::AstNode,
    Org,
};

use crate::api::types::RoamID;

/// Get the subtree of a given node.
/// ```rust
/// use org_roamers::subtree::Subtree;
///
/// let org = concat!(
///     "* Hello World\n:PROPERTIES:\n:ID:       ",
///     "aa\n:END:\n aa"
/// );
/// let sub = Subtree::new("aa".into(), org);
/// assert_eq!(sub.unwrap(), org);
/// ```
pub struct Subtree {
    on: RoamID,
    subtree: Option<String>,
}

impl Subtree {
    /// Construct and get a subtree.
    pub fn new(on: RoamID, org: &str) -> Option<String> {
        let org = Org::parse(org);
        let mut traverser = Subtree { on, subtree: None };
        org.traverse(&mut traverser);
        traverser.subtree
    }
}

impl Traverser for Subtree {
    fn event(&mut self, event: Event, _: &mut TraversalContext) {
        if self.subtree.is_some() {
            return;
        }
        match event {
            Event::Enter(Container::Document(document)) => {
                if let Some(properties) = document.properties() {
                    if let Some(id) = properties.get("ID") {
                        if id == self.on.id() {
                            self.subtree = Some(document.syntax().clone_subtree().to_string());
                        }
                    }
                }
            }
            Event::Enter(Container::Headline(headline)) => {
                if let Some(properties) = headline.properties() {
                    if let Some(id) = properties.get("ID") {
                        if id == self.on.id() {
                            self.subtree = Some(headline.syntax().clone_subtree().to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Subtree;
    #[test]
    fn test_subtree() {
        let org = concat!(
            "* Hello World\n",
            ":PROPERTIES:\n",
            ":ID:       e655725f-97db-4eec-925a-b80d66ad97e8\n",
            ":END:\n",
            "Welcome\n",
            "** Hello\n",
            "test\n",
            "*** testing\n",
            ":PROPERTIES:\n",
            ":ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n",
            ":END:\n",
            "some text\n"
        );
        let res = Subtree::new("e6557233-97db-4eec-925a-b80d66ad97e8".into(), org).unwrap();
        let expected = concat!(
            "*** testing\n",
            ":PROPERTIES:\n",
            ":ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n",
            ":END:\n",
            "some text\n"
        );
        assert_eq!(res, expected);
    }
    #[test]
    fn test_subtree_nested() {
        let org = concat!(
            "* Hello World\n",
            "Welcome\n",
            "** Hello\n",
            ":PROPERTIES:\n",
            ":ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n",
            ":END:\n",
            "test\n",
            "* testing\n",
            "some text\n"
        );
        let res = Subtree::new("e6557233-97db-4eec-925a-b80d66ad97e8".into(), org).unwrap();
        let expected = concat!(
            "** Hello\n",
            ":PROPERTIES:\n",
            ":ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n",
            ":END:\n",
            "test\n",
        );
        assert_eq!(res, expected);
    }
    #[test]
    fn test_subtree_with_children() {
        let org = concat!(
            "* Hello World\n",
            ":PROPERTIES:\n",
            ":ID:       e6557233-97db-4eec-925a-b80d66ad97e8\n",
            ":END:\n",
            "** Hello\n",
            "Welcome\n",
            "*** test\n",
            "** testing\n",
            "some text\n"
        );
        let res = Subtree::new("e6557233-97db-4eec-925a-b80d66ad97e8".into(), org).unwrap();
        assert_eq!(res, org);
    }
}
