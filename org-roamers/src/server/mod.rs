use std::sync::Arc;

use crate::{
    config::{AuthConfig, SessionExpiryMode},
    ServerState,
};
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use handlers::{assets, auth, emacs as emacs_handler, graph, health, latex, org, tags, websocket};
use time::Duration;
use tower_http::cors::CorsLayer;
use tower_sessions::{session_store::ExpiredDeletion, Expiry, SessionManagerLayer};
use tracing::info;

mod data;
mod emacs;
mod handlers;
mod middleware;
mod services;
pub mod types;

pub async fn build_server_with_auth(
    app_state: Arc<ServerState>,
    auth_config: &AuthConfig,
) -> Router {
    info!("Setting up authentication middleware...");

    // Create session store from existing pool (clone for session use)
    let session_store = crate::auth::session_store::create_session_store(app_state.sqlite.clone())
        .await
        .expect("Failed to initialize session store");

    let expiry = match auth_config.session.expiry_mode {
        SessionExpiryMode::OnInactivity => Expiry::OnInactivity(Duration::hours(
            auth_config.session.expiry_duration_hours as i64,
        )),
        SessionExpiryMode::BrowserSession => Expiry::OnSessionEnd,
    };

    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(auth_config.session.secure_cookie)
        .with_expiry(expiry);

    info!(
        "Session expiry: {:?}, secure_cookie: {}",
        auth_config.session.expiry_mode, auth_config.session.secure_cookie
    );

    // Spawn cleanup task
    let cleanup_interval =
        tokio::time::Duration::from_secs(auth_config.session.cleanup_interval_minutes * 60);

    tokio::task::spawn(async move {
        let interva_min = cleanup_interval.as_secs() / 60;
        info!("Starting session cleanup task (interval: {}m)", interva_min);
        session_store
            .continuously_delete_expired(cleanup_interval)
            .await
    });

    let num_users = app_state
        .user_store
        .as_ref()
        .map(|s| s.user_count())
        .unwrap_or(0);
    info!("Authentication enabled with {} user(s)", num_users);

    // Build protected and public routers separately, then merge
    // Protected routes - API endpoints that require authentication
    let protected = Router::new()
        .route("/assets", get(assets::serve_assets_handler))
        .route("/org", get(org::get_org_as_html_handler))
        .route("/graph", get(graph::get_graph_data_handler))
        .route("/tags", get(tags::get_tags_handler))
        .route("/latex", get(latex::get_latex_svg_handler))
        .route("/ws", get(websocket::websocket_handler))
        .route("/emacs", post(emacs_handler::emacs_handler))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth::require_auth,
        ));

    // Public routes - static assets and auth endpoints (no auth required)
    let public = Router::new()
        .route("/", get(health::default_route))
        .route("/api/login", post(auth::login_handler))
        .route("/api/logout", post(auth::logout_handler))
        .route("/api/session", get(auth::check_session_handler))
        .fallback(assets::fallback_handler);

    public
        .merge(protected)
        .layer(session_layer)
        .with_state(app_state.clone())
}

pub async fn build_server(app_state: Arc<ServerState>) -> Router {
    // Add authentication if enabled
    if let Some(auth_config) = &app_state.config.authentication {
        if auth_config.enabled {
            return build_server_with_auth(app_state.clone(), auth_config).await;
        }
    }

    // No authentication - return router without session layer
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
        .layer(CorsLayer::permissive().allow_credentials(true))
        .with_state(app_state.clone())
}
