use crate::AtasciiByte;

/// Atari internal/screen code, including inverse bit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScreenCode(pub u8);

/// Converts ATASCII interchange code to Atari internal screen code.
pub const fn atascii_to_screen(value: AtasciiByte) -> ScreenCode {
    let inverse = value.0 & 0x80;
    let b = value.0 & 0x7f;
    let s = if b < 0x20 {
        b + 0x40
    } else if b < 0x60 {
        b - 0x20
    } else {
        b
    };
    ScreenCode(s | inverse)
}

/// Converts Atari internal screen code to ATASCII interchange code.
pub const fn screen_to_atascii(value: ScreenCode) -> AtasciiByte {
    let inverse = value.0 & 0x80;
    let b = value.0 & 0x7f;
    let a = if b < 0x40 {
        b + 0x20
    } else if b < 0x60 {
        b - 0x40
    } else {
        b
    };
    AtasciiByte(a | inverse)
}
