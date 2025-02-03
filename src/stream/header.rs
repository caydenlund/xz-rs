use super::StreamFlags;

const MAGIC_BYTES: [u8; 6] = [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];

#[derive(Debug, Clone)]
pub struct StreamHeader {
    pub flags: StreamFlags,

    /// CRC32 of the `flags` field
    pub crc_32: u32,
}
