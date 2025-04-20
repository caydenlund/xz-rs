use crate::error::DecodeResult;
use crate::util::InputRead;

#[derive(Debug, Clone, Default)]
pub struct RangeDecoder {
    pub range: u32,
    pub code: u32,
}

impl RangeDecoder {
    /// Used to determine whether the range has a byte of free space.
    const RANGE_MIN: u32 = 0x0100_0000;

    /// For 2048 probability states, according to lzma spec.
    const BIT_MODEL_TOTAL_BITS: u32 = 11;

    /// The maximum probability of a bit being 0.
    const PROB_MAX: u16 = 0x800;

    /// Makes a new [`RangeCoder`] with the initial code from `input`.
    pub fn new<R: InputRead>(input: &mut R) -> DecodeResult<Self> {
        let mut rc = Self::default();
        rc.initialize(input)?;
        Ok(rc)
    }

    /// If `self.range` has at least one byte of free space,
    /// then read one byte from the input into `self.code`.
    pub fn normalize<R: InputRead>(&mut self, input: &mut R) -> DecodeResult<()> {
        if self.range < Self::RANGE_MIN {
            self.range <<= 8;
            self.code = (self.code << 8) + (input.read_u8()? as u32);
        }

        Ok(())
    }

    /// Decodes one bit from `self.code` using probability model.
    /// Performs `self.normalize()` as necessary.
    /// Updates given probability `prob` based on whether the bit is 0 or 1.
    pub fn decode_bit<R: InputRead>(
        &mut self,
        input: &mut R,
        prob: &mut u16,
    ) -> DecodeResult<bool> {
        let bound = (self.range >> Self::BIT_MODEL_TOTAL_BITS) * (*prob as u32);

        let bit = self.code >= bound;
        if bit {
            *prob -= *prob >> 5; // more likely to be 1
            self.code -= bound;
            self.range -= bound;
        } else {
            *prob += (Self::PROB_MAX - *prob) >> 5; // more likely to be 0
            self.range = bound;
        }

        self.normalize(input)?;

        Ok(bit)
    }

    pub(crate) fn initialize<R: InputRead>(&mut self, input: &mut R) -> std::io::Result<()> {
        input.read_u8()?; // skip first byte
        self.range = u32::MAX;
        self.code = input.read_be_u32()?; // next 4 bytes are the initial code
        Ok(())
    }

    pub(crate) fn bit_tree<R: InputRead>(
        &mut self,
        input: &mut R,
        probs: &mut [u16],
        limit: usize,
    ) -> DecodeResult<usize> {
        let mut symbol = 1;
        while symbol < limit {
            let bit = self.decode_bit(input, &mut probs[symbol])?;
            symbol = (symbol << 1) + bit as usize;
        }
        Ok(symbol)
    }

    pub(crate) fn bit_tree_rev<R: InputRead>(
        &mut self,
        input: &mut R,
        probs: &mut [u16],
        mut initial: usize,
        limit: usize,
    ) -> DecodeResult<usize> {
        let mut symbol = 1;

        for i in 0..limit.max(1) {
            let bit = self.decode_bit(input, &mut probs[symbol])?;
            if bit {
                initial += 1 << i;
            }
            symbol = (symbol << 1) + bit as usize;
        }

        Ok(initial)
    }

    pub(crate) fn direct<R: InputRead>(
        &mut self,
        input: &mut R,
        mut initial: u32,
        limit: usize,
    ) -> DecodeResult<u32> {
        for _ in 0..limit {
            self.normalize(input)?;
            self.range >>= 1;
            let bit = self.code >= self.range;
            if bit {
                self.code -= self.range
            };
            initial = (initial << 1) + (bit as u32);
        }
        Ok(initial)
    }
}
