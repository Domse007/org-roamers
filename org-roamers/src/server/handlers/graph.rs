use axum::{extract::State, response::IntoResponse};

use crate::server::services::graph_service;
use crate::server::AppState;

pub async fn get_graph_data_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let sqlite = app_state.lock().unwrap().sqlite.clone();
    graph_service::get_graph_data(&sqlite).await
}
