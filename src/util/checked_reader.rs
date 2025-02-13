use crate::checksum::Checksum;
use std::io::{BufRead, Read};

#[derive(Debug)]
pub struct CheckedReader<'r, R: BufRead, C: Checksum> {
    pub inner: &'r mut R,
    checksum: C,
    read: usize,
}

impl<'r, R: BufRead, C: Checksum> CheckedReader<'r, R, C> {
    pub fn new(inner: &'r mut R, checksum: C) -> Self {
        Self {
            inner,
            checksum,
            read: 0,
        }
    }

    pub fn checksum(&self) -> C::Result {
        self.checksum.result()
    }

    pub fn len(&self) -> usize {
        self.read
    }
}

impl<R: BufRead, C: Checksum> Read for CheckedReader<'_, R, C> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        self.checksum.process_bytes(&buf[..bytes_read]);
        self.read += bytes_read;
        Ok(bytes_read)
    }
}

impl<R: BufRead, C: Checksum> BufRead for CheckedReader<'_, R, C> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        let data = self.inner.fill_buf().unwrap_or(&[]);
        self.checksum.process_bytes(&data[..amt.min(data.len())]);
        self.inner.consume(amt);
    }
}
