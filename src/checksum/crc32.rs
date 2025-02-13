use super::Checksum;

static CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if crc & 1 == 1 {
                crc = (crc >> 1) ^ 0xEDB88320;
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
pub struct Crc32(u32);

impl Crc32 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self(0xFFFFFFFF)
    }
}

impl Checksum for Crc32 {
    type Result = u32;

    fn process_next_byte(&mut self, byte: u8) {
        self.0 = CRC32_TABLE[((self.0 ^ u32::from(byte)) & 0xFF) as usize] ^ (self.0 >> 8);
    }

    fn result(&self) -> Self::Result {
        self.0 ^ 0xFFFFFFFF
    }
}
