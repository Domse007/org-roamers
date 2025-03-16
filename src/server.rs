//! # Web server for HTML WebUI

use axum::{response::IntoResponse, routing::get, serve::Listener, Router};
use anyhow::Result;

pub struct Server (Router);

impl Server {
    pub fn new() -> Self {
        Self(Router::new().route("/", get(Handler::get_index)))
    }

    pub fn service(self) -> Router {
        self.0
    }
}

struct Handler {}

impl Handler {
    // TODO: serve HTML
    async fn get_index() -> impl IntoResponse {
        "org-roamers"
    }
}
