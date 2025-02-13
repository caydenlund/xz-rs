pub trait Checksum {
    type Result;

    fn process_next_byte(&mut self, word: u8);

    fn process_bytes(&mut self, words: &[u8]) {
        words.iter().for_each(|word| self.process_next_byte(*word));
    }

    fn result(&self) -> Self::Result;
}
