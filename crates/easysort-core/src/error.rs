use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a directory: {0}")]
    NotDir(String),
    #[error("move failed: {0}")]
    Move(String),
}
