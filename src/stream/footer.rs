use crate::checksum::{Checksum, Crc32};
use crate::decode::{Decode, DecodeError};
use crate::encode::{Encode, EncodeError};

use super::StreamFlags;

const MAGIC_BYTES_LEN: usize = 2;
const MAGIC_BYTES: [u8; MAGIC_BYTES_LEN] = [0x59, 0x5A];

#[derive(Debug, Clone)]
pub struct StreamFooter {
    pub backward_size: u32,
    pub stream_flags: StreamFlags,
}

impl Encode for StreamFooter {
    fn encoding(&self) -> Result<Vec<u8>, EncodeError> {
        if matches!(self.stream_flags, StreamFlags::Reserved) {
            return Err(EncodeError::ReservedStreamFlags);
        }

        let mut crc_32 = Crc32::new();

        let backward_size = self.backward_size.to_le_bytes();
        crc_32.process_words(&backward_size);

        let stream_flags = self.stream_flags.encoding()?;
        crc_32.process_words(&stream_flags);

        let crc_32 = crc_32.result().to_le_bytes();

        Ok(crc_32
            .into_iter()
            .chain(backward_size)
            .chain(stream_flags)
            .chain(MAGIC_BYTES)
            .collect())
    }
}

impl Decode for StreamFooter {
    fn decode(src: &[u8]) -> Result<(Self, usize), DecodeError> {
        if src.len() < 12 {
            return Err(DecodeError::InvalidFooter);
        }
        if src[10..] != MAGIC_BYTES {
            return Err(DecodeError::InvalidFooter);
        }

        let backward_size = u32::from_le_bytes(src[4..8].try_into().unwrap());

        let (stream_flags, _) = StreamFlags::decode(&src[8..10])?;

        let mut crc_32 = Crc32::new();
        crc_32.process_words(&src[4..10]);
        if src[..4] != crc_32.result().to_le_bytes() {
            return Err(DecodeError::InvalidFooter);
        }

        Ok((
            Self {
                backward_size,
                stream_flags,
            },
            12,
        ))
    }
}
