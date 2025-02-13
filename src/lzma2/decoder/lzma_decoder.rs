use std::io::Cursor;

use crate::error::{DecodeError, DecodeResult};
use crate::lzma2::Lzma2DecodeError;

use super::lzma_state::LzmaState;

pub(crate) struct LzmaDecoder {
    /// Number of literal context bits
    lc_bits: u32,
    /// Mask from the literal position bits (1 << lp - 1)
    lp_mask: u32,
    /// Mask from the number position bits (1 << pb - 1)
    pb_mask: u32,

    pub(crate) uncompressed_size: Option<usize>,

    /// The most-recently seen symbols
    state: LzmaState,

    temp_buffer: Cursor<[u8; Self::MAX_REQUIRED_INPUT]>,
}

impl LzmaDecoder {
    /// The maximum amount of input that can be consumed in a single iteration.
    /// This value comes from the Linux kernel's lzma2 implementation.
    const MAX_REQUIRED_INPUT: usize = 21;

    pub fn new() -> Self {
        Self {
            lc_bits: 0,
            lp_mask: 0,
            pb_mask: 0,
            uncompressed_size: None,
            state: LzmaState::default(),
            temp_buffer: Cursor::default(),
        }
    }

    pub fn set_props(&mut self, props: u8) -> DecodeResult<()> {
        let err = Err(DecodeError::from(Lzma2DecodeError::InvalidProperties));

        todo!();
        return err;
    }
}

impl Default for LzmaDecoder {
    fn default() -> Self {
        Self::new()
    }
}
