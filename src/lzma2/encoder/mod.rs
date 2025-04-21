use crate::error::EncodeResult;
use lzma2_encoder::Lzma2Encoder;
use std::io::{BufRead, Write};

mod dict;
mod lzma2_encoder;
mod lzma_encoder;
mod range_encoder;

pub fn encode_lzma2<R: BufRead, W: Write>(input: &mut R, output: &mut W) -> EncodeResult<()> {
    let mut encoder = Lzma2Encoder::new(output);
    encoder.encode(input)
}
