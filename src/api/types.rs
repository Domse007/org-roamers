use rouille::Response;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct RoamID(String);

impl RoamID {
    pub fn id(&self) -> &str {
        &self.0
    }

    /// ```rust
    /// use org_roamers::api::types::RoamID;
    ///
    /// let id: RoamID = "t t".into();
    /// assert_eq!(id.with_quotes(3), "\"\"\"t t\"\"\"");
    /// ```
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

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
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

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RoamLink {
    pub from: RoamID,
    pub to: RoamID,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RoamNode {
    pub title: RoamTitle,
    pub id: RoamID,
    pub parent: RoamID,
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

impl Into<Response> for GraphData {
    fn into(self) -> Response {
        Response::json(&serde_json::to_string(&self).unwrap())
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Hash, Eq)]
pub struct SearchResponseElement {
    pub display: String,
    pub id: RoamID,
    pub tags: Vec<String>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponseProvider {
    pub source: String,
    pub results: Vec<SearchResponseElement>,
}

/// # Example
/// ```json
/// {
///   "providers": [{
///       "source": "sqlite",
///       "results": [{
///           "display": "Vec<T>",
///           "id": "bcb77e31-b4c6-4cf9-a05d-47b766349e57"
///       }]
///   }]
/// }
/// ```
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub providers: Vec<SearchResponseProvider>,
}

impl Into<Response> for SearchResponse {
    fn into(self) -> Response {
        Response::json(&serde_json::to_string(&self).unwrap())
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
                },
                RoamNode {
                    title: RoamTitle("Vec<T>".to_string()),
                    id: RoamID("bcb77e31-b4c6-4cf9-a05d-47b766349e57".to_string()),
                    parent: RoamID("".to_string()),
                },
            ],
            links: vec![RoamLink {
                from: RoamID("bcb77e31-b4c6-4cf9-a05d-47b766349e57".to_string()),
                to: RoamID("a64477aa-d900-476d-b500-b8ab0b03c17d".to_string()),
            }],
        };

        let serialized = concat!(
            "{\"nodes\":[{\"title\":\"Rust\",\"id\":\"a64477aa-d900-476d-b500-b8ab0b03c17d\",",
            "\"parent\":\"\"},{\"title\":\"Vec<T>\",\"id\":\"bcb77e31-b4c6-4cf9-a05d-47b766349e57\",",
            "\"parent\":\"\"}],\"links\":[{\"from\":\"bcb77e31-b4c6-4cf9-a05d-47b766349e57\",",
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
    fn test_search_response_serialization() {
        let data = SearchResponse {
            providers: vec![SearchResponseProvider {
                source: "sqlite".to_string(),
                results: vec![SearchResponseElement {
                    display: "Vec<T>".to_string(),
                    id: RoamID("bcb77e31-b4c6-4cf9-a05d-47b766349e57".to_string()),
                    tags: vec!["rust".to_string()],
                }],
            }],
        };

        let expected = concat!(
            "{\"providers\":[{\"source\":\"sqlite\",\"results\":[{\"display\"",
            ":\"Vec<T>\",\"id\":\"bcb77e31-b4c6-4cf9-a05d-47b766349e57\",",
            "\"tags\":[\"rust\"]}]}]}"
        );

        assert_eq!(serde_json::to_string(&data).unwrap(), expected);
    }
}
