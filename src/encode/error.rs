use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum EncodeError {
    WriteError(std::io::Error),
}

impl Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for EncodeError {}
