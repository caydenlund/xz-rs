use std::result::Result;

use crate::checksum::{Checksum, Crc32};
use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;

use super::{StreamDecodeError, StreamFlagsError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamFlags {
    // 0x00: 0 bytes
    None,

    // 0x01: 4 bytes
    Crc32,

    // 0x04: 8 bytes
    Crc64,

    // 0x0A: 32 bytes
    Sha256,
}

impl StreamFlags {
    fn flag_encoding(&self) -> [u8; 2] {
        match self {
            StreamFlags::None => [0, 0x0],
            StreamFlags::Crc32 => [0, 0x1],
            StreamFlags::Crc64 => [0, 0x4],
            StreamFlags::Sha256 => [0, 0xA],
        }
    }

    pub fn crc_32(&self) -> u32 {
        let mut crc_32 = Crc32::new();
        crc_32.process_words(&self.flag_encoding());

        crc_32.result()
    }
}

impl Encode for StreamFlags {
    fn encoding(&self) -> Vec<u8> {
        self.flag_encoding()
            .into_iter()
            .chain(self.crc_32().to_le_bytes())
            .collect()
    }
}

impl TryFrom<&[u8; 2]> for StreamFlags {
    type Error = DecodeError;

    fn try_from(bytes: &[u8; 2]) -> Result<Self, Self::Error> {
        let err = |e| Err(DecodeError::from(StreamDecodeError::from(e)));
        use StreamFlagsError::*;

        if bytes[0] != 0 {
            return err(InvalidStreamFlags);
        }

        match bytes[1] {
            0x0 => Ok(StreamFlags::None),
            0x1 => Ok(StreamFlags::Crc32),
            0x4 => Ok(StreamFlags::Crc64),
            0xA => Ok(StreamFlags::Sha256),
            _ => err(ReservedStreamFlags),
        }
    }
}

impl Decode for StreamFlags {
    fn decode<R: std::io::Read>(src: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = [0u8; 2];
        src.read_exact(&mut bytes)?;
        Self::try_from(&bytes)
    }
}
