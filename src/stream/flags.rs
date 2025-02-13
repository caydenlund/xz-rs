use super::{StreamDecodeError, StreamFlagsError};
use crate::checksum::{Checksum, Crc32};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{Decode, Encode};
use std::io::BufRead;
use std::result::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamFlags {
    None,   // 0x0
    Crc32,  // 0x1
    Crc64,  // 0x4
    Sha256, // 0xA
}

impl Encode for StreamFlags {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let flag_enc = match self {
            StreamFlags::None => [0, 0x0],
            StreamFlags::Crc32 => [0, 0x1],
            StreamFlags::Crc64 => [0, 0x4],
            StreamFlags::Sha256 => [0, 0xA],
        };
        let mut crc32 = Crc32::new();
        crc32.process_words(&flag_enc);
        Ok(flag_enc
            .into_iter()
            .chain(crc32.result().to_le_bytes())
            .collect())
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
    fn decode<R: BufRead>(src: &mut R) -> DecodeResult<Self> {
        use StreamFlagsError::*;
        let err = |e| Err(DecodeError::from(StreamDecodeError::from(e)));

        let mut bytes = [0u8; 2];
        src.read_exact(&mut bytes)?;

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
