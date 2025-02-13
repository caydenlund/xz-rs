use std::io::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncodeError {
    #[error("I/O error: {0}")]
    IoError(#[from] Error),
}

pub type EncodeResult<T> = Result<T, EncodeError>;
