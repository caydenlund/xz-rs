use crate::block::BlockDecodeError;
use crate::lzma2::Lzma2DecodeError;
use crate::stream::StreamDecodeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Invalid stream: {0}")]
    StreamDecodeError(#[from] StreamDecodeError),

    #[error("Invalid block: {0}")]
    BlockDecodeError(#[from] BlockDecodeError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("VLI overflow")]
    VliOverflowError,

    #[error("LZMA2 error: {0}")]
    LzmaError(#[from] Lzma2DecodeError),
}

pub type DecodeResult<T> = Result<T, DecodeError>;
