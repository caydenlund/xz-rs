use std::io::Write;

pub trait Encode {
    fn encoding(&self) -> Vec<u8>;

    fn insert_encoding<T: Write>(&self, dst: &mut T) -> Result<(), std::io::Error> {
        dst.write_all(&self.encoding())
    }
}
