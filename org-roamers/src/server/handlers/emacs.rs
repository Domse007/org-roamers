use std::{collections::HashMap, path::PathBuf};

use axum::{
    extract::{Query as AxumQuery, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::diff;
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
            let (ref mut server_state, _) = *state;

            match req {
                EmacsRequest::BufferOpened(id) => {
                    let roam_id: RoamID = id.clone().into();
                    server_state
                        .dynamic_state
                        .update_working_id(roam_id.clone());

                    // Broadcast node visit via WebSocket
                    let broadcaster = server_state.websocket_broadcaster.clone();
                    tokio::spawn(async move {
                        broadcaster.broadcast_node_visited(roam_id).await;
                    });
                }
                EmacsRequest::BufferModified(file) => {
                    // Broadcast pending changes status
                    let broadcaster = server_state.websocket_broadcaster.clone();
                    let visited_node = server_state
                        .dynamic_state
                        .working_id
                        .as_ref()
                        .map(|(id, _)| id.clone());

                    tokio::spawn(async move {
                        broadcaster
                            .broadcast_status_update(
                                visited_node,
                                true,   // pending_changes = true
                                vec![], // updated_nodes will be populated by watcher
                                vec![], // updated_links will be populated by watcher
                            )
                            .await;
                    });

                    server_state.cache.invalidate(PathBuf::from(file));
                }
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(err) => err.into_response(),
    }
}
