use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;

use super::{StreamDecodeError, StreamFlags};

const MAGIC_BYTES_LEN: usize = 6;
const MAGIC_BYTES: [u8; MAGIC_BYTES_LEN] = [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];

#[derive(Debug, Clone)]
pub struct StreamHeader {
    pub flags: StreamFlags,
}

impl Encode for StreamHeader {
    fn encoding(&self) -> Vec<u8> {
        MAGIC_BYTES
            .into_iter()
            .chain(self.flags.encoding())
            .chain(self.flags.crc_32().to_le_bytes())
            .collect()
    }
}

impl Decode for StreamHeader {
    fn decode(src: &[u8]) -> std::result::Result<(StreamHeader, usize), DecodeError> {
        let err = |e| Err(DecodeError::from(e));
        use StreamDecodeError::*;

        if src.len() < 12 {
            return err(InvalidHeader);
        }

        if src[..MAGIC_BYTES_LEN] != MAGIC_BYTES {
            return err(InvalidHeader);
        }

        let (flags, offset) = StreamFlags::decode(&src[MAGIC_BYTES_LEN..])?;
        let crc_index = MAGIC_BYTES_LEN + offset;

        let end_index = crc_index + 4;
        if flags.crc_32().to_le_bytes() != src[crc_index..end_index] {
            return err(InvalidHeader);
        }

        Ok((Self { flags }, end_index))
    }
}
