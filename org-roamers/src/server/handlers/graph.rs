use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::server::services::graph_service;
use crate::ServerState;

#[derive(Deserialize)]
pub struct GraphParams {
    tags: Option<String>,
    exclude: Option<String>,
}

impl GraphParams {
    pub fn parse_tags(&self) -> (Option<Vec<String>>, Option<Vec<String>>) {
        let filter_tags = self
            .tags
            .as_ref()
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());
        let exclude_tags = self
            .exclude
            .as_ref()
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect());
        (filter_tags, exclude_tags)
    }
}

pub async fn get_graph_data_handler(
    State(app_state): State<Arc<ServerState>>,
    Query(params): Query<GraphParams>,
) -> impl IntoResponse {
    let sqlite = &app_state.sqlite;
    let (filter_tags, exclude_tags) = params.parse_tags();
    graph_service::get_graph_data(sqlite, filter_tags, exclude_tags).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tags_both_none() {
        let params = GraphParams {
            tags: None,
            exclude: None,
        };
        let (include, exclude) = params.parse_tags();
        assert!(include.is_none());
        assert!(exclude.is_none());
    }

    #[test]
    fn test_parse_tags_single_include() {
        let params = GraphParams {
            tags: Some("rust".to_string()),
            exclude: None,
        };
        let (include, exclude) = params.parse_tags();
        assert_eq!(include, Some(vec!["rust".to_string()]));
        assert!(exclude.is_none());
    }

    #[test]
    fn test_parse_tags_multiple_include() {
        let params = GraphParams {
            tags: Some("rust,emacs,org".to_string()),
            exclude: None,
        };
        let (include, exclude) = params.parse_tags();
        assert_eq!(
            include,
            Some(vec![
                "rust".to_string(),
                "emacs".to_string(),
                "org".to_string()
            ])
        );
        assert!(exclude.is_none());
    }

    #[test]
    fn test_parse_tags_with_whitespace() {
        let params = GraphParams {
            tags: Some("rust , emacs , org".to_string()),
            exclude: None,
        };
        let (include, exclude) = params.parse_tags();
        assert_eq!(
            include,
            Some(vec![
                "rust".to_string(),
                "emacs".to_string(),
                "org".to_string()
            ])
        );
        assert!(exclude.is_none());
    }

    #[test]
    fn test_parse_tags_single_exclude() {
        let params = GraphParams {
            tags: None,
            exclude: Some("archived".to_string()),
        };
        let (include, exclude) = params.parse_tags();
        assert!(include.is_none());
        assert_eq!(exclude, Some(vec!["archived".to_string()]));
    }

    #[test]
    fn test_parse_tags_both_include_and_exclude() {
        let params = GraphParams {
            tags: Some("rust,emacs".to_string()),
            exclude: Some("archived,wip".to_string()),
        };
        let (include, exclude) = params.parse_tags();
        assert_eq!(include, Some(vec!["rust".to_string(), "emacs".to_string()]));
        assert_eq!(
            exclude,
            Some(vec!["archived".to_string(), "wip".to_string()])
        );
    }

    #[test]
    fn test_parse_tags_empty_strings() {
        let params = GraphParams {
            tags: Some("".to_string()),
            exclude: Some("".to_string()),
        };
        let (include, exclude) = params.parse_tags();
        assert_eq!(include, Some(vec!["".to_string()]));
        assert_eq!(exclude, Some(vec!["".to_string()]));
    }
}
