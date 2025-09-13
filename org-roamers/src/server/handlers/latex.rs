use std::collections::HashMap;

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::services::latex_service;
use crate::server::AppState;

pub async fn get_latex_svg_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, _) = *state;

    match (params.get("tex"), params.get("color"), params.get("id")) {
        (Some(tex), Some(color), Some(id)) => {
            latex_service::get_latex_svg(server_state, tex.clone(), color.clone(), id.clone())
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}