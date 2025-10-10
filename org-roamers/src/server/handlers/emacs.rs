use std::{collections::HashMap, path::PathBuf, sync::Arc};

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::server::types::RoamID;
use crate::{
    server::emacs::{route_emacs_traffic, EmacsRequest},
    ServerState,
};

pub async fn emacs_handler(
    AxumQuery(params): AxumQuery<HashMap<String, String>>,
    State(app_state): State<Arc<ServerState>>,
) -> Response {
    tracing::debug!("Emacs request with params: {:?}", params);

    match route_emacs_traffic(params) {
        Ok(req) => {
            match req {
                EmacsRequest::BufferOpened(id) => {
                    let roam_id: RoamID = id.clone().into();

                    // Notify all WebSocket clients about node visit
                    let message =
                        crate::client::message::WebSocketMessage::NodeVisited { node_id: roam_id };
                    app_state.broadcast_to_websockets(message);
                }
                EmacsRequest::BufferModified(file) => {
                    // Notify all WebSocket clients about pending changes
                    let message = crate::client::message::WebSocketMessage::BufferModified;
                    app_state.broadcast_to_websockets(message);

                    app_state.cache.invalidate(PathBuf::from(file));
                }
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}
