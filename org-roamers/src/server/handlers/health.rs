use std::sync::Arc;

use axum::{extract::State, response::Response};

use crate::{server::services::asset_service, ServerState};

pub async fn default_route(State(app_state): State<Arc<ServerState>>) -> Response {
    let conf = app_state
        .config
        .org_roamers_root
        .to_string_lossy()
        .to_string();
    asset_service::default_route_content(app_state, conf, None)
}
