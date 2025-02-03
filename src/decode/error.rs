use std::error::Error;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum DecodeError {}

impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for DecodeError {}
