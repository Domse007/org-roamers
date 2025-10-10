use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{server::services::asset_service, ServerState};

pub async fn serve_assets_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<Arc<ServerState>>,
) -> Response {
    match params.get("file") {
        Some(path) => {
            let org_roam_path = app_state.cache.path();
            asset_service::serve_assets(org_roam_path, path.clone())
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn fallback_handler(
    uri: axum::http::Uri,
    State(app_state): State<Arc<ServerState>>,
) -> Response {
    let conf = app_state
        .config
        .org_roamers_root
        .to_str()
        .unwrap()
        .to_string();
    asset_service::default_route_content(app_state, conf, Some(uri.path().to_string()))
}
