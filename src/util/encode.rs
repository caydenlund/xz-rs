use crate::error::EncodeResult;
use std::io::Write;

pub trait Encode {
    fn encode(&self) -> EncodeResult<Vec<u8>>;

    fn encode_into<W: Write>(&self, dst: &mut W) -> EncodeResult<()> {
        Ok(dst.write_all(&self.encode()?)?)
    }
}
