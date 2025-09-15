use std::collections::HashMap;

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::services::org_service::{self, Query};
use crate::server::AppState;

pub async fn get_org_as_html_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    let scope = params
        .get("scope")
        .cloned()
        .unwrap_or_else(|| "file".to_string());

    let query = match params.get("id") {
        Some(id) => Query::ById(id.clone().into()),
        None => match params.get("title") {
            Some(title) => Query::ByTitle(title.clone().into()),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
    };

    org_service::get_org_as_html(app_state, query, scope).into_response()
}
