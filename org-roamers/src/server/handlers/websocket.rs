use std::sync::Arc;

use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
};

use crate::{client::handle_websocket, ServerState};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<ServerState>>,
) -> Response {
    let app_state_clone = app_state.clone();
    ws.on_upgrade(move |socket| handle_websocket(socket, app_state_clone))
}
