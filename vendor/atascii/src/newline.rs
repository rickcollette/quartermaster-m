use crate::{AtasciiByte, IncomingNewlinePolicy};

/// Counts observed newline conventions without modifying the stream.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NewlineEvidence {
    /// Number of native ATASCII EOL bytes (`0x9B`).
    pub atascii_eol: usize,
    /// Number of adjacent ASCII CRLF pairs.
    pub crlf_pairs: usize,
    /// Number of unmatched ASCII CR bytes.
    pub bare_cr: usize,
    /// Number of unmatched ASCII LF bytes.
    pub bare_lf: usize,
}

impl NewlineEvidence {
    /// Returns a conservative recommendation when one convention clearly dominates.
    pub const fn recommendation(self) -> Option<IncomingNewlinePolicy> {
        if self.atascii_eol > self.crlf_pairs + self.bare_cr + self.bare_lf {
            Some(IncomingNewlinePolicy::NativeAtascii)
        } else if self.crlf_pairs > self.atascii_eol + self.bare_cr + self.bare_lf {
            Some(IncomingNewlinePolicy::CrLfToAtasciiEol)
        } else if self.bare_cr > self.atascii_eol + self.crlf_pairs + self.bare_lf {
            Some(IncomingNewlinePolicy::CrToAtasciiEol)
        } else if self.bare_lf > self.atascii_eol + self.crlf_pairs + self.bare_cr {
            Some(IncomingNewlinePolicy::LfToAtasciiEol)
        } else {
            None
        }
    }
}

/// A tiny fixed-capacity result used by the streaming newline normalizer.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NormalizedBytes {
    bytes: [u8; 2],
    len: u8,
}

impl NormalizedBytes {
    const fn none() -> Self {
        Self {
            bytes: [0; 2],
            len: 0,
        }
    }
    const fn one(a: u8) -> Self {
        Self {
            bytes: [a, 0],
            len: 1,
        }
    }
    const fn two(a: u8, b: u8) -> Self {
        Self {
            bytes: [a, b],
            len: 2,
        }
    }
    /// Returns the normalized bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len as usize]
    }
}

/// Stateful transport newline normalizer.
#[derive(Clone, Debug)]
pub struct IncomingNewlineDecoder {
    policy: IncomingNewlinePolicy,
    pending_cr: bool,
    evidence: NewlineEvidence,
}

impl IncomingNewlineDecoder {
    /// Creates a decoder for the selected policy.
    pub const fn new(policy: IncomingNewlinePolicy) -> Self {
        Self {
            policy,
            pending_cr: false,
            evidence: NewlineEvidence {
                atascii_eol: 0,
                crlf_pairs: 0,
                bare_cr: 0,
                bare_lf: 0,
            },
        }
    }

    /// Returns the active policy.
    pub const fn policy(&self) -> IncomingNewlinePolicy {
        self.policy
    }

    /// Returns accumulated newline evidence.
    pub const fn evidence(&self) -> NewlineEvidence {
        self.evidence
    }

    /// Changes policy and clears pending CR state while retaining evidence.
    pub fn set_policy(&mut self, policy: IncomingNewlinePolicy) {
        self.policy = policy;
        self.pending_cr = false;
    }

    /// Feeds one transport byte and returns zero, one, or two normalized bytes.
    pub fn feed(&mut self, byte: AtasciiByte) -> NormalizedBytes {
        let b = byte.get();
        if b == 0x9B {
            self.evidence.atascii_eol += 1;
        }

        match self.policy {
            IncomingNewlinePolicy::NativeAtascii
            | IncomingNewlinePolicy::Preserve
            | IncomingNewlinePolicy::DetectOnly => {
                self.observe_preserved(b);
                NormalizedBytes::one(b)
            }
            IncomingNewlinePolicy::CrToAtasciiEol => {
                if b == 0x0D {
                    NormalizedBytes::one(0x9B)
                } else {
                    NormalizedBytes::one(b)
                }
            }
            IncomingNewlinePolicy::LfToAtasciiEol => {
                if b == 0x0A {
                    NormalizedBytes::one(0x9B)
                } else {
                    NormalizedBytes::one(b)
                }
            }
            IncomingNewlinePolicy::CrLfToAtasciiEol => self.feed_crlf(b, false),
            IncomingNewlinePolicy::CrOrLfToAtasciiEol => self.feed_crlf(b, true),
        }
    }

    /// Flushes a pending unmatched CR at end of stream.
    pub fn finish(&mut self) -> NormalizedBytes {
        if !self.pending_cr {
            return NormalizedBytes::none();
        }
        self.pending_cr = false;
        self.evidence.bare_cr += 1;
        match self.policy {
            IncomingNewlinePolicy::CrOrLfToAtasciiEol => NormalizedBytes::one(0x9B),
            IncomingNewlinePolicy::CrLfToAtasciiEol => NormalizedBytes::one(0x0D),
            _ => NormalizedBytes::none(),
        }
    }

    fn observe_preserved(&mut self, b: u8) {
        if self.pending_cr {
            self.pending_cr = false;
            if b == 0x0A {
                self.evidence.crlf_pairs += 1;
                return;
            }
            self.evidence.bare_cr += 1;
        }
        if b == 0x0D {
            self.pending_cr = true;
        } else if b == 0x0A {
            self.evidence.bare_lf += 1;
        }
    }

    fn feed_crlf(&mut self, b: u8, convert_bare: bool) -> NormalizedBytes {
        if self.pending_cr {
            self.pending_cr = false;
            if b == 0x0A {
                self.evidence.crlf_pairs += 1;
                return NormalizedBytes::one(0x9B);
            }
            self.evidence.bare_cr += 1;
            let first = if convert_bare { 0x9B } else { 0x0D };
            if b == 0x0D {
                self.pending_cr = true;
                return NormalizedBytes::one(first);
            }
            if b == 0x0A {
                self.evidence.bare_lf += 1;
                let second = if convert_bare { 0x9B } else { 0x0A };
                return NormalizedBytes::two(first, second);
            }
            return NormalizedBytes::two(first, b);
        }

        if b == 0x0D {
            self.pending_cr = true;
            NormalizedBytes::none()
        } else if b == 0x0A {
            self.evidence.bare_lf += 1;
            if convert_bare {
                NormalizedBytes::one(0x9B)
            } else {
                NormalizedBytes::one(0x0A)
            }
        } else {
            NormalizedBytes::one(b)
        }
    }
}
