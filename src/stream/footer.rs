use super::{StreamDecodeError, StreamFlags};
use crate::checksum::{Checksum, Crc32};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{Decode, Encode};
use std::io::BufRead;

const MAGIC_BYTES_LEN: usize = 2;
const MAGIC_BYTES: [u8; MAGIC_BYTES_LEN] = [0x59, 0x5A];

#[derive(Debug, Clone)]
pub struct StreamFooter {
    pub backward_size: u32,
    pub flags: StreamFlags,
}

impl Encode for StreamFooter {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let mut crc32 = Crc32::new();

        let backward_size = self.backward_size.to_le_bytes();
        crc32.process_bytes(&backward_size);

        let flags = self.flags.encode()?;
        crc32.process_bytes(&flags);

        let crc32 = crc32.result().to_le_bytes();

        Ok(crc32
            .into_iter()
            .chain(backward_size)
            .chain(flags)
            .chain(MAGIC_BYTES)
            .collect())
    }
}

impl Decode for StreamFooter {
    fn decode<R: BufRead>(src: &mut R) -> DecodeResult<Self> {
        let err = Err(DecodeError::StreamDecodeError(
            StreamDecodeError::InvalidFooter,
        ));

        let mut bytes = [0u8; 12];
        src.read_exact(&mut bytes)?;
        if bytes[10..] != MAGIC_BYTES {
            return err;
        }

        let backward_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap());

        let flags = StreamFlags::try_from(&[bytes[8], bytes[9]])?;

        let mut crc32 = Crc32::new();
        crc32.process_bytes(&bytes[4..10]);
        if bytes[..4] != crc32.result().to_le_bytes() {
            return err;
        }

        Ok(Self {
            backward_size,
            flags,
        })
    }
}
