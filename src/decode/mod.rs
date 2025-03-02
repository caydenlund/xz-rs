mod error;
pub use error::*;

mod recorded_reader;
pub use recorded_reader::*;

use std::io::Read;

pub trait Decode
where
    Self: Sized,
{
    fn decode<R: Read>(src: &mut R) -> Result<Self, DecodeError>;
}
