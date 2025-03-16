use std::{env, process::ExitCode};

use anyhow::Result;
use axum::routing::get;
use axum::Router;
use org_roamers::database::Database;
use orgize::Org;
use tracing::info;
use std::fs::{self, DirEntry};
use std::path::Path;

use org_roamers::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_ansi(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .with_line_number(true)
        .init();

    info!("Starting up org-roamers standalone application");

    let app = Server::new();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9298").await?;
    axum::serve(listener, app.service()).await?;

    Ok(())
}
