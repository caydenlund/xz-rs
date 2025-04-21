use crate::error::EncodeResult;

use super::dict::Dict;
use super::lzma_encoder::LzmaEncoder;
use super::range_encoder::RangeEncoder;
use std::io::{BufRead, Write};

pub(crate) struct Lzma2Encoder<W: Write> {
    lzma_enc: LzmaEncoder,
    dict: Dict<W>,
    rc: RangeEncoder,
}

impl<W: Write> Lzma2Encoder<W> {
    pub fn new(output: &mut W) -> Self {
        todo!()
    }

    pub fn encode<R: BufRead>(&mut self, input: &mut R) -> EncodeResult<()> {
        todo!()
    }
}
