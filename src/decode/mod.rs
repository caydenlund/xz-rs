mod error;
pub use error::*;

pub trait Decode
where
    Self: Sized,
{
    fn decode(src: &[u8]) -> Result<(Self, usize), DecodeError>;
}
