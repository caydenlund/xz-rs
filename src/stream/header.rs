use crate::decode::{Decode, DecodeError};
use crate::encode::{Encode, EncodeError};

use super::StreamFlags;

const MAGIC_BYTES_LEN: usize = 6;
const MAGIC_BYTES: [u8; MAGIC_BYTES_LEN] = [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];

#[derive(Debug, Clone)]
pub struct StreamHeader {
    pub flags: StreamFlags,
}

impl Encode for StreamHeader {
    fn encoding(&self) -> Result<Vec<u8>, EncodeError> {
        Ok(MAGIC_BYTES
            .into_iter()
            .chain(self.flags.encoding()?)
            .chain(
                self.flags
                    .crc_32()
                    .ok_or(EncodeError::ReservedStreamFlags)?
                    .to_le_bytes(),
            )
            .collect())
    }
}

impl Decode for StreamHeader {
    fn decode(src: &[u8]) -> std::result::Result<(StreamHeader, usize), DecodeError> {
        if src.len() < 12 {
            return Err(DecodeError::InvalidHeader);
        }

        if src[..MAGIC_BYTES_LEN] != MAGIC_BYTES {
            return Err(DecodeError::InvalidHeader);
        }

        let (flags, offset) = StreamFlags::decode(&src[MAGIC_BYTES_LEN..])?;
        let crc_index = MAGIC_BYTES_LEN + offset;
        if matches!(flags, StreamFlags::Reserved) {
            return Err(DecodeError::ReservedStreamFlags);
        }

        let end_index = crc_index + 4;
        if flags
            .crc_32()
            .ok_or(DecodeError::ReservedStreamFlags)?
            .to_le_bytes()
            != src[crc_index..end_index]
        {
            return Err(DecodeError::InvalidHeader);
        }

        Ok((Self { flags }, end_index))
    }
}
