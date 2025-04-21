use crate::checksum::Checksum;
use std::io::Write;

#[derive(Debug)]
pub struct CheckedWriter<'r, W: Write, C: Checksum> {
    pub inner: &'r mut W,
    checksum: C,
}

impl<'r, W: Write, C: Checksum> CheckedWriter<'r, W, C> {
    pub fn new(inner: &'r mut W, checksum: C) -> Self {
        Self { inner, checksum }
    }

    pub fn checksum(&self) -> C::Result {
        self.checksum.result()
    }
}

impl<W: Write, C: Checksum> Write for CheckedWriter<'_, W, C> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.checksum.process_bytes(buf);
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
