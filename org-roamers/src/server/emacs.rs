use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;

pub enum EmacsRequest {
    /// Arg: id where point is in
    BufferOpened(String),
    /// Arg: string modified of filename
    BufferModified(String),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum EmacsRequestError {
    #[error("No id was provided")]
    NoIDProvided,
    #[error("No file provided")]
    NoFileProvided,
    #[error("No task provided")]
    NoTaskProvided,
    #[error("Unsupported task: {0}")]
    UnsupportedTask(String),
}

impl IntoResponse for EmacsRequestError {
    fn into_response(self) -> Response {
        tracing::error!("{self:?}");
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub fn route_emacs_traffic(
    params: HashMap<String, String>,
) -> Result<EmacsRequest, EmacsRequestError> {
    match params.get("task") {
        Some(task) if task == "opened" => match params.get("id") {
            Some(id) => Ok(EmacsRequest::BufferOpened(id.clone())),
            None => Err(EmacsRequestError::NoIDProvided),
        },
        Some(task) if task == "modified" => match params.get("file") {
            Some(file) => Ok(EmacsRequest::BufferModified(file.clone())),
            None => Err(EmacsRequestError::NoFileProvided),
        },
        Some(task) => Err(EmacsRequestError::UnsupportedTask(task.clone())),
        None => Err(EmacsRequestError::NoTaskProvided),
    }
}
