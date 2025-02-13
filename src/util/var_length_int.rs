use super::{Decode, Encode};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use std::io::BufRead;

pub struct VarLengthInt(pub u64);

impl Encode for VarLengthInt {
    fn encode(&self) -> EncodeResult<Vec<u8>> {
        let mut bytes = Vec::new();
        let mut value = self.0;

        while value >= 0x80 {
            bytes.push((value as u8) | 0x80);
            value >>= 7;
        }

        bytes.push(value as u8);
        Ok(bytes)
    }
}

impl Decode for VarLengthInt {
    fn decode<R: BufRead>(src: &mut R) -> DecodeResult<Self> {
        let err = Err(DecodeError::VliOverflowError);

        let mut bytes = [0u8];

        let mut result = bytes[0] as u64;
        let mut shift = 0;

        while src.read_exact(&mut bytes).is_ok() {
            result |= ((bytes[0] & 0x7F) as u64) << shift;

            if (bytes[0] & 0x80) == 0 {
                return if bytes[0] == 0 && shift != 0 {
                    err
                } else {
                    Ok(VarLengthInt(result))
                };
            }

            shift += 7;
            if shift == 63 {
                return err;
            }
        }

        Ok(VarLengthInt(result))
    }
}
