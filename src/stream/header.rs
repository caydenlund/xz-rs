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
    fn decode<R: std::io::Read>(src: &mut R) -> Result<Self, DecodeError> {
        let err = |e| Err(DecodeError::from(e));
        use StreamDecodeError::*;

        let mut bytes = [0u8; 12];
        src.read_exact(&mut bytes)?;

        if bytes[..MAGIC_BYTES_LEN] != MAGIC_BYTES {
            return err(InvalidHeader);
        }

        let flags = StreamFlags::try_from(&[bytes[MAGIC_BYTES_LEN], bytes[MAGIC_BYTES_LEN + 1]])?;

        if flags.crc_32().to_le_bytes() != bytes[8..] {
            return err(InvalidHeader);
        }

        Ok(Self { flags })
    }
}
