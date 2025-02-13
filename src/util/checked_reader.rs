use crate::checksum::{Checksum, Crc32};
use std::io::{BufRead, Read};

#[derive(Debug)]
pub struct CheckedReader<'r, R: BufRead> {
    pub inner: &'r mut R,
    buffer: Vec<u8>,
    start: usize,
    end: usize,
}

impl<'r, R: BufRead> CheckedReader<'r, R> {
    pub fn new(inner: &'r mut R) -> Self {
        Self {
            inner,
            buffer: Vec::new(),
            start: 0,
            end: 0,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer[self.start..self.end]
    }

    pub fn whole_buffer(&self) -> &[u8] {
        &self.buffer[..self.end]
    }

    pub fn crc32(&self) -> u32 {
        let mut crc32 = Crc32::new();
        crc32.process_words(self.whole_buffer());
        crc32.result()
    }
}

impl<R: BufRead> Read for CheckedReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.start >= self.end {
            self.fill_buf()?;
            if self.start >= self.end {
                return Ok(0);
            }
        }

        let len = buf.len().min(self.end - self.start);
        buf[..len].copy_from_slice(&self.buffer[self.start..(self.start + len)]);
        self.start += len;
        Ok(len)
    }
}

impl<R: BufRead> BufRead for CheckedReader<'_, R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.start >= self.end {
            self.start = 0;
            self.end = 0;
        }

        if self.buffer.capacity() == 0 {
            self.buffer.reserve(8192);
        }

        let read = self.inner.read(&mut self.buffer[self.end..])?;
        self.end += read;

        Ok(&self.buffer[self.start..self.end])
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}
