pub fn atascii_to_screen(code: u8) -> u8 {
    let inverse = code & 0x80;
    let base = code & 0x7f;
    let screen = match base {
        0..=31 => base + 64,
        32..=95 => base - 32,
        _ => base,
    };
    screen | inverse
}

pub fn screen_to_atascii(code: u8) -> u8 {
    let inverse = code & 0x80;
    let base = code & 0x7f;
    let atascii = match base {
        0..=63 => base + 32,
        64..=95 => base - 64,
        _ => base,
    };
    atascii | inverse
}
