use thiserror::Error;

#[derive(Error, Debug)]
pub enum Lzma2DecodeError {
    #[error("Invalid lc/lp/pb properties")]
    InvalidProperties,

    #[error("Invalid control byte")]
    InvalidControlByte,
}
