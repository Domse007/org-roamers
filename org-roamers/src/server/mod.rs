use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use tower_http::cors::CorsLayer;

use crate::ServerState;
use handlers::{assets, emacs as emacs_handler, graph, health, latex, org, tags, websocket};

mod data;
mod emacs;
mod handlers;
mod services;
pub mod types;

pub async fn build_server(app_state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/", get(health::default_route))
        .route("/org", get(org::get_org_as_html_handler))
        .route("/graph", get(graph::get_graph_data_handler))
        .route("/tags", get(tags::get_tags_handler))
        .route("/latex", get(latex::get_latex_svg_handler))
        .route("/ws", get(websocket::websocket_handler))
        .route("/emacs", post(emacs_handler::emacs_handler))
        .route("/assets", get(assets::serve_assets_handler))
        .fallback(assets::fallback_handler)
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone())
}
