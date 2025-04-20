use crate::error::DecodeResult;
use crate::util::InputRead;
use lzma2_decoder::Lzma2Decoder;
use std::io::Write;

mod dict;
mod len_decoder;
mod lzma2_decoder;
mod lzma_decoder;
mod lzma_state;
mod range_decoder;

pub fn decode_lzma2<R: InputRead, W: Write>(input: &mut R, output: &mut W) -> DecodeResult<()> {
    let mut decoder = Lzma2Decoder::new(output);
    decoder.decode(input)
}
