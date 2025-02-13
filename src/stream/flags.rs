use std::result::Result;

use crate::checksum::{Checksum, Crc32};
use crate::decode::{Decode, DecodeError};
use crate::encode::{Encode, EncodeError};

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
    fn flag_encoding(&self) -> Result<[u8; 2], EncodeError> {
        match self {
            StreamFlags::None => Ok([0, 0x0]),
            StreamFlags::Crc32 => Ok([0, 0x1]),
            StreamFlags::Crc64 => Ok([0, 0x4]),
            StreamFlags::Sha256 => Ok([0, 0xA]),
            StreamFlags::Reserved => Err(EncodeError::ReservedStreamFlags),
        }
    }

    pub fn crc_32(&self) -> Option<u32> {
        let Ok(flag_bytes) = self.flag_encoding() else {
            return None;
        };

        let mut crc_32 = Crc32::new();
        crc_32.process_words(&flag_bytes);

        Some(crc_32.result())
    }
}

impl Encode for StreamFlags {
    fn encoding(&self) -> Result<Vec<u8>, EncodeError> {
        Ok(self
            .flag_encoding()?
            .into_iter()
            .chain(
                self.crc_32()
                    .ok_or(EncodeError::ReservedStreamFlags)?
                    .to_le_bytes(),
            )
            .collect())
    }
}

impl Decode for StreamFlags {
    fn decode(src: &[u8]) -> Result<(StreamFlags, usize), DecodeError> {
        if src.len() < 2 || src[0] != 0 {
            return Err(DecodeError::InvalidHeader);
        }
        match src[1] {
            0x0 => Ok((StreamFlags::None, 2)),
            0x1 => Ok((StreamFlags::Crc32, 2)),
            0x4 => Ok((StreamFlags::Crc64, 2)),
            0xA => Ok((StreamFlags::Sha256, 2)),
            _ => Err(DecodeError::InvalidHeader),
        }
    }
}
