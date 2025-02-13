use super::BlockDecodeError;
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::util::{Decode, Encode, VarLengthInt};
use std::io::BufRead;

#[derive(Debug, Clone)]
pub enum Filter {
    Lzma2 { dict_size: u32 },
}

impl Filter {
    pub fn try_new(id: u64, properties: &[u8]) -> DecodeResult<Self> {
        let err = Err(DecodeError::BlockDecodeError(
            BlockDecodeError::InvalidHeader,
        ));

        match (id, properties.len()) {
            (0x21, 1) => {
                let dict_size = match properties[0] {
                    41.. => Err(DecodeError::BlockDecodeError(
                        BlockDecodeError::InvalidHeader,
                    )),
                    40 => Ok(u32::MAX),
                    bits => Ok((2 | (bits as u32 & 1)) << (bits as u32 / 2 + 11)),
                }?;
                Ok(Self::Lzma2 { dict_size })
            }
            _ => err,
        }
    }

    pub fn id(&self) -> u64 {
        match self {
            Filter::Lzma2 { .. } => 0x21,
        }
    }

    pub fn properties(&self) -> Vec<u8> {
        match self {
            Filter::Lzma2 { dict_size } => {
                for bits in 0..=40 {
                    if Self::lzma2_dict_size(bits) == *dict_size {
                        return vec![bits];
                    }
                }
                vec![0]
            }
        }
    }

    fn lzma2_dict_size(bits: u8) -> u32 {
        match bits {
            41.. => 0,
            40 => u32::MAX,
            bits => (2 | (bits as u32 & 1)) << (bits as u32 / 2 + 11),
        }
    }
}

impl Encode for Filter {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let mut bytes = Vec::new();

        let id = self.id();
        let properties = self.properties();

        bytes.extend_from_slice(&VarLengthInt(id).encode()?);
        bytes.extend_from_slice(&VarLengthInt(properties.len() as u64).encode()?);
        bytes.extend_from_slice(&properties);

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

        Self::try_new(id, &properties)
    }
}
