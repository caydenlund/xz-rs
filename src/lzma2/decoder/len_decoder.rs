use logomotion::{func, log};

use crate::{error::DecodeResult, util::InputRead};

use super::{lzma_decoder::LzmaDecoder, range_decoder::RangeDecoder};

pub(crate) struct LenDecoder {
    /// Probability of match length being >= 10.
    choice: u16,

    /// Probability of match length being >= 18.
    choice2: u16,

    /// Probabilities for match lengths 0-9.
    low: [[u16; Self::LEN_LOW_SYMBOLS]; LzmaDecoder::POS_STATES_MAX],

    /// Probabilities for match lengths 10-17.
    med: [[u16; Self::LEN_MID_SYMBOLS]; LzmaDecoder::POS_STATES_MAX],

    /// Probabilities for match lengths 18-273.
    high: [u16; Self::LEN_HIGH_SYMBOLS],
}

impl LenDecoder {
    const LEN_LOW_BITS: usize = 3;
    const LEN_LOW_SYMBOLS: usize = 1 << Self::LEN_LOW_BITS;

    const LEN_MID_BITS: usize = 3;
    const LEN_MID_SYMBOLS: usize = 1 << Self::LEN_MID_BITS;

    const LEN_HIGH_BITS: usize = 8;
    const LEN_HIGH_SYMBOLS: usize = 1 << Self::LEN_HIGH_BITS;

    pub(crate) fn new() -> Self {
        Self {
            choice: LzmaDecoder::DEFAULT_PROB,
            choice2: LzmaDecoder::DEFAULT_PROB,
            low: [[LzmaDecoder::DEFAULT_PROB; Self::LEN_LOW_SYMBOLS]; LzmaDecoder::POS_STATES_MAX],
            med: [[LzmaDecoder::DEFAULT_PROB; Self::LEN_MID_SYMBOLS]; LzmaDecoder::POS_STATES_MAX],
            high: [LzmaDecoder::DEFAULT_PROB; Self::LEN_HIGH_SYMBOLS],
        }
    }

    pub(crate) fn reset(&mut self) {
        let _ctx = func!("LenDecoder::reset()");

        self.choice = LzmaDecoder::DEFAULT_PROB;
        self.choice2 = LzmaDecoder::DEFAULT_PROB;
        self.low
            .fill([LzmaDecoder::DEFAULT_PROB; Self::LEN_LOW_SYMBOLS]);
        self.med
            .fill([LzmaDecoder::DEFAULT_PROB; Self::LEN_MID_SYMBOLS]);
        self.high.fill(LzmaDecoder::DEFAULT_PROB);
    }

    pub(crate) fn decode<R: InputRead>(
        &mut self,
        input: &mut R,
        rc: &mut RangeDecoder,
        pos_state: usize,
    ) -> DecodeResult<usize> {
        let _ctx =
            func!("LenDecoder::decode(input, rc, pos_state: 0x{pos_state:08X} ({pos_state}))");

        let (probs, limit, start): (&mut [u16], usize, usize) =
            if !rc.decode_bit(input, &mut self.choice)? {
                log!("low length");
                (&mut self.low[pos_state], Self::LEN_LOW_SYMBOLS, 2)
            } else if !rc.decode_bit(input, &mut self.choice2)? {
                log!("mid length");
                (
                    &mut self.med[pos_state],
                    Self::LEN_MID_SYMBOLS,
                    2 + Self::LEN_LOW_SYMBOLS,
                )
            } else {
                log!("high length");
                (
                    &mut self.high,
                    Self::LEN_HIGH_SYMBOLS,
                    2 + Self::LEN_LOW_SYMBOLS + Self::LEN_MID_SYMBOLS,
                )
            };

        let result = start + rc.bit_tree(input, probs, limit)? - limit;
        log!("result: {result}",);
        Ok(result)
    }
}
