use rouille::{Request, Response};

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

impl EmacsRequestError {
    pub fn handle(self) -> Response {
        tracing::error!("{self:?}");
        Response::empty_400()
    }
}

pub fn route_emacs_traffic(request: &Request) -> Result<EmacsRequest, EmacsRequestError> {
    match request.get_param("task") {
        Some(task) if task == "opened" => match request.get_param("id") {
            Some(id) => Ok(EmacsRequest::BufferOpened(id)),
            None => Err(EmacsRequestError::NoIDProvided),
        },
        Some(task) if task == "modified" => match request.get_param("file") {
            Some(file) => Ok(EmacsRequest::BufferModified(file)),
            None => Err(EmacsRequestError::NoFileProvided),
        },
        Some(task) => Err(EmacsRequestError::UnsupportedTask(task)),
        None => Err(EmacsRequestError::NoTaskProvided),
    }
}
