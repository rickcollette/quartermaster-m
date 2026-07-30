use crate::{AtasciiByte, Glyph};

/// Atari screen-editor control operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Control {
    /// Escape: quote the next editor-control byte as a glyph, except EOL.
    Escape,
    /// Move cursor up.
    CursorUp,
    /// Move cursor down.
    CursorDown,
    /// Move cursor left.
    CursorLeft,
    /// Move cursor right.
    CursorRight,
    /// Clear screen and home cursor.
    ClearScreen,
    /// Backspace/delete.
    Delete,
    /// Advance to next tab stop.
    Tab,
    /// Atari end-of-line (`0x9B`).
    EndOfLine,
    /// Delete current logical line.
    DeleteLine,
    /// Insert a logical line.
    InsertLine,
    /// Clear tab stop at cursor column.
    ClearTab,
    /// Set tab stop at cursor column.
    SetTab,
    /// Sound buzzer.
    Buzzer,
    /// Delete character at cursor.
    DeleteCharacter,
    /// Insert blank character at cursor.
    InsertCharacter,
}

impl Control {
    /// Interprets a raw ATASCII byte as an editor control, when applicable.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            0x1b => Self::Escape,
            0x1c => Self::CursorUp,
            0x1d => Self::CursorDown,
            0x1e => Self::CursorLeft,
            0x1f => Self::CursorRight,
            0x7d => Self::ClearScreen,
            0x7e => Self::Delete,
            0x7f => Self::Tab,
            0x9b => Self::EndOfLine,
            0x9c => Self::DeleteLine,
            0x9d => Self::InsertLine,
            0x9e => Self::ClearTab,
            0x9f => Self::SetTab,
            0xfd => Self::Buzzer,
            0xfe => Self::DeleteCharacter,
            0xff => Self::InsertCharacter,
            _ => return None,
        })
    }
}

/// A parsed ATASCII stream token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
    /// A rendered character-cell glyph.
    Glyph(Glyph),
    /// A screen-editor operation.
    Control(Control),
    /// An uninterpreted byte retained by a raw parser policy.
    Raw(AtasciiByte),
}
