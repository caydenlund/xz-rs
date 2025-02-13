use std::io::Write;

use crate::{
    error::{DecodeError, DecodeResult},
    lzma2::Lzma2DecodeError,
    util::InputRead,
};

use super::dict::Dict;
use super::lzma_decoder::LzmaDecoder;
use super::range_decoder::RangeDecoder;

pub(crate) struct Lzma2Decoder<W: Write> {
    lzma_dec: LzmaDecoder,
    dict: Dict<W>,
    rc: RangeDecoder,
}

impl<W: Write> Lzma2Decoder<W> {
    pub(crate) fn new<R: InputRead>(input: &mut R, output: W) -> DecodeResult<Self> {
        Ok(Self {
            lzma_dec: LzmaDecoder::new(),
            dict: Dict::new(output),
            rc: RangeDecoder::new(input)?,
        })
    }

    pub(crate) fn decode<R: InputRead>(&mut self, input: &mut R) -> DecodeResult<()> {
        loop {
            let control_byte = input.read_u8()?;
            match control_byte {
                0x00 => break,
                0x01 => self.decode_uncompressed(input, true)?,
                0x02 => self.decode_uncompressed(input, false)?,
                0x80.. => self.decode_compressed(input, control_byte)?,
                _ => return Err(DecodeError::LzmaError(Lzma2DecodeError::InvalidControlByte)),
            }
        }

        Ok(())
    }

    fn decode_uncompressed<R: InputRead>(
        &mut self,
        input: &mut R,
        reset_dict: bool,
    ) -> DecodeResult<()> {
        todo!(); //
    }

    fn decode_compressed<R: InputRead>(
        &mut self,
        input: &mut R,
        control_byte: u8,
    ) -> DecodeResult<()> {
        todo!(); //
    }
}
