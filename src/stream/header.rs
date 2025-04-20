use super::{StreamDecodeError, StreamFlags};
use crate::checksum::{Checksum, Crc32};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{Decode, Encode};
use std::io::BufRead;

const MAGIC_BYTES_LEN: usize = 6;
const MAGIC_BYTES: [u8; MAGIC_BYTES_LEN] = [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];

#[derive(Debug, Clone)]
pub struct StreamHeader {
    pub flags: StreamFlags,
}

impl Encode for StreamHeader {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        Ok(MAGIC_BYTES
            .into_iter()
            .chain(self.flags.encode()?)
            .collect())
    }
}

impl Decode for StreamHeader {
    fn decode<R: BufRead>(src: &mut R) -> DecodeResult<Self> {
        let err = Err(DecodeError::StreamDecodeError(
            StreamDecodeError::InvalidHeader,
        ));

        let mut bytes = [0u8; 12];
        src.read_exact(&mut bytes)?;

        if bytes[..MAGIC_BYTES_LEN] != MAGIC_BYTES {
            return err;
        }

        let flag_bytes = [bytes[MAGIC_BYTES_LEN], bytes[MAGIC_BYTES_LEN + 1]];
        let flags = StreamFlags::try_from(&flag_bytes)?;

        let mut crc32 = Crc32::new();
        crc32.process_bytes(&flag_bytes);
        if crc32.result().to_le_bytes() != bytes[8..] {
            return err;
        }

        Ok(Self { flags })
    }
}
