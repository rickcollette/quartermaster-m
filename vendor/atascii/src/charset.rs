use crate::AtasciiByte;

/// Atari ROM character-set selection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Charset {
    /// Original 400/800 and XL/XE standard set.
    #[default]
    Standard,
    /// XL/XE international set.
    International,
}

/// Stable identifier for one of the 128 base glyph shapes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GlyphId(pub u8);

/// A glyph plus display attributes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Glyph {
    /// Base 7-bit ATASCII code.
    pub id: GlyphId,
    /// Whether inverse video is requested.
    pub inverse: bool,
    /// Character ROM selection.
    pub charset: Charset,
}

impl Glyph {
    /// Constructs a glyph directly from a byte without treating it as control.
    pub const fn quoted(byte: AtasciiByte, charset: Charset) -> Self {
        Self {
            id: GlyphId(byte.base()),
            inverse: byte.is_inverse(),
            charset,
        }
    }
    /// Converts the glyph back into its raw ATASCII byte.
    pub const fn to_byte(self) -> AtasciiByte {
        AtasciiByte(self.id.0 | if self.inverse { 0x80 } else { 0 })
    }
    /// Returns a practical Unicode approximation of the standard glyph.
    ///
    /// Unicode cannot preserve Atari inverse-video or every original bitmap.
    pub const fn unicode_approx(self) -> char {
        let b = self.id.0;
        if b >= 0x20 && b <= 0x7a {
            return b as char;
        }
        match b {
            0x00 => '♥',
            0x01 => '├',
            0x02 => '▕',
            0x03 => '┘',
            0x04 => '┤',
            0x05 => '┐',
            0x06 => '╱',
            0x07 => '╲',
            0x08 => '◢',
            0x09 => '▗',
            0x0a => '◣',
            0x0b => '▝',
            0x0c => '▘',
            0x0d => '▔',
            0x0e => '▂',
            0x0f => '▖',
            0x10 => '♣',
            0x11 => '┌',
            0x12 => '─',
            0x13 => '┬',
            0x14 => '┴',
            0x15 => '▌',
            0x16 => '└',
            0x17 => '│',
            0x18 => '♦',
            0x19 => '┼',
            0x1a => '●',
            0x1b => '␛',
            0x1c => '↑',
            0x1d => '↓',
            0x1e => '←',
            0x1f => '→',
            0x60 => '◆',
            0x7b => '♠',
            0x7c => '|',
            0x7d => '↶',
            0x7e => '◀',
            0x7f => '▶',
            _ => '�',
        }
    }
}
