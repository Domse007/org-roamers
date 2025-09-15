use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use orgize::Org;

use std::path::PathBuf;

use crate::latex;
use crate::server::AppState;
use crate::sqlite::helpers;
use crate::transform::export::HtmlExport;
use crate::transform::subtree::Subtree;
use crate::FileProcessingGuard;

pub fn get_latex_svg_by_index(
    app_state: AppState,
    id: String,
    latex_index: usize,
    color: String,
    scope: String,
) -> Response {
    tracing::info!(
        "LaTeX request: id={}, index={}, color={}, scope={}",
        id,
        latex_index,
        color,
        scope
    );

    // First, find the node and get the file path without holding the lock too long
    let (file_path, node_id, db_result) = {
        let mut state = app_state.lock().unwrap();
        let (ref mut server_state, _) = *state;

        // Find the node by ID
        let [_title, node_id, file] =
            match helpers::get_all_nodes(server_state.sqlite.connection(), ["title", "id", "file"])
                .into_iter()
                .find(|[_, node, _]| node.contains(&id))
            {
                Some(node) => node,
                None => {
                    tracing::error!("Node not found: {}", id);
                    return (StatusCode::NOT_FOUND, "Node not found").into_response();
                }
            };

        let file = file.replace('"', "");
        (
            PathBuf::from(&file),
            node_id,
            (
                file,
                server_state.org_roam_db_path.clone(),
                server_state.html_export_settings.clone(),
            ),
        )
    }; // Lock is released here

    // Create file processing guard to prevent watcher conflicts
    let _guard = match FileProcessingGuard::new(app_state, file_path.clone()) {
        Ok(guard) => guard,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not acquire file processing lock",
            )
                .into_response();
        }
    };

    let (file, org_roam_db_path, html_export_settings) = db_result;

    let contents = match std::fs::read_to_string(&file) {
        Ok(f) => f,
        Err(err) => {
            let error_msg = format!("Could not read file contents: {err}");
            return (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response();
        }
    };

    let contents = if scope == "file" {
        contents
    } else {
        Subtree::get(node_id.into(), contents.as_str()).unwrap_or(contents)
    };

    // Extract LaTeX blocks from the content
    let relative_file = std::path::PathBuf::from(&file)
        .strip_prefix(&org_roam_db_path)
        .unwrap_or(std::path::Path::new(&file))
        .to_string_lossy()
        .to_string();

    let mut handler = HtmlExport::new(&html_export_settings, relative_file);
    Org::parse(contents).traverse(&mut handler);

    let (_, _, latex_blocks) = handler.finish();

    tracing::info!("Found {} LaTeX blocks in content", latex_blocks.len());

    // Get the specific LaTeX block
    let latex_content = match latex_blocks.get(latex_index) {
        Some(content) => {
            tracing::info!(
                "Found LaTeX block {}: {}",
                latex_index,
                content.chars().take(100).collect::<String>()
            );
            content
        }
        None => {
            let error_msg = format!(
                "LaTeX block index {} not found (total: {})",
                latex_index,
                latex_blocks.len()
            );
            tracing::error!("{}", error_msg);
            return (StatusCode::NOT_FOUND, error_msg).into_response();
        }
    };

    // Render the LaTeX
    let svg = latex::get_image_with_ctx(latex_content.clone(), color, &file);

    match svg {
        Ok(svg) => {
            let mut headers = HeaderMap::new();
            headers.insert("content-type", "image/svg+xml".parse().unwrap());
            (StatusCode::OK, headers, svg).into_response()
        }
        Err(err) => {
            let error_msg = format!("Could not generate svg: {:#?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response()
        }
    }
}
