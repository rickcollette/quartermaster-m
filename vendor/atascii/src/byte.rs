/// A lossless ATASCII byte.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct AtasciiByte(pub u8);

impl AtasciiByte {
    /// Returns the underlying byte.
    pub const fn get(self) -> u8 {
        self.0
    }
    /// Returns true when inverse-video bit 7 is set.
    pub const fn is_inverse(self) -> bool {
        self.0 & 0x80 != 0
    }
    /// Returns the base 7-bit code.
    pub const fn base(self) -> u8 {
        self.0 & 0x7f
    }
    /// Returns the same base code with the requested inverse state.
    pub const fn with_inverse(self, inverse: bool) -> Self {
        Self(self.base() | if inverse { 0x80 } else { 0 })
    }
    /// Toggles inverse video.
    pub const fn toggled_inverse(self) -> Self {
        Self(self.0 ^ 0x80)
    }
}

impl From<u8> for AtasciiByte {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<AtasciiByte> for u8 {
    fn from(value: AtasciiByte) -> Self {
        value.0
    }
}
