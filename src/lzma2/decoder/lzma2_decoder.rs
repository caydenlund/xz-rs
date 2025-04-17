use std::io::{Cursor, Write};

use logomotion::{func, log};

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
    pub(crate) fn new(output: W) -> Self {
        Self {
            lzma_dec: LzmaDecoder::new(),
            dict: Dict::new(output),
            rc: RangeDecoder::default(),
        }
    }

    pub(crate) fn decode<R: InputRead>(&mut self, input: &mut R) -> DecodeResult<()> {
        let _ctx = func!("Lzma2Decoder::decode(input)");

        loop {
            let control_byte = input.read_u8()?;

            log!("control byte: 0x{control_byte:02X}");

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
        let _ctx = func!("Lzma2Decoder::decode_uncompressed(input, reset_dict: {reset_dict})");

        todo!(); //
    }

    fn decode_compressed<R: InputRead>(
        &mut self,
        input: &mut R,
        control_byte: u8,
    ) -> DecodeResult<()> {
        let _ctx =
            func!("Lzma2Decoder::decode_compressed(input, control_byte: 0x{control_byte:02X})");

        // Bits 5-6 of the control byte tell us what needs to be reset.
        let (reset_dict, reset_props, reset_state) = match (control_byte >> 5) & 0x3 {
            0 => (false, false, false),
            1 => (false, false, true),
            2 => (false, true, true),
            3 => (true, true, true),
            _ => unreachable!(),
        };

        let decompressed_size = {
            // Bits 0-4 of the control byte are bits 16-20 of decompressed size,
            // before we add 1.
            let size = input.read_be_u16()? as usize;
            (((control_byte & 0x1F) as usize) << 16) + size + 1
        };
        log!("decompressed size: {decompressed_size} bytes");

        let compressed_size = {
            let size = input.read_be_u16()? as usize;
            size + 1
        };
        log!("compressed size: {compressed_size} bytes");

        if reset_dict {
            log!("resetting dictionary");
            self.dict.flush()?;
        }

        if reset_state {
            log!("resetting state");

            if reset_props {
                log!("resetting props");
                self.lzma_dec.set_props(input.read_u8()?)?
            }

            self.lzma_dec.reset_state();
        }

        self.lzma_dec.decompressed_size = Some(decompressed_size);
        self.rc.initialize(input)?;

        while self.dict.len() < decompressed_size {
            log!("decoded {}/{decompressed_size} bytes", self.dict.len());

            self.lzma_dec.decode(&mut self.dict, &mut self.rc, input)?;
        }

        todo!();
    }
}
