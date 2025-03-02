use crate::checksum::{Checksum, Crc32};
use std::io::Read;

#[derive(Debug)]
pub struct RecordedReader<'r, R: Read> {
    pub reader: &'r mut R,
    pub recording: Vec<u8>,
}

impl<'r, R: Read> RecordedReader<'r, R> {
    pub fn new(reader: &'r mut R) -> Self {
        Self {
            reader,
            recording: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.recording.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recording.is_empty()
    }

    pub fn crc32(&self) -> u32 {
        let mut crc32 = Crc32::new();
        crc32.process_words(&self.recording);
        crc32.result()
    }
}

impl<R: Read> Read for RecordedReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let num_bytes = self.reader.read(buf)?;
        self.recording.extend_from_slice(&buf[..num_bytes]);
        Ok(num_bytes)
    }
}
