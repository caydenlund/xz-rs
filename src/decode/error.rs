use crate::block::BlockDecodeError;
use crate::stream::StreamDecodeError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid stream: {0}")]
    StreamError(#[from] StreamDecodeError),

    #[error("Invalid block: {0}")]
    BlockError(#[from] BlockDecodeError),

    #[error("Error reading data: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("VLI overflow")]
    VliOverflowError,
}
