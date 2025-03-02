mod error;
pub use error::*;

use crate::stream::{StreamDecodeError, StreamFooter, StreamHeader};
use std::{fmt::Write, io::Read};

pub trait Decode
where
    Self: Sized,
{
    fn decode<R: Read>(src: &mut R) -> Result<Self, DecodeError>;
}

pub fn decode_xz<R: Read, W: Write>(mut src: R, dst: &mut W) -> Result<(), DecodeError> {
    todo!()
}
