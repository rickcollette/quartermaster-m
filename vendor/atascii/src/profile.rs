/// How incoming transport newline bytes are normalized before ATASCII parsing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum IncomingNewlinePolicy {
    /// Strict native ATASCII: only `0x9B` is end-of-line; CR and LF remain glyph bytes.
    #[default]
    NativeAtascii,
    /// Convert ASCII carriage return (`0x0D`) to ATASCII EOL (`0x9B`).
    CrToAtasciiEol,
    /// Convert ASCII line feed (`0x0A`) to ATASCII EOL (`0x9B`).
    LfToAtasciiEol,
    /// Collapse CRLF to one ATASCII EOL while preserving unmatched CR or LF bytes.
    CrLfToAtasciiEol,
    /// Convert CR, LF, or CRLF to one ATASCII EOL.
    CrOrLfToAtasciiEol,
    /// Preserve all bytes while collecting newline evidence for profile selection.
    DetectOnly,
    /// Preserve all transport bytes unchanged.
    Preserve,
}

/// How the local Return key is encoded for the remote endpoint.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutgoingNewlinePolicy {
    /// Send native ATASCII EOL (`0x9B`).
    #[default]
    AtasciiEol,
    /// Send ASCII carriage return (`0x0D`).
    Cr,
    /// Send ASCII line feed (`0x0A`).
    Lf,
    /// Send ASCII CRLF (`0x0D 0x0A`).
    CrLf,
}

/// How the local Backspace key is encoded for the remote endpoint.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutgoingBackspace {
    /// Send native ATASCII Delete/Backspace (`0x7E`).
    #[default]
    AtasciiDelete,
    /// Send native ATASCII Delete Character (`0xFE`).
    AtasciiDeleteCharacter,
    /// Send ASCII Backspace (`0x08`).
    AsciiBackspace,
    /// Send ASCII DEL (`0x7F`). This conflicts with native ATASCII Tab.
    AsciiDelete,
}

/// Optional compatibility conversions for non-native hosts.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AsciiCompatibility {
    /// Treat incoming ASCII BS (`0x08`) as Atari Delete/Backspace.
    pub backspace: bool,
    /// Treat incoming ASCII BEL (`0x07`) as Atari Buzzer.
    pub bell: bool,
    /// Treat incoming ASCII form feed (`0x0C`) as Atari Clear Screen.
    pub form_feed: bool,
}

/// A complete terminal connection profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TerminalProfile {
    /// Incoming newline normalization policy.
    pub incoming_newlines: IncomingNewlinePolicy,
    /// Outgoing Return-key encoding.
    pub outgoing_newline: OutgoingNewlinePolicy,
    /// Outgoing Backspace-key encoding.
    pub outgoing_backspace: OutgoingBackspace,
    /// Optional ASCII control compatibility.
    pub ascii_compatibility: AsciiCompatibility,
}

impl OutgoingNewlinePolicy {
    /// Returns the bytes to transmit for the local Return key.
    pub const fn bytes(self) -> &'static [u8] {
        match self {
            Self::AtasciiEol => &[0x9B],
            Self::Cr => &[0x0D],
            Self::Lf => &[0x0A],
            Self::CrLf => &[0x0D, 0x0A],
        }
    }
}

impl OutgoingBackspace {
    /// Returns the byte to transmit for the local Backspace key.
    pub const fn byte(self) -> u8 {
        match self {
            Self::AtasciiDelete => 0x7E,
            Self::AtasciiDeleteCharacter => 0xFE,
            Self::AsciiBackspace => 0x08,
            Self::AsciiDelete => 0x7F,
        }
    }
}
