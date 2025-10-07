use std::any::Any;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use axum::{
    routing::{get, post},
    Router,
};

use tower_http::cors::CorsLayer;

use crate::{watcher, ServerState};

pub mod data;
pub mod emacs;
pub mod handlers;
pub mod services;
pub mod types;

use handlers::{assets, emacs as emacs_handler, graph, health, latex, org, websocket};

pub type AppState = Arc<Mutex<ServerState>>;

pub struct ServerRuntime {
    handle: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>>,
    sender: Sender<()>,
}

impl ServerRuntime {
    pub fn graceful_shutdown(self) -> Result<(), Box<dyn Any + Send>> {
        self.sender.send(()).unwrap();
        match self.handle.join() {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(Box::new(e) as Box<dyn Any + Send>),
            Err(e) => Err(e),
        }
    }
}

pub fn start_server(url: String, state: ServerState) -> Result<ServerRuntime, Box<dyn Error>> {
    tracing::info!(
        "Using server configuration: {:?}",
        serde_json::to_string(&state.config)
    );

    let org_roam_db_path = state.cache.path().to_path_buf();
    let use_fs_watcher = state.config.fs_watcher;

    let app_state = Arc::new(Mutex::new(state));

    let app = Router::new()
        .route("/", get(health::default_route))
        .route("/org", get(org::get_org_as_html_handler))
        // .route("/search", get(search::search_handler))
        .route("/graph", get(graph::get_graph_data_handler))
        .route("/latex", get(latex::get_latex_svg_handler))
        .route("/ws", get(websocket::websocket_handler))
        .route("/emacs", post(emacs_handler::emacs_handler))
        .route("/assets", get(assets::serve_assets_handler))
        .fallback(assets::fallback_handler)
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            // Start file watcher with concurrency conflict resolution
            if use_fs_watcher {
                let app_state_clone = app_state.clone();
                let watch_path = org_roam_db_path.clone();
                let runtime_handle = tokio::runtime::Handle::current();

                let _watcher_handle = watcher::start_watcher_runtime(
                    app_state_clone,
                    watch_path,
                    Some(runtime_handle),
                )?;

                tracing::info!("File watcher enabled with concurrency conflict resolution");
            }

            let listener = tokio::net::TcpListener::bind(&url).await?;
            tracing::info!("Server listening on {}", url);

            // Configure server with proper keepalive and timeout settings
            let server = axum::serve(listener, app).tcp_nodelay(true);

            // Set up graceful shutdown
            tokio::select! {
                result = server => {
                    if let Err(err) = result {
                        tracing::error!("Server error: {}", err);
                        return Err(Box::new(err) as Box<dyn Error + Send + Sync>);
                    }
                },
                _ = async {
                    while shutdown_rx.try_recv().is_err() {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                } => {
                    tracing::info!("Shutdown signal received");
                }
            }
            Ok(())
        })
    });

    Ok(ServerRuntime {
        handle,
        sender: shutdown_tx,
    })
}
