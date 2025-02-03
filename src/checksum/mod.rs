mod crc32;
pub use crc32::*;

pub trait Checksum {
    type Word;
    type Result;

    fn process_next_word(&mut self, word: &Self::Word);

    fn process_words(&mut self, words: &[Self::Word]) {
        words.iter().for_each(|word| self.process_next_word(word));
    }

    fn result(&self) -> Self::Result;
}
