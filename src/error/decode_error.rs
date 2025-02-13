use crate::block::BlockDecodeError;
use crate::stream::StreamDecodeError;
use std::io::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid stream: {0}")]
    StreamDecodeError(#[from] StreamDecodeError),

    #[error("Invalid block: {0}")]
    BlockDecodeError(#[from] BlockDecodeError),

    #[error("I/O error: {0}")]
    IoError(#[from] Error),

    #[error("VLI overflow")]
    VliOverflowError,
}

pub type DecodeResult<T> = Result<T, DecodeError>;
