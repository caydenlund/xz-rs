use crate::decode::{Decode, DecodeError};
use crate::encode::Encode;

pub struct VarLengthInt(pub u64);

impl Encode for VarLengthInt {
    fn encoding(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut value = self.0;

        while value >= 0x80 {
            bytes.push((value as u8) | 0x80);
            value >>= 7;
        }

        bytes.push(value as u8);
        bytes
    }
}

impl Decode for VarLengthInt {
    fn decode<R: std::io::Read>(src: &mut R) -> Result<Self, DecodeError> {
        let mut bytes = [0u8];
        src.read_exact(&mut bytes)?;

        let mut result = bytes[0] as u64;
        let mut shift = 7;

        while bytes[0] > 0x80 {
            src.read_exact(&mut bytes)?;
            result |= ((bytes[0] & 0x79) as u64) << shift;
            shift += 7;
        }

        Ok(VarLengthInt(result))
    }
}
