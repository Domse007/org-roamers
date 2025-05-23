use thiserror::Error;

#[derive(Error, PartialEq, Debug)]
pub enum ServerError {
    #[error("An rusqlite error occured: {0:?}")]
    Rusqlite(rusqlite::Error),
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
        }
    }
}
