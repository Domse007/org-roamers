use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RoamID(pub String);

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RoamLink {
    pub from: RoamID,
    pub to: RoamID,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct RoamNode {
    pub title: String,
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

mod tests {
    use super::*;

    #[test]
    fn test_graph_data_serialization() {
        let data = GraphData {
            nodes: vec![
                RoamNode {
                    title: "Rust".to_string(),
                    id: RoamID("a64477aa-d900-476d-b500-b8ab0b03c17d".to_string()),
                },
                RoamNode {
                    title: "Vec<T>".to_string(),
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
}
