use std::{collections::HashMap, path::PathBuf};

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::emacs::{route_emacs_traffic, EmacsRequest};
use crate::server::types::RoamID;
use crate::server::AppState;

pub async fn emacs_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<AppState>,
) -> Response {
    tracing::debug!("Emacs request with params: {:?}", params);

    match route_emacs_traffic(params) {
        Ok(req) => {
            let mut state = app_state.lock().unwrap();
            let ref mut server_state = *state;

            match req {
                EmacsRequest::BufferOpened(id) => {
                    let roam_id: RoamID = id.clone().into();
                    server_state
                        .dynamic_state
                        .update_working_id(roam_id.clone());

                    // Notify all WebSocket clients about node visit
                    let message =
                        crate::client::message::WebSocketMessage::NodeVisited { node_id: roam_id };
                    server_state.broadcast_to_websockets(message);
                }
                EmacsRequest::BufferModified(file) => {
                    // Notify all WebSocket clients about pending changes
                    let message = crate::client::message::WebSocketMessage::BufferModified;
                    server_state.broadcast_to_websockets(message);

                    server_state.cache.invalidate(PathBuf::from(file));
                }
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}
