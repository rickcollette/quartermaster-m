use crate::{AtasciiByte, Charset, IncomingNewlineDecoder, Parser, TerminalProfile, Token};

/// Combines transport newline normalization with ATASCII parsing.
#[derive(Clone, Debug)]
pub struct TerminalDecoder {
    newline: IncomingNewlineDecoder,
    parser: Parser,
}

impl TerminalDecoder {
    /// Creates a terminal decoder from a connection profile and parser domain.
    pub const fn new(
        profile: TerminalProfile,
        domain: crate::DecodeDomain,
        charset: Charset,
    ) -> Self {
        Self {
            newline: IncomingNewlineDecoder::new(profile.incoming_newlines),
            parser: Parser::with_compatibility(domain, charset, profile.ascii_compatibility),
        }
    }

    /// Feeds a byte slice and emits parsed tokens through the supplied callback.
    pub fn push(&mut self, input: &[u8], sink: &mut impl FnMut(Token)) {
        for &byte in input {
            let normalized = self.newline.feed(AtasciiByte(byte));
            for &item in normalized.as_slice() {
                if let Some(token) = self.parser.feed(AtasciiByte(item)) {
                    sink(token);
                }
            }
        }
    }

    /// Flushes pending transport state at end of stream.
    pub fn finish(&mut self, sink: &mut impl FnMut(Token)) {
        let normalized = self.newline.finish();
        for &item in normalized.as_slice() {
            if let Some(token) = self.parser.feed(AtasciiByte(item)) {
                sink(token);
            }
        }
    }

    /// Returns the newline evidence collected so far.
    pub const fn newline_evidence(&self) -> crate::NewlineEvidence {
        self.newline.evidence()
    }

    /// Returns whether the ATASCII parser is waiting for an ESC-quoted byte.
    pub const fn has_pending_escape(&self) -> bool {
        self.parser.has_pending_escape()
    }
}
