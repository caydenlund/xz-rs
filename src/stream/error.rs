use thiserror::Error;

#[derive(Error, Debug)]
pub enum StreamFlagsError {
    #[error("Reserved stream flags")]
    ReservedStreamFlags,

    #[error("Invalid stream flags")]
    InvalidStreamFlags,
}

#[derive(Error, Debug)]
pub enum StreamDecodeError {
    #[error("{0}")]
    StreamFlagsError(#[from] StreamFlagsError),

    #[error("Stream header/footer mismatch")]
    HeaderFooterMismatch,

    #[error("Invalid stream header")]
    InvalidHeader,

    #[error("Invalid stream footer")]
    InvalidFooter,

    #[error("Invalid stream index")]
    InvalidIndex,
}
