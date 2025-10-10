use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::server::services::graph_service;
use crate::ServerState;

pub async fn get_graph_data_handler(
    State(app_state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let sqlite = &app_state.sqlite;
    graph_service::get_graph_data(sqlite).await
}
