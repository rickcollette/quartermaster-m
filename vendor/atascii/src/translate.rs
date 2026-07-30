extern crate alloc;

use crate::{AtasciiByte, Charset, Control, DecodeDomain, Parser, Token};

/// Policy for decoding ATASCII into modern text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodePolicy {
    /// Preserve approximated graphics and map inverse glyphs normally.
    UnicodeApprox,
    /// Replace non-ASCII graphics with `?`.
    AsciiSafe,
    /// Emit visible tags such as `<CursorUp>` for controls.
    Debug,
}

/// Policy for encoding modern text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodePolicy {
    /// Reject unrepresentable input.
    Strict,
    /// Replace unrepresentable input with `?`.
    Replace,
    /// Recognize common Unicode approximations for Atari graphics.
    GraphicsApprox,
}

/// Translation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranslationError {
    /// Unicode character that could not be encoded.
    pub character: char,
}

/// Decodes an ATASCII editor stream to Unicode according to a stated lossy policy.
pub fn decode_unicode(bytes: &[u8], policy: DecodePolicy) -> alloc::string::String {
    use core::fmt::Write;

    let mut output = alloc::string::String::new();
    let mut parser = Parser::new(DecodeDomain::ScreenEditor, Charset::Standard);

    for &byte in bytes {
        let Some(token) = parser.feed(AtasciiByte(byte)) else {
            continue;
        };

        match token {
            Token::Glyph(glyph) => {
                let character = glyph.unicode_approx();
                output.push(
                    if policy == DecodePolicy::AsciiSafe && !character.is_ascii() {
                        '?'
                    } else {
                        character
                    },
                );
            }
            Token::Control(Control::EndOfLine) => output.push('\n'),
            Token::Control(control) if policy == DecodePolicy::Debug => {
                let _ = write!(output, "<{control:?}>");
            }
            Token::Control(_) | Token::Raw(_) => {}
        }
    }

    output
}

/// Decodes ATASCII using ASCII-safe replacement rules.
pub fn decode_ascii(bytes: &[u8]) -> alloc::string::String {
    decode_unicode(bytes, DecodePolicy::AsciiSafe)
}

/// Encodes Unicode text to ATASCII.
///
/// Newline (`\n`) is encoded as native ATASCII EOL (`0x9B`). Carriage returns
/// are ignored so that a Rust string containing CRLF does not produce two EOLs.
pub fn encode_ascii(
    text: &str,
    policy: EncodePolicy,
) -> Result<alloc::vec::Vec<u8>, TranslationError> {
    let mut output = alloc::vec::Vec::new();

    for character in text.chars() {
        let byte = match character {
            '\n' => 0x9B,
            '\r' => continue,
            character if (' '..='z').contains(&character) => character as u8,
            '♥' => 0x00,
            '├' => 0x01,
            '┘' => 0x03,
            '┤' => 0x04,
            '┐' => 0x05,
            '╱' => 0x06,
            '╲' => 0x07,
            '♣' => 0x10,
            '┌' => 0x11,
            '─' => 0x12,
            '┬' => 0x13,
            '┴' => 0x14,
            '└' => 0x16,
            '│' => 0x17,
            '♦' => 0x18,
            '┼' => 0x19,
            '↑' if policy == EncodePolicy::GraphicsApprox => 0x1C,
            '↓' if policy == EncodePolicy::GraphicsApprox => 0x1D,
            '←' if policy == EncodePolicy::GraphicsApprox => 0x1E,
            '→' if policy == EncodePolicy::GraphicsApprox => 0x1F,
            '♠' if policy == EncodePolicy::GraphicsApprox => 0x7B,
            _ if policy != EncodePolicy::Strict => b'?',
            _ => return Err(TranslationError { character }),
        };
        output.push(byte);
    }

    Ok(output)
}
