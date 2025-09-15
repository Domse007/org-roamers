use axum::{extract::State, response::Response};

use crate::server::services::asset_service;
use crate::server::AppState;

pub async fn default_route(State(app_state): State<AppState>) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, _) = *state;
    let conf = server_state.static_conf.root.to_string();
    asset_service::default_route_content(server_state, conf, None)
}
