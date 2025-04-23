use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RoamID(String);

impl RoamID {
    fn id(&self) -> &str {
        &self.0
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
pub struct RoamTitle(pub String);

impl RoamTitle {
    fn title(&self) -> &str {
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
///     },
///     {
///       "title": "Vec<T>",
///       "id": "bcb77e31-b4c6-4cf9-a05d-47b766349e57"
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
                },
                RoamNode {
                    title: RoamTitle("Vec<T>".to_string()),
                    id: RoamID("bcb77e31-b4c6-4cf9-a05d-47b766349e57".to_string()),
                },
            ],
            links: vec![RoamLink {
                from: RoamID("bcb77e31-b4c6-4cf9-a05d-47b766349e57".to_string()),
                to: RoamID("a64477aa-d900-476d-b500-b8ab0b03c17d".to_string()),
            }],
        };

        let serialized = concat!(
            r#"{"nodes":[{"title":"Rust","id":"a64477aa-d900-476d-b500-b8ab0b03c17d"},"#,
            r#"{"title":"Vec<T>","id":"bcb77e31-b4c6-4cf9-a05d-47b766349e57"}],"#,
            r#""links":[{"from":"bcb77e31-b4c6-4cf9-a05d-47b766349e57","#,
            r#""to":"a64477aa-d900-476d-b500-b8ab0b03c17d"}]}"#
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
}
