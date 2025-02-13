use super::StreamDecodeError;
use crate::checksum::{Checksum, Crc32};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{CheckedReader, Decode, Encode, VarLengthInt};
use std::io::{BufRead, Read};

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
    fn decode<R: BufRead>(src: &mut R) -> DecodeResult<Self> {
        let err = Err(DecodeError::StreamDecodeError(
            StreamDecodeError::InvalidIndex,
        ));
        let mut src = CheckedReader::new(src, Crc32::new());

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

        let padding_size = (4 - (src.len() % 4)) % 4;
        let mut padding = vec![0u8; padding_size];
        src.read_exact(&mut padding)?;

        if padding.iter().any(|&b| b != 0) {
            return err;
        }

        let actual_crc32 = src.checksum();
        let mut expected_crc32 = [0u8; 4];
        src.read_exact(&mut expected_crc32)?;

        if actual_crc32.to_le_bytes() != expected_crc32 {
            return err;
        }

        Ok(BlockIndex { records })
    }
}

impl Encode for BlockIndex {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let mut bytes = vec![0];

        bytes.extend_from_slice(&VarLengthInt(self.records.len() as u64).encode()?);

        for record in &self.records {
            bytes.extend_from_slice(&VarLengthInt(record.uncompressed_size).encode()?);
            bytes.extend_from_slice(&VarLengthInt(record.unpadded_size).encode()?);
        }

        let padding_needed = (4 - ((bytes.len() + 4) % 4)) % 4;
        bytes.extend_from_slice(&vec![0u8; padding_needed]);

        let mut crc32 = Crc32::new();
        crc32.process_bytes(&bytes);
        bytes.extend_from_slice(&crc32.result().to_le_bytes());

        Ok(bytes)
    }
}
