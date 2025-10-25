use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};

use crate::ServerState;

pub async fn get_tags_handler(State(app_state): State<Arc<ServerState>>) -> impl IntoResponse {
    let sqlite = &app_state.sqlite;
    let tags = sqlx::query_scalar::<_, String>("SELECT DISTINCT tag FROM tags ORDER BY tag")
        .fetch_all(sqlite)
        .await
        .unwrap_or_default();
    Json(tags)
}
