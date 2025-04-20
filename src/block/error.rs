use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockDecodeError {
    #[error("Invalid block header")]
    InvalidHeader,

    #[error("Invalid block footer")]
    InvalidIndex,

    #[error("Reserved block flags")]
    ReservedBlockFlags,

    #[error("Invalid variable-length integer")]
    InvalidVarInt,

    #[error("Invalid compressed data")]
    InvalidData,

    #[error("Checksum didn't match")]
    ChecksumMismatch,
}
