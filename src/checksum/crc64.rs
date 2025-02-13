use super::Checksum;

static CRC64_TABLE: [u64; 256] = {
    let mut table = [0u64; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u64;
        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0x42F0E1EBA9EA3693;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
};

#[derive(Debug, Clone)]
pub struct Crc64(u64);

impl Crc64 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Crc64 {
    fn default() -> Self {
        Self(u64::MAX)
    }
}

impl Checksum for Crc64 {
    type Word = u8;

    type Result = u64;

    fn process_next_word(&mut self, word: &Self::Word) {
        self.0 = CRC64_TABLE[((self.0 ^ u64::from(*word)) & 0xFF) as usize] ^ (self.0 >> 8);
    }

    fn result(&self) -> Self::Result {
        self.0 ^ u64::MAX
    }
}
