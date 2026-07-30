use crate::{Control, Glyph, Token};

/// One terminal cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
    /// Optional glyph; `None` is blank.
    pub glyph: Option<Glyph>,
}

/// Screen operation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScreenError {
    /// Dimensions must be nonzero.
    InvalidDimensions,
}

/// A deterministic Atari-style editor screen model.
#[derive(Clone, Debug)]
pub struct Screen {
    width: usize,
    height: usize,
    cells: alloc::vec::Vec<Cell>,
    x: usize,
    y: usize,
    tabs: alloc::vec::Vec<bool>,
    wrap: bool,
}

extern crate alloc;

impl Screen {
    /// Creates a blank screen.
    pub fn new(width: usize, height: usize) -> Result<Self, ScreenError> {
        if width == 0 || height == 0 {
            return Err(ScreenError::InvalidDimensions);
        }
        let mut tabs = alloc::vec![false; width];
        for i in (0..width).step_by(8) {
            tabs[i] = true;
        }
        Ok(Self {
            width,
            height,
            cells: alloc::vec![Cell { glyph: None }; width * height],
            x: 0,
            y: 0,
            tabs,
            wrap: true,
        })
    }
    /// Applies a parsed token.
    pub fn apply(&mut self, token: Token) {
        match token {
            Token::Glyph(g) => self.put(g),
            Token::Control(c) => self.control(c),
            Token::Raw(_) => {}
        }
    }
    /// Returns a cell.
    pub fn cell(&self, x: usize, y: usize) -> Option<Cell> {
        if x < self.width && y < self.height {
            Some(self.cells[y * self.width + x])
        } else {
            None
        }
    }
    /// Returns cursor coordinates.
    pub const fn cursor(&self) -> (usize, usize) {
        (self.x, self.y)
    }
    fn put(&mut self, g: Glyph) {
        self.cells[self.y * self.width + self.x].glyph = Some(g);
        self.advance();
    }
    fn advance(&mut self) {
        if self.x + 1 < self.width {
            self.x += 1;
        } else if self.wrap {
            self.x = 0;
            self.line_down();
        }
    }
    fn line_down(&mut self) {
        if self.y + 1 < self.height {
            self.y += 1;
        } else {
            self.scroll();
        }
    }
    fn scroll(&mut self) {
        self.cells.copy_within(self.width.., 0);
        let n = self.cells.len();
        for c in &mut self.cells[n - self.width..] {
            c.glyph = None;
        }
    }
    fn control(&mut self, c: Control) {
        match c {
            Control::Escape => {}
            Control::CursorUp => self.y = self.y.saturating_sub(1),
            Control::CursorDown => self.y = (self.y + 1).min(self.height - 1),
            Control::CursorLeft => self.x = self.x.saturating_sub(1),
            Control::CursorRight => self.x = (self.x + 1).min(self.width - 1),
            Control::ClearScreen => {
                for c in &mut self.cells {
                    c.glyph = None;
                }
                self.x = 0;
                self.y = 0;
            }
            Control::Delete => {
                if self.x > 0 {
                    self.x -= 1;
                }
                self.cells[self.y * self.width + self.x].glyph = None;
            }
            Control::Tab => {
                while self.x + 1 < self.width {
                    self.x += 1;
                    if self.tabs[self.x] {
                        break;
                    }
                }
            }
            Control::EndOfLine => {
                self.x = 0;
                self.line_down();
            }
            Control::DeleteLine => {
                let s = self.y * self.width;
                self.cells.copy_within(s + self.width.., s);
                let n = self.cells.len();
                for c in &mut self.cells[n - self.width..] {
                    c.glyph = None;
                }
            }
            Control::InsertLine => {
                let s = self.y * self.width;
                let end = self.cells.len() - self.width;
                self.cells.copy_within(s..end, s + self.width);
                for c in &mut self.cells[s..s + self.width] {
                    c.glyph = None;
                }
            }
            Control::ClearTab => self.tabs[self.x] = false,
            Control::SetTab => self.tabs[self.x] = true,
            Control::Buzzer => {}
            Control::DeleteCharacter => {
                let s = self.y * self.width;
                for i in self.x..self.width - 1 {
                    self.cells[s + i] = self.cells[s + i + 1];
                }
                self.cells[s + self.width - 1].glyph = None;
            }
            Control::InsertCharacter => {
                let s = self.y * self.width;
                for i in (self.x + 1..self.width).rev() {
                    self.cells[s + i] = self.cells[s + i - 1];
                }
                self.cells[s + self.x].glyph = None;
            }
        }
    }
}
