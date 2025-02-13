use std::io::BufRead;

use super::BlockDecodeError;
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{Decode, Encode, VarLengthInt};

#[derive(Debug, Clone)]
pub struct Filter {
    pub id: u64,
    pub properties: Vec<u8>,
}

impl Encode for Filter {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&VarLengthInt(self.id).encode()?);
        bytes.extend_from_slice(&VarLengthInt(self.properties.len() as u64).encode()?);
        bytes.extend_from_slice(&self.properties);

        Ok(bytes)
    }
}

impl Decode for Filter {
    fn decode<R: BufRead>(src: &mut R) -> DecodeResult<Self> {
        let err = Err(DecodeError::BlockDecodeError(
            BlockDecodeError::InvalidHeader,
        ));
        let id = VarLengthInt::decode(src)?.0;

        if id > 0x4000_0000_0000_0000 {
            return err;
        }

        let properties_size = VarLengthInt::decode(src)?.0;

        if properties_size > 1024 {
            return err;
        }

        let mut properties = vec![0u8; properties_size as usize];
        src.read_exact(&mut properties)?;

        Ok(Self { id, properties })
    }
}
