use thiserror::Error;

use crate::sqlite::olp::OlpError;

#[derive(Error, PartialEq, Debug)]
pub enum ServerError {
    #[error("An rusqlite error occured: {0:?}")]
    Rusqlite(rusqlite::Error),
    #[error("Error occured while parsing olp: {0:?}")]
    OlpError(OlpError),
}

impl From<rusqlite::Error> for ServerError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Rusqlite(value)
    }
}

impl ServerError {
    pub fn recoverable(&self) -> bool {
        match self {
            Self::Rusqlite(_) => true,
            Self::OlpError(_) => true,
        }
    }
}
