use crate::{
    decode::{Decode, DecodeError},
    encode::Encode,
};

use super::{BlockDecodeError, VarLengthInt};

#[derive(Debug, Clone)]
pub struct Filter {
    pub id: u64,
    pub properties: Vec<u8>,
}

impl Encode for Filter {
    fn encoding(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&VarLengthInt(self.id).encoding());
        bytes.extend_from_slice(&VarLengthInt(self.properties.len() as u64).encoding());
        bytes.extend_from_slice(&self.properties);

        bytes
    }
}

impl Decode for Filter {
    fn decode<R: std::io::Read>(src: &mut R) -> Result<Self, DecodeError> {
        let err = Err(DecodeError::BlockError(BlockDecodeError::InvalidHeader));
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
