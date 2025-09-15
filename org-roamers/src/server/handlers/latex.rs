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
    match (params.get("id"), params.get("index"), params.get("color")) {
        (Some(id), Some(index_str), Some(color)) => {
            let scope = params
                .get("scope")
                .cloned()
                .unwrap_or_else(|| "file".to_string());
            match index_str.parse::<usize>() {
                Ok(index) => latex_service::get_latex_svg_by_index(
                    app_state,
                    id.clone(),
                    index,
                    color.clone(),
                    scope,
                ),
                Err(_) => (StatusCode::BAD_REQUEST, "Invalid index parameter").into_response(),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            "Missing required parameters: id, index, color",
        )
            .into_response(),
    }
}
