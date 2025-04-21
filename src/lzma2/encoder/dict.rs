use std::io::{self, Write};

pub(crate) struct Dict<W: Write> {
    output: W,
    buf: Vec<u8>,
    start: usize,
    end: usize,
}

impl<W: Write> Dict<W> {
    pub(crate) fn new(output: W, size: usize) -> Self {
        Self {
            output,
            buf: vec![0; size],
            start: 0,
            end: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.buf.len()
    }

    pub(crate) fn flush(&mut self, num_bytes: usize) -> io::Result<()> {
        if self.end >= self.start {
            let num_bytes = (self.end - self.start).min(num_bytes);
            self.output
                .write_all(&self.buf[self.start..(self.start + num_bytes)])?;
            self.start += num_bytes;
        } else {
            let num_bytes = (self.end - self.start + self.len()).min(num_bytes);
            let partial_bytes = (self.len() - self.start).min(num_bytes);
            self.output
                .write_all(&self.buf[self.start..(self.start + partial_bytes)])?;
            self.output
                .write_all(&self.buf[0..(num_bytes - partial_bytes)])?;
            self.start = (self.start + num_bytes) % self.len();
        }

        Ok(())
    }

    pub(crate) fn push(&mut self, byte: u8) -> io::Result<()> {
        if self.end == self.start {
            self.flush(1)?;
        }
        self.buf[self.end] = byte;
        self.end = (self.end + 1) % self.len();
        Ok(())
    }

    pub(crate) fn extend(&mut self, bytes: &[u8]) -> io::Result<()> {
        let remaining_space = (self.len() + self.end - self.start) % self.len();
        if bytes.len() > remaining_space {
            self.flush(remaining_space - bytes.len())?;
        }

        let partial_bytes = bytes.len().min(self.len() - self.end);
        self.buf[self.end..(self.end + partial_bytes)].copy_from_slice(&bytes[..partial_bytes]);
        self.buf[0..(bytes.len() - partial_bytes)].copy_from_slice(&bytes[partial_bytes..]);

        self.end = (self.end + bytes.len()) % self.len();
        Ok(())
    }
}
