mod error;
pub use error::*;

use std::io::Write;

pub trait Encode {
    fn encoding(&self) -> Result<Vec<u8>, EncodeError>;

    fn insert_encoding<T: Write>(&self, dst: &mut T) -> Result<(), EncodeError> {
        dst.write_all(&self.encoding()?)
            .map_err(EncodeError::WriteError)
    }
}
