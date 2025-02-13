use crate::error::DecodeResult;
use std::io::BufRead;

pub trait Decode
where
    Self: Sized,
{
    fn decode<R: BufRead>(src: &mut R) -> DecodeResult<Self>;
}
