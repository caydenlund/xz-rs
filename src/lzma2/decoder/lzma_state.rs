#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LzmaState {
    #[default]
    LitLit,
    MatchLitLit,
    RepLitLit,
    ShortrepLitLit,
    MatchLit,
    RepLit,
    ShortrepLit,
    LitMatch,
    LitLongrep,
    LitShortrep,
    NonlitMatch,
    NonlitRep,
}

impl LzmaState {
    pub(crate) const NUM_STATES: usize = 12;
    pub(crate) const NUM_LITERAL_STATES: usize = 7;

    #[inline(always)]
    pub(crate) fn is_literal(&self) -> bool {
        (*self as u8) < Self::NUM_LITERAL_STATES as u8
    }

    #[inline(always)]
    pub(crate) fn state_literal(&mut self) {
        *self = if *self <= Self::ShortrepLitLit {
            Self::LitLit
        } else if *self <= Self::LitShortrep {
            Self::from(*self as u8 - 3)
        } else {
            Self::from(*self as u8 - 6)
        };
    }

    #[inline(always)]
    pub(crate) fn state_short_rep(&mut self) {
        *self = if self.is_literal() {
            Self::LitShortrep
        } else {
            Self::NonlitRep
        };
    }

    pub(crate) fn state_long_rep(&mut self) {
        *self = if self.is_literal() {
            Self::LitLongrep
        } else {
            Self::NonlitRep
        };
    }

    pub(crate) fn state_match(&mut self) {
        *self = if self.is_literal() {
            Self::LitMatch
        } else {
            Self::NonlitMatch
        };
    }
}

impl From<u8> for LzmaState {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<LzmaState> for u8 {
    fn from(state: LzmaState) -> Self {
        unsafe { std::mem::transmute(state) }
    }
}
