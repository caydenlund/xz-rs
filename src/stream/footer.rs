const MAGIC_BYTES: [u8; 2] = [0x59, 0x5A];

#[derive(Debug, Clone)]
pub struct StreamFooter {
    pub backward_size: u32,
    pub stream_flags: StreamFlags,
}
