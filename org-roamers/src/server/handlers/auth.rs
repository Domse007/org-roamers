use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::ServerState;

const SESSION_USER_KEY: &str = "username";

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub username: String,
}

#[derive(Serialize)]
pub struct SessionInfo {
    pub authenticated: bool,
    pub username: Option<String>,
}

/// POST /api/login
/// Authenticate user and create session
pub async fn login_handler(
    State(state): State<Arc<ServerState>>,
    session: Session,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    use tracing::{info, warn};

    // Check if auth is enabled
    let user_store = state
        .user_store
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    // Verify credentials
    if user_store.verify(&credentials.username, &credentials.password) {
        // Store username in session
        session
            .insert(SESSION_USER_KEY, credentials.username.clone())
            .await
            .map_err(|e| {
                tracing::error!("Failed to insert session: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        info!("Login successful for user: {}", credentials.username);

        Ok(Json(LoginResponse {
            success: true,
            username: credentials.username,
        }))
    } else {
        warn!("Login failed for user: {}", credentials.username);
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// POST /api/logout
/// Destroy session and logout user
pub async fn logout_handler(session: Session) -> Result<StatusCode, StatusCode> {
    use tracing::info;

    // Get username before clearing session (for logging)
    let username: Option<String> = session
        .get(SESSION_USER_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Clear session
    session
        .delete()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(user) = username {
        info!("Logout successful for user: {}", user);
    }

    Ok(StatusCode::OK)
}

/// GET /api/session
/// Check if user is authenticated and return session info
pub async fn check_session_handler(session: Session) -> Result<Json<SessionInfo>, StatusCode> {
    let username: Option<String> = session
        .get(SESSION_USER_KEY)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SessionInfo {
        authenticated: username.is_some(),
        username,
    }))
}
