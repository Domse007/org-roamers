use std::collections::HashMap;

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::services::asset_service;
use crate::server::AppState;

pub async fn serve_assets_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    match params.get("file") {
        Some(path) => {
            let state = app_state.lock().unwrap();
            let (ref server_state, _) = *state;
            let org_roam_path = server_state.cache.path();
            asset_service::serve_assets(org_roam_path, path.clone())
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn fallback_handler(uri: axum::http::Uri, State(app_state): State<AppState>) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, _) = *state;
    let conf = server_state
        .config
        .org_roamers_root
        .to_str()
        .unwrap()
        .to_string();
    asset_service::default_route_content(server_state, conf, Some(uri.path().to_string()))
}
