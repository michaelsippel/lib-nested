pub mod style;
pub use style::TerminalStyle;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct TerminalAtom {
    pub c: Option<char>,
    pub style: TerminalStyle,
}

impl TerminalAtom {
    pub fn new(c: char, style: TerminalStyle) -> Self {
        TerminalAtom { c: Some(c), style }
    }

    pub fn new_bg(bg_color: (u8, u8, u8)) -> Self {
        TerminalAtom {
            c: None,
            style: TerminalStyle::bg_color(bg_color),
        }
    }

    pub fn add_style_front(mut self, style: TerminalStyle) -> Self {
        self.style = self.style.add(style);
        self
    }

    pub fn add_style_back(mut self, style: TerminalStyle) -> Self {
        self.style = style.add(self.style);
        self
    }
}

impl From<char> for TerminalAtom {
    fn from(c: char) -> Self {
        TerminalAtom {
            c: Some(c),
            style: TerminalStyle::default(),
        }
    }
}

impl From<Option<char>> for TerminalAtom {
    fn from(c: Option<char>) -> Self {
        TerminalAtom {
            c,
            style: TerminalStyle::default(),
        }
    }
}

impl From<&char> for TerminalAtom {
    fn from(c: &char) -> Self {
        TerminalAtom {
            c: Some(*c),
            style: TerminalStyle::default(),
        }
    }
}
