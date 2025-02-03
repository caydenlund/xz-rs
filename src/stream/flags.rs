#[derive(Debug, Clone)]
pub enum StreamFlags {
    // 0x00: 0 bytes
    None,

    // 0x01: 4 bytes
    Crc32,

    // 0x04: 8 bytes
    Crc64,

    // 0x0A: 32 bytes
    Sha256,

    // Everything else from 0x00-0x0F
    Reserved,
}

impl StreamFlags {
    pub fn crc_32(&self) -> u32 {
        todo!()
    }
}
