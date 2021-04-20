use serde::{Serialize, Deserialize};

#[derive(Default, Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct TerminalStyle {
    pub fg_color: Option<(u8, u8, u8)>,
    pub bg_color: Option<(u8, u8, u8)>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>
}

impl TerminalStyle {
    pub fn add(&self, mut dominant: TerminalStyle) -> Self {
        if dominant.fg_color == None {
            dominant.fg_color = self.fg_color;
        }
        if dominant.bg_color == None {
            dominant.bg_color = self.bg_color;
        }
        if dominant.bold == None {
            dominant.bold = self.bold;
        }
        if dominant.italic == None {
            dominant.italic = self.italic;
        }
        if dominant.underline == None {
            dominant.underline = self.underline;
        }
        dominant
    }

    pub fn fg_color(rgb: (u8, u8, u8)) -> Self {
        let mut style = TerminalStyle::default();
        style.fg_color = Some(rgb);
        style
    }

    pub fn bg_color(rgb: (u8, u8, u8)) -> Self {
        let mut style = TerminalStyle::default();
        style.bg_color = Some(rgb);
        style
    }

    pub fn bold(b: bool) -> Self {
        let mut style = TerminalStyle::default();
        style.bold = Some(b);
        style
    }

    pub fn italic(i: bool) -> Self {
        let mut style = TerminalStyle::default();
        style.italic = Some(i);
        style
    }

    pub fn underline(u: bool) -> Self {
        let mut style = TerminalStyle::default();
        style.underline = Some(u);
        style
    }
}

impl std::fmt::Display for TerminalStyle {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.fg_color {
            Some((r, g, b)) => write!(fmt, "{}", termion::color::Fg(termion::color::Rgb(r, g, b)))?,
            None => write!(fmt, "{}", termion::color::Fg(termion::color::Reset))?,
        };
        match self.bg_color {
            Some((r, g, b)) => write!(fmt, "{}", termion::color::Bg(termion::color::Rgb(r, g, b)))?,
            None => write!(fmt, "{}", termion::color::Bg(termion::color::Reset))?,
        };
        match self.bold {
            Some(true) => write!(fmt, "{}", termion::style::Bold)?,
            _ => write!(fmt, "{}", termion::style::NoBold)?,
        };
        match self.italic {
            Some(true) => write!(fmt, "{}", termion::style::Italic)?,
            _ => write!(fmt, "{}", termion::style::NoItalic)?,
        };
        match self.underline {
            Some(true) => write!(fmt, "{}", termion::style::Underline)?,
            _ => write!(fmt, "{}", termion::style::NoUnderline)?,
        };
        Ok(())
    }
}

