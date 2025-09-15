use axum::{
    extract::{ws::WebSocketUpgrade, State},
    response::Response,
};

use crate::server::AppState;
use crate::websocket::handle_websocket;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
) -> Response {
    let broadcaster = {
        let state = app_state.lock().unwrap();
        let (ref server_state, _) = *state;
        server_state.websocket_broadcaster.clone()
    };

    let app_state_clone = app_state.clone();
    ws.on_upgrade(move |socket| handle_websocket(socket, broadcaster, app_state_clone))
}
