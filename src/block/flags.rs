use crate::block::BlockDecodeError;
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{Decode, Encode};

#[derive(Debug, Clone)]
pub struct BlockFlags {
    pub filter_count: u8,
    pub has_compressed_size: bool,
    pub has_uncompressed_size: bool,
}

const FILTER_COUNT_MASK: u8 = 0x03;
const RESERVED_MASK: u8 = 0x3C;
const HAS_COMPRESSED_SIZE_MASK: u8 = 0x40;
const HAS_UNCOMPRESSED_SIZE_MASK: u8 = 0x80;

impl Encode for BlockFlags {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let filter_count = (self.filter_count - 1) & FILTER_COUNT_MASK;

        let has_compressed_size = if self.has_compressed_size {
            HAS_COMPRESSED_SIZE_MASK
        } else {
            0
        };

        let has_uncompressed_size = if self.has_uncompressed_size {
            HAS_UNCOMPRESSED_SIZE_MASK
        } else {
            0
        };

        Ok(vec![
            filter_count | has_compressed_size | has_uncompressed_size,
        ])
    }
}

impl Decode for BlockFlags {
    fn decode<R: std::io::Read>(src: &mut R) -> DecodeResult<Self> {
        let mut bytes = [0u8];
        src.read_exact(&mut bytes)?;
        let byte = bytes[0];

        let filter_count = (byte & FILTER_COUNT_MASK) + 1;
        let reserved = byte & RESERVED_MASK;
        let has_compressed_size = (byte & HAS_COMPRESSED_SIZE_MASK) > 0;
        let has_uncompressed_size = (byte & HAS_UNCOMPRESSED_SIZE_MASK) > 0;

        if reserved > 0 {
            return Err(DecodeError::BlockDecodeError(
                BlockDecodeError::InvalidHeader,
            ));
        }

        Ok(Self {
            filter_count,
            has_compressed_size,
            has_uncompressed_size,
        })
    }
}
