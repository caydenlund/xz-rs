use crate::checksum::{Checksum, Crc32};
use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;

use super::{StreamDecodeError, StreamFlags};

const MAGIC_BYTES_LEN: usize = 2;
const MAGIC_BYTES: [u8; MAGIC_BYTES_LEN] = [0x59, 0x5A];

#[derive(Debug, Clone)]
pub struct StreamFooter {
    pub backward_size: u32,
    pub flags: StreamFlags,
}

impl Encode for StreamFooter {
    fn encoding(&self) -> Vec<u8> {
        let mut crc_32 = Crc32::new();

        let backward_size = self.backward_size.to_le_bytes();
        crc_32.process_words(&backward_size);

        let flags = self.flags.encoding();
        crc_32.process_words(&flags);

        let crc_32 = crc_32.result().to_le_bytes();

        crc_32
            .into_iter()
            .chain(backward_size)
            .chain(flags)
            .chain(MAGIC_BYTES)
            .collect()
    }
}

impl Decode for StreamFooter {
    fn decode<R: std::io::Read>(src: &mut R) -> Result<Self, DecodeError> {
        let err = |e| Err(DecodeError::from(e));
        use StreamDecodeError::*;

        let mut bytes = [0u8; 12];
        src.read_exact(&mut bytes)?;
        if bytes[10..] != MAGIC_BYTES {
            return err(InvalidFooter);
        }

        let backward_size = u32::from_le_bytes(bytes[4..8].try_into().unwrap());

        let flags = StreamFlags::try_from(&[bytes[8], bytes[9]])?;

        let mut crc_32 = Crc32::new();
        crc_32.process_words(&bytes[4..10]);
        if bytes[..4] != crc_32.result().to_le_bytes() {
            return err(InvalidFooter);
        }

        Ok(Self {
            backward_size,
            flags,
        })
    }
}
