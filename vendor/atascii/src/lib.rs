#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Lossless primitives and state machines for Atari 8-bit ATASCII.
//!
//! This crate deliberately distinguishes transport normalization, raw ATASCII
//! bytes, interpreted editor commands, rendered glyphs, Atari screen codes,
//! keyboard output policies, and lossy Unicode/ASCII text.

mod byte;
mod charset;
mod control;
mod input;
mod newline;
mod parser;
mod profile;
mod screen;
mod screen_code;
mod terminal;
mod translate;

pub use byte::AtasciiByte;
pub use charset::{Charset, Glyph, GlyphId};
pub use control::{Control, Token};
pub use input::{
    encode_backspace, encode_profile_backspace, encode_profile_return, encode_return, EncodedKey,
};
pub use newline::{IncomingNewlineDecoder, NewlineEvidence, NormalizedBytes};
pub use parser::{DecodeDomain, Parser, ParserMode};
pub use profile::{
    AsciiCompatibility, IncomingNewlinePolicy, OutgoingBackspace, OutgoingNewlinePolicy,
    TerminalProfile,
};
pub use screen::{Cell, Screen, ScreenError};
pub use screen_code::{atascii_to_screen, screen_to_atascii, ScreenCode};
pub use terminal::TerminalDecoder;
pub use translate::{
    decode_ascii, decode_unicode, encode_ascii, DecodePolicy, EncodePolicy, TranslationError,
};
