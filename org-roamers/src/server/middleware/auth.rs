use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tower_sessions::Session;

use crate::ServerState;

const SESSION_USER_KEY: &str = "username";

/// Middleware to require authentication
/// Checks if session contains an authenticated user
pub async fn require_auth(
    State(_state): State<Arc<ServerState>>,
    session: Session,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if user is authenticated
    let username: Option<String> = session.get(SESSION_USER_KEY).await.map_err(|e| {
        tracing::error!("Failed to get session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if username.is_none() {
        tracing::debug!("Unauthorized access attempt to protected route");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // User is authenticated, proceed
    Ok(next.run(request).await)
}
