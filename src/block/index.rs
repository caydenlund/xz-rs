use std::io::Read;

use crate::checksum::{Checksum, Crc32};
use crate::decode::{Decode, DecodeError, RecordedReader};
use crate::encode::Encode;

use super::{BlockDecodeError, VarLengthInt};

#[derive(Debug, Clone)]
pub struct IndexRecord {
    pub uncompressed_size: u64,
    pub unpadded_size: u64,
}

#[derive(Debug, Clone)]
pub struct BlockIndex {
    pub records: Vec<IndexRecord>,
}

impl Decode for BlockIndex {
    fn decode<R: std::io::Read>(src: &mut R) -> Result<Self, DecodeError> {
        let err = Err(DecodeError::BlockError(BlockDecodeError::InvalidIndex));
        let mut src = RecordedReader::new(src);

        let mut bytes = [0u8];
        src.read_exact(&mut bytes)?;
        if bytes[0] != 0 {
            return err;
        }

        let num_records = VarLengthInt::decode(&mut src)?.0 as usize;

        let mut records = Vec::with_capacity(num_records);
        for _ in 0..num_records {
            let uncompressed_size = VarLengthInt::decode(&mut src)?.0;

            let unpadded_size = VarLengthInt::decode(&mut src)?.0;

            records.push(IndexRecord {
                uncompressed_size,
                unpadded_size,
            });
        }

        let padding_size = (4 - ((src.len() + 4) % 4)) % 4;
        let mut padding = vec![0u8; padding_size];
        src.read_exact(&mut padding)?;

        if padding.iter().any(|&b| b != 0) {
            return err;
        }

        let actual_crc32 = src.crc32();
        let mut crc32_bytes = [0u8; 4];
        src.read_exact(&mut crc32_bytes)?;
        let expected_crc32 = u32::from_le_bytes(crc32_bytes);

        if actual_crc32 != expected_crc32 {
            return err;
        }

        Ok(BlockIndex { records })
    }
}

impl Encode for BlockIndex {
    fn encoding(&self) -> Vec<u8> {
        let mut bytes = vec![0];

        bytes.extend_from_slice(&VarLengthInt(self.records.len() as u64).encoding());

        for record in &self.records {
            bytes.extend_from_slice(&VarLengthInt(record.uncompressed_size).encoding());
            bytes.extend_from_slice(&VarLengthInt(record.unpadded_size).encoding());
        }

        let padding_needed = (4 - ((bytes.len() + 4) % 4)) % 4;
        bytes.extend_from_slice(&vec![0u8; padding_needed]);

        let mut crc32 = Crc32::new();
        crc32.process_words(&bytes);
        bytes.extend_from_slice(&crc32.result().to_le_bytes());

        bytes
    }
}
