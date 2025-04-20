use std::io::Write;

use logomotion::{func, log};

use crate::error::{DecodeError, DecodeResult};
use crate::lzma2::Lzma2DecodeError;
use crate::util::InputRead;

use super::dict::Dict;
use super::len_decoder::LenDecoder;
use super::lzma_state::LzmaState;
use super::range_decoder::RangeDecoder;

pub(crate) struct LzmaDecoder {
    /// Number of literal context bits.
    lc_bits: u32,
    /// Mask from the literal position bits: `1 << lp - 1`.
    lp_mask: usize,
    /// Mask from the number position bits: `1 << pb - 1`.
    pb_mask: usize,

    pub(crate) decompressed_size: Option<usize>,

    /// If 1, it's a match. Otherwise, it's a literal byte.
    is_match: [[u16; Self::POS_STATES_MAX]; LzmaState::NUM_STATES],

    /// If 1, the match distance is in `self.rep[]`.
    is_rep: [u16; LzmaState::NUM_STATES],

    /// If 0, the match distance is in `self.rep[0]`.
    is_rep0: [u16; LzmaState::NUM_STATES],

    /// If 0, the match distance is in `self.rep[1]`.
    is_rep1: [u16; LzmaState::NUM_STATES],

    /// If 0, the match distance is in `self.rep[2]`.
    /// Otherwise, it's in `self.rep[3]`.
    is_rep2: [u16; LzmaState::NUM_STATES],

    /// If 1, the repeated match has length 1.
    /// Otherwise, decode the length with `self.rep_len_decoder`.
    is_rep0_long: [[u16; Self::POS_STATES_MAX]; LzmaState::NUM_STATES],

    /// Probability trees for the highest 2 bits of the match distance.
    /// Separate tree for match lengths of 2, 3, 4, and [5, 273].
    dist_slot: [[u16; Self::DIST_SLOTS]; Self::DIST_STATES],

    /// Probability trees for additional bits for match distance
    /// when the distance is in the range [4, 127].
    dist_special: [u16; 128 - Self::DIST_MODEL_END],

    /// Probability trees for the lowest 4 bits for match distance
    /// when the distance >= 128.
    dist_align: [u16; 1 << Self::ALIGN_BITS],

    /// Probabilities of literals.
    literal: [[u16; Self::LITERAL_CODER_SIZE]; Self::LITERAL_CODERS_MAX],

    /// Most-recent 4 match distances.
    rep: [usize; 4],

    /// The most-recently seen symbols.
    state: LzmaState,

    /// Length of a normal match.
    match_len_dec: LenDecoder,

    /// Length of a repeated match.
    rep_len_dec: LenDecoder,
}

impl LzmaDecoder {
    /// The maximum number of position states, depending on the number of pb bits.
    /// (The maximum number of pb bits is 4.)
    pub(crate) const POS_STATES_MAX: usize = 1 << 4;

    /// The default probability of a bit being 0 or 1.
    /// (I.e., exactly in the middle of the probability range.)
    pub(crate) const DEFAULT_PROB: u16 = 0x0400;

    /// The maximum number of literal coders
    const LITERAL_CODERS_MAX: usize = (1 << 4);

    const DIST_STATES: usize = 4;
    const DIST_SLOTS: usize = 64;
    const DIST_MODEL_START: usize = 4;
    const DIST_MODEL_END: usize = 14;

    const MATCH_LEN_MIN: usize = 2;
    const ALIGN_BITS: usize = 4;

    /// Each literal coder is divided into three ranges:
    ///   - 0x001..=0x0FF: Without match byte
    ///   - 0x101..=0x1FF: With match byte; match bit is 0
    ///   - 0x201..=0x2FF: With match byte; match bit is 1
    ///
    /// A match byte is used when the previous LZMA symbol was a match.
    const LITERAL_CODER_SIZE: usize = 0x0300;

