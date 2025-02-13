use std::io::Write;

use crate::{error::DecodeResult, util::InputRead};

mod dict;
mod lzma2_decoder;
mod lzma_decoder;
mod lzma_state;
mod range_decoder;

pub fn decode_lzma2<R: InputRead, W: Write>(input: &mut R, output: &mut W) -> DecodeResult<()> {
    Ok(())
}
