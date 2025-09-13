use std::collections::HashMap;

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::services::search_service;
use crate::server::AppState;

pub async fn search_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, _) = *state;

    match params.get("q") {
        Some(query) => search_service::search(server_state, query.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}