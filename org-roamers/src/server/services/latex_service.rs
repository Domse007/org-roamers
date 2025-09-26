use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use orgize::Org;


use crate::latex;
use crate::server::AppState;
use crate::transform::export::HtmlExport;
use crate::StaticServerConfiguration;

pub fn get_latex_svg_by_index(
    config: &StaticServerConfiguration,
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


    let mut state = app_state.lock().unwrap();
    let (ref mut server_state, _) = *state;

    let content = server_state.cache.retrieve(&id.into()).unwrap().content();

    let mut handler = HtmlExport::new(&server_state.html_export_settings, String::new());
    Org::parse(content).traverse(&mut handler);

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
    let svg =
        latex::get_image_with_ctx(&config.latex_config, latex_content.clone(), color, content);

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
