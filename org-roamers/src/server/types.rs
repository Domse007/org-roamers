use axum::response::{IntoResponse, Json, Response};
use serde::{Deserialize, Serialize};

use crate::transform::node_builder::OrgNode;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialOrd, Ord)]
pub struct RoamID(String);

impl RoamID {
    pub fn id(&self) -> &str {
        &self.0
    }

    pub fn with_quotes(&self, num: usize) -> String {
        let quotes = "\"".repeat(num);
        format!("{}{}{}", quotes, self.0, quotes)
    }
}

impl From<&str> for RoamID {
    fn from(value: &str) -> Self {
        let mut iter = value.chars();
        let mut id = String::new();
        if let Some(c) = iter.next() {
            if c != '"' {
                id.push(c);
            }
        }
        id.push_str(iter.take_while(|c| *c != '"').collect::<String>().as_str());
        Self(id)
    }
}

impl From<String> for RoamID {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, Eq)]
pub struct RoamTitle(String);

impl RoamTitle {
    pub fn title(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RoamTitle {
    fn from(value: &str) -> Self {
        if value.starts_with('"') && value.ends_with('"') {
            RoamTitle(value[1..value.len() - 1].to_string())
        } else {
            RoamTitle(value.to_string())
        }
    }
}

impl From<String> for RoamTitle {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, Eq, Hash)]
pub struct RoamLink {
    pub from: RoamID,
    pub to: RoamID,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, Eq)]
pub struct RoamNode {
    pub title: RoamTitle,
    pub id: RoamID,
    pub parent: RoamID,
    pub num_links: usize,
}

impl From<OrgNode> for RoamNode {
    fn from(value: OrgNode) -> Self {
        Self {
            title: value.title.into(),
            id: value.uuid.into(),
            parent: value
                .parent
                .map(Into::into)
                .unwrap_or(RoamID("".to_string())),
            num_links: value.links.len(),
        }
    }
}

/// Response structure for transmitting graph information.
///
/// The rust data structure serialized to json is of the form:
/// ```json
/// {
///   "nodes": [
///     {
///       "title": "Rust",
///       "id": "a64477aa-d900-476d-b500-b8ab0b03c17d"
///       "parent": "",
///     },
///     {
///       "title": "Vec<T>",
///       "id": "bcb77e31-b4c6-4cf9-a05d-47b766349e57"
///       "parent": "",
///     }
///   ],
///   "links": [
///     {
///       "from": "bcb77e31-b4c6-4cf9-a05d-47b766349e57",
///       "to": "a64477aa-d900-476d-b500-b8ab0b03c17d"
///     }
///   ]
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<RoamNode>,
    pub links: Vec<RoamLink>,
}

impl IntoResponse for GraphData {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct OutgoingLink {
    pub display: RoamTitle,
    pub id: RoamID,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct IncomingLink {
    pub display: RoamTitle,
    pub id: RoamID,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct OrgAsHTMLResponse {
    pub org: String,
    pub tags: Vec<String>,
    pub outgoing_links: Vec<OutgoingLink>,
    pub incoming_links: Vec<IncomingLink>,
    pub latex_blocks: Vec<String>,
}

impl IntoResponse for OrgAsHTMLResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_graph_data_serialization() {
        let data = GraphData {
            nodes: vec![
                RoamNode {
                    title: RoamTitle("Rust".to_string()),
                    id: RoamID("a64477aa-d900-476d-b500-b8ab0b03c17d".to_string()),
                    parent: RoamID("".to_string()),
                    num_links: 1,
                },
                RoamNode {
                    title: RoamTitle("Vec<T>".to_string()),
                    id: RoamID("bcb77e31-b4c6-4cf9-a05d-47b766349e57".to_string()),
                    parent: RoamID("".to_string()),
                    num_links: 1,
                },
            ],
            links: vec![RoamLink {
                from: RoamID("bcb77e31-b4c6-4cf9-a05d-47b766349e57".to_string()),
                to: RoamID("a64477aa-d900-476d-b500-b8ab0b03c17d".to_string()),
            }],
        };

        let serialized = concat!(
            "{\"nodes\":[{\"title\":\"Rust\",\"id\":\"a64477aa-d900-476d-b500-b8ab0b03c17d\",",
            "\"parent\":\"\",\"num_links\":1},{\"title\":\"Vec<T>\",\"id\":\"bcb77e31-b4c6-4cf9-a05d-47b766349e57\",",
            "\"parent\":\"\",\"num_links\":1}],\"links\":[{\"from\":\"bcb77e31-b4c6-4cf9-a05d-47b766349e57\",",
            "\"to\":\"a64477aa-d900-476d-b500-b8ab0b03c17d\"}]}"
        );

        assert_eq!(serde_json::to_string(&data).unwrap(), serialized);
    }

    #[test]
    fn test_id_from() {
        let s = "\"a64477aa-d900-476d-b500-b8ab0b03c17d\"";
        assert_eq!(
            RoamID::from(s),
            RoamID("a64477aa-d900-476d-b500-b8ab0b03c17d".to_string())
        );
        let s = "a64477aa-d900-476d-b500-b8ab0b03c17d";
        assert_eq!(
            RoamID::from(s),
            RoamID("a64477aa-d900-476d-b500-b8ab0b03c17d".to_string())
        );
    }

    #[test]
    fn test_title_from() {
        let s = "\"Vec<T> in Rust\"";
        assert_eq!(RoamTitle::from(s), RoamTitle("Vec<T> in Rust".to_string()));
        let s = "Vec<T> in \"Rust\"";
        assert_eq!(
            RoamTitle::from(s),
            RoamTitle("Vec<T> in \"Rust\"".to_string())
        );
    }

    #[test]
    fn test_org_as_html_serialization() {
        let resp = OrgAsHTMLResponse {
            org: "<h1>title</h1>".to_string(),
            outgoing_links: vec![OutgoingLink {
                display: "t".into(),
                id: "id".into(),
            }],
            tags: vec![],
            incoming_links: vec![],
            latex_blocks: vec![],
        };
        let expected = concat!(
            "{\"org\":\"<h1>title</h1>\",\"tags\":[],",
            "\"outgoing_links\":[{\"display\":\"t\",\"id\":\"id\"}],",
            "\"incoming_links\":[],\"latex_blocks\":[]}"
        );
        assert_eq!(serde_json::to_string(&resp).unwrap(), expected);
    }
}
