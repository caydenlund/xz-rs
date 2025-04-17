use logomotion::log;
use std::io::{BufRead, Read};

pub struct LoggingReader<R: BufRead> {
    inner: R,
}

impl<R: BufRead> LoggingReader<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R: BufRead> Read for LoggingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let num_bytes = self.inner.read(buf)?;
        (0..num_bytes).for_each(|i| log!("\x1b[31m[0x{:02X}]", buf[i]));
        Ok(num_bytes)
    }
}

impl<R: BufRead> BufRead for LoggingReader<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}
