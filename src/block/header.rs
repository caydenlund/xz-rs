use super::{BlockFlags, Filter};
use crate::block::BlockDecodeError;
use crate::checksum::{Checksum, Crc32};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{CheckedReader, Decode, Encode, VarLengthInt};
use std::io::{BufRead, Cursor, Read};

#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub flags: BlockFlags,
    pub compressed_size: Option<u64>,
    pub uncompressed_size: Option<u64>,
    pub filters: Vec<Filter>,
}

impl Encode for BlockHeader {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let mut bytes = self.flags.encode()?;

        for filter in &self.filters {
            bytes.extend_from_slice(&filter.encode()?);
        }

        if let Some(compressed_size) = self.compressed_size {
            bytes.extend_from_slice(&VarLengthInt(compressed_size).encode()?);
        }

        if let Some(uncompressed_size) = self.uncompressed_size {
            bytes.extend_from_slice(&VarLengthInt(uncompressed_size).encode()?);
        }

        let header_size = bytes.len();
        let padding_needed = (4 - ((header_size + 4) % 4)) % 4;
        bytes.extend_from_slice(&vec![0u8; padding_needed]);

        let mut crc32 = Crc32::new();
        crc32.process_bytes(&bytes);
        bytes.extend_from_slice(&crc32.result().to_le_bytes());

        Ok(bytes)
    }
}

impl Decode for BlockHeader {
    fn decode<R: BufRead>(mut src: &mut R) -> DecodeResult<Self> {
        let mut src = CheckedReader::new(&mut src, Crc32::new());

        let err = Err(DecodeError::BlockDecodeError(
            BlockDecodeError::InvalidHeader,
        ));

        let read_bytes =
            |n: usize, src: &mut CheckedReader<&mut R, Crc32>| -> Result<Vec<u8>, DecodeError> {
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

        let padding_size = header_size - src.len() - 4;
        if read_bytes(padding_size, &mut src)?.iter().any(|&b| b != 0) {
            return err;
        }

        let actual_crc32 = src.checksum();
        let mut expected_crc32 = [0u8; 4];
        src.read_exact(&mut expected_crc32)?;
        if actual_crc32.to_le_bytes() != expected_crc32 {
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
