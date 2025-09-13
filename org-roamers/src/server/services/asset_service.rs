use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::server::data::{self, DataLoader};
use crate::ServerState;

pub fn default_route_content(_db: &mut ServerState, root: String, url: Option<String>) -> Response {
    let root = PathBuf::from(root);

    let rel_path = match url {
        Some(url) => PathBuf::from(url.strip_prefix("/").unwrap_or(&url)),
        None => PathBuf::from("index.html"),
    };

    let mime = match rel_path.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "html" => "text/html",
            "js" => "text/javascript",
            "css" => "text/css",
            "ico" => "image/x-icon",
            _ => {
                tracing::error!(
                    "Unsupported file extension: {:?} ({:?})",
                    rel_path.extension(),
                    rel_path
                );
                return StatusCode::NOT_FOUND.into_response();
            }
        },
        _ => {
            tracing::error!("No file extension provided.");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let asset_loader = data::get_loader(root);

    let bytes = match asset_loader.load(&rel_path) {
        Some(bytes) => {
            tracing::info!("Serving file {rel_path:?}");
            bytes
        }
        None => {
            tracing::error!("File not found: {rel_path:?}");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.parse().unwrap());

    (StatusCode::OK, headers, bytes).into_response()
}

pub fn serve_assets<P: AsRef<Path>>(root: P, file: String) -> Response {
    let file_path = root.as_ref().join(&file);

    let mime = match PathBuf::from(&file).extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "jpeg" | "jpg" => "image/jpeg",
            "png" => "image/png",
            _ => return StatusCode::NOT_FOUND.into_response(),
        },
        _ => {
            tracing::error!("No file extension provided.");
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    let mut buffer = vec![];
    let mut source_file = match File::open(&file_path) {
        Ok(file) => file,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    if source_file.read_to_end(&mut buffer).is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.parse().unwrap());

    (StatusCode::OK, headers, buffer).into_response()
}