use crate::{OutgoingBackspace, OutgoingNewlinePolicy, TerminalProfile};

/// A transport-ready encoding of common local terminal keys.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EncodedKey {
    bytes: [u8; 2],
    len: u8,
}

impl EncodedKey {
    const fn one(byte: u8) -> Self {
        Self {
            bytes: [byte, 0],
            len: 1,
        }
    }

    const fn two(first: u8, second: u8) -> Self {
        Self {
            bytes: [first, second],
            len: 2,
        }
    }

    /// Returns the bytes that should be passed to the transport layer.
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

/// Encodes the local Return key according to a terminal profile.
pub const fn encode_return(policy: OutgoingNewlinePolicy) -> EncodedKey {
    match policy {
        OutgoingNewlinePolicy::AtasciiEol => EncodedKey::one(0x9B),
        OutgoingNewlinePolicy::Cr => EncodedKey::one(0x0D),
        OutgoingNewlinePolicy::Lf => EncodedKey::one(0x0A),
        OutgoingNewlinePolicy::CrLf => EncodedKey::two(0x0D, 0x0A),
    }
}

/// Encodes the local Backspace key according to a terminal profile.
pub const fn encode_backspace(policy: OutgoingBackspace) -> EncodedKey {
    EncodedKey::one(policy.byte())
}

/// Encodes Return using a complete terminal profile.
pub const fn encode_profile_return(profile: TerminalProfile) -> EncodedKey {
    encode_return(profile.outgoing_newline)
}

/// Encodes Backspace using a complete terminal profile.
pub const fn encode_profile_backspace(profile: TerminalProfile) -> EncodedKey {
    encode_backspace(profile.outgoing_backspace)
}
