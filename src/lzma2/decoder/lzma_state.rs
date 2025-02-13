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
