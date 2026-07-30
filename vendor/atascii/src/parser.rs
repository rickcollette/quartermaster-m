use crate::{AsciiCompatibility, AtasciiByte, Charset, Control, Glyph, Token};

/// Selects the semantic domain used to interpret incoming bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DecodeDomain {
    /// Execute Atari E: screen-editor controls and ESC quoting.
    #[default]
    ScreenEditor,
    /// Recognize ATASCII EOL but preserve other bytes as glyphs.
    TextFile,
    /// Treat every byte as a glyph, useful for font tables and binary art.
    RawGlyphs,
    /// Emit every byte unchanged.
    RawBytes,
}

/// Backward-compatible alias for the original parser mode name.
pub type ParserMode = DecodeDomain;

/// Stateful incremental ATASCII parser.
#[derive(Clone, Debug)]
pub struct Parser {
    domain: DecodeDomain,
    charset: Charset,
    escaped: bool,
    compatibility: AsciiCompatibility,
}

impl Parser {
    /// Creates a parser with strict native ATASCII behavior.
    pub const fn new(domain: DecodeDomain, charset: Charset) -> Self {
        Self {
            domain,
            charset,
            escaped: false,
            compatibility: AsciiCompatibility {
                backspace: false,
                bell: false,
                form_feed: false,
            },
        }
    }

    /// Creates a parser with optional ASCII-control compatibility.
    pub const fn with_compatibility(
        domain: DecodeDomain,
        charset: Charset,
        compatibility: AsciiCompatibility,
    ) -> Self {
        Self {
            domain,
            charset,
            escaped: false,
            compatibility,
        }
    }

    /// Feeds one byte and returns its parsed token, if any.
    pub fn feed(&mut self, byte: AtasciiByte) -> Option<Token> {
        match self.domain {
            DecodeDomain::RawBytes => Some(Token::Raw(byte)),
            DecodeDomain::RawGlyphs => Some(Token::Glyph(Glyph::quoted(byte, self.charset))),
            DecodeDomain::TextFile => {
                if byte.get() == 0x9B {
                    Some(Token::Control(Control::EndOfLine))
                } else {
                    Some(Token::Glyph(Glyph::quoted(byte, self.charset)))
                }
            }
            DecodeDomain::ScreenEditor => self.feed_editor(byte),
        }
    }

    /// Returns whether an ESC has been received without its following byte.
    pub const fn has_pending_escape(&self) -> bool {
        self.escaped
    }

    /// Clears pending parser state.
    pub fn reset(&mut self) {
        self.escaped = false;
    }

    fn feed_editor(&mut self, byte: AtasciiByte) -> Option<Token> {
        if self.escaped {
            self.escaped = false;
            if byte.get() == 0x9B {
                return Some(Token::Control(Control::EndOfLine));
            }
            return Some(Token::Glyph(Glyph::quoted(byte, self.charset)));
        }

        if let Some(control) = self.compatibility_control(byte.get()) {
            return Some(Token::Control(control));
        }

        match Control::from_byte(byte.get()) {
            Some(Control::Escape) => {
                self.escaped = true;
                None
            }
            Some(control) => Some(Token::Control(control)),
            None => Some(Token::Glyph(Glyph::quoted(byte, self.charset))),
        }
    }

    const fn compatibility_control(&self, byte: u8) -> Option<Control> {
        if self.compatibility.backspace && byte == 0x08 {
            Some(Control::Delete)
        } else if self.compatibility.bell && byte == 0x07 {
            Some(Control::Buzzer)
        } else if self.compatibility.form_feed && byte == 0x0C {
            Some(Control::ClearScreen)
        } else {
            None
        }
    }
}
