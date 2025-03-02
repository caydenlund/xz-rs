use std::io::{Cursor, Read};

use crate::block::BlockDecodeError;
use crate::checksum::{Checksum, Crc32};
use crate::decode::{Decode, DecodeError, RecordedReader};
use crate::encode::Encode;

use super::{BlockFlags, Filter, VarLengthInt};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub flags: BlockFlags,
    pub compressed_size: Option<u64>,
    pub uncompressed_size: Option<u64>,
    pub filters: Vec<Filter>,
}

impl Encode for BlockHeader {
    fn encoding(&self) -> Vec<u8> {
        let mut bytes = self.flags.encoding();

        for filter in &self.filters {
            bytes.extend_from_slice(&filter.encoding());
        }

        if let Some(compressed_size) = self.compressed_size {
            bytes.extend_from_slice(&VarLengthInt(compressed_size).encoding());
        }

        if let Some(uncompressed_size) = self.uncompressed_size {
            bytes.extend_from_slice(&VarLengthInt(uncompressed_size).encoding());
        }

        let header_size = bytes.len();
        let padding_needed = (4 - ((header_size + 4) % 4)) % 4;
        bytes.extend_from_slice(&vec![0u8; padding_needed]);

        let mut crc32 = Crc32::new();
        crc32.process_words(&bytes);
        bytes.extend_from_slice(&crc32.result().to_le_bytes());

        bytes
    }
}

impl Decode for BlockHeader {
    fn decode<R: Read>(mut src: &mut R) -> Result<Self, DecodeError> {
        let mut src = RecordedReader::new(&mut src);

        let err = Err(DecodeError::BlockError(BlockDecodeError::InvalidHeader));

        let read_bytes =
            |n: usize, src: &mut RecordedReader<&mut R>| -> Result<Vec<u8>, DecodeError> {
                let mut buf = vec![0u8; n];
                src.read_exact(&mut buf)?;
                Ok(buf)
            };

        let header_size = read_bytes(1, &mut src)?[0];
        if header_size == 0 {
            return err;
        }
        let header_size = ((header_size as usize) + 1) * 4;

        let flag_byte = read_bytes(1, &mut src)?;
        let flags = BlockFlags::decode(&mut Cursor::new(&flag_byte))?;

        let compressed_size = if flags.has_compressed_size {
            let size = VarLengthInt::decode(&mut src)?.0;
            if size == 0 {
                return err;
            }
            Some(size)
        } else {
            None
        };

        let uncompressed_size = if flags.has_compressed_size {
            let size = VarLengthInt::decode(&mut src)?.0;
            if size == 0 {
                return err;
            }
            Some(size)
        } else {
            None
        };

        let mut filters = Vec::with_capacity(flags.filter_count as usize);
        for _ in 0..flags.filter_count {
            let filter = Filter::decode(&mut src)?;
            filters.push(filter);
        }

        let padding_size = header_size - src.recording.len() - 4;
        if read_bytes(padding_size, &mut src)?.iter().any(|&b| b != 0) {
            return err;
        }

        let actual_crc32 = src.crc32();
        let mut crc32_bytes = [0u8; 4];
        src.read_exact(&mut crc32_bytes)?;
        let expected_crc32 = u32::from_le_bytes(crc32_bytes);
        if actual_crc32 != expected_crc32 {
            return err;
        }

        Ok(BlockHeader {
            flags,
            compressed_size,
            uncompressed_size,
            filters,
        })
    }
}
