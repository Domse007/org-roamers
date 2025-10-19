use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::config::AssetPolicy;
use crate::server::data::{self, DataLoader};
use crate::ServerState;

pub fn default_route_content(_db: Arc<ServerState>, root: String, url: Option<String>) -> Response {
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
            // Font file support for KaTeX
            "woff2" => "font/woff2",
            "woff" => "font/woff",
            "ttf" => "font/ttf",
            "otf" => "font/otf",
            "eot" => "application/vnd.ms-fontobject",
            // Additional web asset types
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "json" => "application/json",
            "xml" => "application/xml",
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

    // Add caching headers - only apply aggressive caching in release builds
    if cfg!(debug_assertions) {
        // Development mode: minimal caching to avoid stale content
        headers.insert(
            "cache-control",
            "no-cache, must-revalidate".parse().unwrap(),
        );
        tracing::debug!(
            "Serving {} with no-cache headers (development mode)",
            rel_path.display()
        );
    } else {
        // Release mode: optimized caching for better performance
        match rel_path.extension().and_then(|ext| ext.to_str()) {
            Some("woff2") | Some("woff") | Some("ttf") | Some("otf") | Some("eot") => {
                // Font files can be cached for a long time (1 year)
                headers.insert(
                    "cache-control",
                    "public, max-age=31536000, immutable".parse().unwrap(),
                );
            }
            Some("css") | Some("js") => {
                // CSS and JS can be cached for a moderate time (1 day)
                headers.insert("cache-control", "public, max-age=86400".parse().unwrap());
            }
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") | Some("webp")
            | Some("ico") => {
                // Images can be cached for a moderate time (1 week)
                headers.insert("cache-control", "public, max-age=604800".parse().unwrap());
            }
            _ => {
                // Default caching for other files (1 hour)
                headers.insert("cache-control", "public, max-age=3600".parse().unwrap());
            }
        }
        tracing::debug!(
            "Serving {} with optimized caching headers (release mode)",
            rel_path.display()
        );
    }

    (StatusCode::OK, headers, bytes).into_response()
}

pub fn serve_assets<P: AsRef<Path>>(root: P, file: PathBuf, asset_policy: AssetPolicy) -> Response {
    let file_path = match asset_policy {
        AssetPolicy::AllowAll => file.clone(),
        AssetPolicy::AllowChildrenOfRoot => root.as_ref().join(&file),
        AssetPolicy::ForbidAll => {
            tracing::warn!("Cannot serve {file:?} because of access policy restrictions.");
            return StatusCode::from_u16(403).unwrap().into_response();
        }
    };

    let mime = match file.extension() {
        Some(extension) => match extension.to_str().unwrap() {
            "jpeg" | "jpg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            // Font file support for KaTeX
            "woff2" => "font/woff2",
            "woff" => "font/woff",
            "ttf" => "font/ttf",
            "otf" => "font/otf",
            "eot" => "application/vnd.ms-fontobject",
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

    // Add caching headers - only apply aggressive caching in release builds
    if cfg!(debug_assertions) {
        // Development mode: minimal caching to avoid stale content
        headers.insert(
            "cache-control",
            "no-cache, must-revalidate".parse().unwrap(),
        );
        tracing::debug!(
            "Serving asset {} with no-cache headers (development mode)",
            file.display()
        );
    } else {
        // Release mode: optimized caching for better performance
        match PathBuf::from(&file)
            .extension()
            .and_then(|ext| ext.to_str())
        {
            Some("woff2") | Some("woff") | Some("ttf") | Some("otf") | Some("eot") => {
                // Font files can be cached for a long time
                headers.insert(
                    "cache-control",
                    "public, max-age=31536000, immutable".parse().unwrap(),
                );
            }
            Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") | Some("webp") => {
                // Images can be cached for a moderate time
                headers.insert("cache-control", "public, max-age=604800".parse().unwrap());
            }
            _ => {
                // Default caching for other files
                headers.insert("cache-control", "public, max-age=3600".parse().unwrap());
            }
        }
        tracing::debug!(
            "Serving asset {} with optimized caching headers (release mode)",
            file.display()
        );
    }

    (StatusCode::OK, headers, buffer).into_response()
}