    pub fn new() -> Self {
        Self {
            lc_bits: 0,
            lp_mask: 0,
            pb_mask: 0,
            decompressed_size: None,
            is_match: [[Self::DEFAULT_PROB; Self::POS_STATES_MAX]; LzmaState::NUM_STATES],
            is_rep: [Self::DEFAULT_PROB; LzmaState::NUM_STATES],
            is_rep0: [Self::DEFAULT_PROB; LzmaState::NUM_STATES],
            is_rep1: [Self::DEFAULT_PROB; LzmaState::NUM_STATES],
            is_rep2: [Self::DEFAULT_PROB; LzmaState::NUM_STATES],
            is_rep0_long: [[Self::DEFAULT_PROB; Self::POS_STATES_MAX]; LzmaState::NUM_STATES],
            dist_slot: [[Self::DEFAULT_PROB; Self::DIST_SLOTS]; Self::DIST_STATES],
            dist_special: [Self::DEFAULT_PROB; 128 - Self::DIST_MODEL_END],
            dist_align: [Self::DEFAULT_PROB; 1 << Self::ALIGN_BITS],
            literal: [[Self::DEFAULT_PROB; Self::LITERAL_CODER_SIZE]; Self::LITERAL_CODERS_MAX],
            rep: [0; 4],
            state: LzmaState::default(),
            match_len_dec: LenDecoder::new(),
            rep_len_dec: LenDecoder::new(),
        }
    }

    pub(crate) fn reset_state(&mut self) {
        let _ctx = func!("LzmaDecoder::reset_state()");

        self.is_match
            .fill([Self::DEFAULT_PROB; Self::POS_STATES_MAX]);
        self.is_rep.fill(Self::DEFAULT_PROB);
        self.is_rep0.fill(Self::DEFAULT_PROB);
        self.is_rep1.fill(Self::DEFAULT_PROB);
        self.is_rep2.fill(Self::DEFAULT_PROB);
        self.is_rep0_long
            .fill([Self::DEFAULT_PROB; Self::POS_STATES_MAX]);
        self.dist_slot.fill([Self::DEFAULT_PROB; Self::DIST_SLOTS]);
        self.dist_special.fill(Self::DEFAULT_PROB);
        self.dist_align.fill(Self::DEFAULT_PROB);
        self.literal
            .fill([Self::DEFAULT_PROB; Self::LITERAL_CODER_SIZE]);
        self.rep.fill(0);
        self.state = LzmaState::default();

        self.match_len_dec.reset();
        self.rep_len_dec.reset();
        // TODO: Don't forget to reset props here as they're added!
    }

    pub fn set_props(&mut self, props: u8) -> DecodeResult<()> {
        let _ctx = func!("LzmaDecoder::set_props(props: 0x{props:02X})");

        let err = Err(DecodeError::from(Lzma2DecodeError::InvalidProperties));

        if props > (4 * 5 + 4) * 9 + 8 {
            return err;
        }

        let mut props = props as u32;

        let mut pb_bits = 0;
        while props >= 9 * 5 {
            props -= 9 * 5;
            pb_bits += 1;
        }

        log!("pb bits: {pb_bits}");

        self.pb_mask = (1 << pb_bits) - 1;

        let mut lp_bits = 0;
        while props >= 9 {
            props -= 9;
            lp_bits += 1;
        }

        log!("lp bits: {lp_bits}");

        if props + lp_bits > 4 {
            return err;
        }

        log!("lc bits: {props}");

        self.lc_bits = props;

        self.lp_mask = (1 << lp_bits) - 1;

        Ok(())
    }

