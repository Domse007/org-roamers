use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
};

use crate::client::handle_websocket;
use crate::server::AppState;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> Response {
    let app_state_clone = app_state.clone();
    ws.on_upgrade(move |socket| handle_websocket(socket, app_state_clone))
}
