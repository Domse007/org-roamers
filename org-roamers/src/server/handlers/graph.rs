use axum::{extract::State, response::IntoResponse};

use crate::server::services::graph_service;
use crate::server::AppState;

pub async fn get_graph_data_handler(State(app_state): State<AppState>) -> impl IntoResponse {
    let mut state = app_state.lock().unwrap();
    let ref mut server_state = *state;
    graph_service::get_graph_data(server_state)
}