    pub(crate) fn decode<R: InputRead, W: Write>(
        &mut self,
        dict: &mut Dict<W>,
        rc: &mut RangeDecoder,
        input: &mut R,
    ) -> DecodeResult<()> {
        let _ctx = func!("LzmaDecoder::decode(dict, rc)");

        let pos_state = dict.len() & self.pb_mask;

        if rc.decode_bit(input, &mut self.is_match[self.state as usize][pos_state])? {
            log!("decoding match");

            let (len, dist) = if rc.decode_bit(input, &mut self.is_rep[self.state as usize])? {
                log!("distance is repeated from 1 of the last 4 matches");

                let dist = if !rc.decode_bit(input, &mut self.is_rep0[self.state as usize])? {
                    log!("distance is rep0: {}", self.rep[0]);
                    // rep0 has a special case: "short rep"
                    if !rc.decode_bit(
                        input,
                        &mut self.is_rep0_long[self.state as usize][pos_state],
                    )? {
                        log!("performing a short rep");
                        dict.repeat(1, self.rep[0] + 1);
                        self.state.state_short_rep();
                        return Ok(());
                    }
                    self.rep[0]
                } else {
                    let dist;
                    if !rc.decode_bit(input, &mut self.is_rep1[self.state as usize])? {
                        dist = self.rep[1];
                        log!("distance is rep1: {}", dist);
                    } else {
                        if !rc.decode_bit(input, &mut self.is_rep2[self.state as usize])? {
                            dist = self.rep[2];
                            log!("distance is rep2: {}", dist);
                        } else {
                            dist = self.rep[3];
                            log!("distance is rep3: {}", dist);
                            self.rep[3] = self.rep[2];
                        }
                        self.rep[2] = self.rep[1];
                    }
                    self.rep[1] = self.rep[0];
                    self.rep[0] = dist;
                    dist
                };

                log!("performing a long rep");
                self.state.state_long_rep();

                let len = self.rep_len_dec.decode(input, rc, pos_state)?;
                (len, dist)
            } else {
                log!("distance is not a repeat");

                (0..3).rev().for_each(|i| self.rep[i + 1] = self.rep[i]);
                self.state.state_match();

                let len = self.match_len_dec.decode(input, rc, pos_state)?;
                let dist = self.decode_distance(input, rc, len)?;

                (len, dist)
            };

            log!("len: {len}");
            log!("dist: {dist}");

            dict.repeat(len, dist + 1);

            log!(
                "dict: `{}` (len: {})",
                String::from_utf8(dict.buf.clone())
                    .unwrap_or_default()
                    .replace("\n", "\\n"),
                dict.len()
            );
        } else {
            log!("decoding literal");

            let lit_state = {
                let prev_byte = dict.last().unwrap_or(0) as usize;
                let low = prev_byte >> (8 - self.lc_bits);
                let high = (dict.len() & self.lp_mask) << self.lc_bits;
                low + high
            };
            let literal_probs = &mut self.literal[lit_state];

            let literal = if self.state.is_literal() {
                log!("last-seen symbol was literal");

                // decode 8 bits
                let mut result = 1usize;
                while result < 0x100 {
                    result = (result << 1)
                        + (rc.decode_bit(input, &mut literal_probs[result])? as usize);
                    log!("partial result: 0x{result:02X}");
                }
                result as u8
            } else {
                log!("last-seen symbol was match");
                let mut match_byte = dict.last_n(self.rep[0] + 1).unwrap() as usize;

                // decode 8 bits
                let mut result = 1usize;
                while result < 0x100 {
                    let match_bit = (match_byte >> 7) & 1;
                    match_byte <<= 1;
                    let bit = rc
                        .decode_bit(input, &mut literal_probs[((1 + match_bit) << 8) + result])?
                        as usize;
                    result = (result << 1) + bit;
                    if match_bit != bit {
                        break;
                    }
                }

                while result < 0x100 {
                    result =
                        (result << 1) + rc.decode_bit(input, &mut literal_probs[result])? as usize;
                }

                result as u8
            };

            dict.push(literal);
            self.state.state_literal();
        }

        Ok(())
    }

    fn decode_distance<R: InputRead>(
        &mut self,
        input: &mut R,
        rc: &mut RangeDecoder,
        len: usize,
    ) -> DecodeResult<usize> {
        let _ctx = func!("LzmaDecoder::decode_distance(input, rc, len: {len})");

        let dist_state = if len < Self::DIST_STATES + Self::MATCH_LEN_MIN {
            len - Self::MATCH_LEN_MIN
        } else {
            Self::DIST_STATES - 1
        };
        let probs = &mut self.dist_slot[dist_state];
        let dist_slot = rc.bit_tree(input, probs, Self::DIST_SLOTS)? - Self::DIST_SLOTS;

        let dist = if dist_slot < Self::DIST_MODEL_START {
            dist_slot
        } else {
            let limit = (dist_slot >> 1) - 1;
            let mut rep0 = 2 + (dist_slot & 1);

            if dist_slot < Self::DIST_MODEL_END {
                rep0 <<= limit;
                let probs = &mut self.dist_special[(rep0 - dist_slot - 1)..];
                rc.bit_tree_rev(input, probs, rep0, limit)?
            } else {
                rep0 = rc.direct(input, rep0 as u32, limit - Self::ALIGN_BITS)? as usize;
                rep0 <<= Self::ALIGN_BITS;
                rc.bit_tree_rev(input, &mut self.dist_align, rep0, Self::ALIGN_BITS)?
            }
        };
        self.rep[0] = dist;

        Ok(dist)
    }
}

impl Default for LzmaDecoder {
    fn default() -> Self {
        Self::new()
    }
}
