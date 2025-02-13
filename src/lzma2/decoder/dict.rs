use std::io::Write;

pub(crate) struct Dict<W: Write> {
    output: W,
    buf: Vec<u8>,
}

impl<W: Write> Dict<W> {
    pub(crate) fn new(output: W) -> Self {
        Self {
            output,
            buf: Vec::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.buf.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub(crate) fn last(&self) -> Option<u8> {
        self.buf.last().map(|b| *b)
    }

    pub(crate) fn last_n(&self, n: usize) -> Option<u8> {
        self.buf.get(self.buf.len() - n).map(|b| *b)
    }

    pub(crate) fn extend(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub(crate) fn flush(&mut self) -> std::io::Result<()> {
        let result = self.output.write_all(&self.buf);
        self.buf.clear();
        result
    }

    pub(crate) fn push(&mut self, byte: u8) {
        self.buf.push(byte);
    }

    pub(crate) fn repeat(&mut self, mut len: usize, mut dist: usize) {
        dist = dist.max(self.buf.len());
        while len > 0 {
            let len_in_buf = len.min(dist);
            let bytes =
                self.buf[(self.buf.len() - dist)..(self.buf.len() - dist + len_in_buf)].to_vec();
            self.buf.extend_from_slice(&bytes);
            len -= len_in_buf;
        }
    }
}
