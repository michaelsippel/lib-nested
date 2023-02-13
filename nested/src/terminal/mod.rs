pub mod ansi_parser;
pub mod atom;
pub mod compositor;
pub mod style;
pub mod terminal;

pub use {
    atom::TerminalAtom,
    compositor::TerminalCompositor,
    style::TerminalStyle,
    terminal::{Terminal, TerminalEvent},
};

use r3vi::view::grid::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait TerminalView = GridView<Item = TerminalAtom>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub enum TerminalEditorResult {
    Continue,
    Exit,
}

pub trait TerminalEditor {
    fn get_term_view(&self) -> OuterViewPort<dyn TerminalView>;
    fn handle_terminal_event(&mut self, event: &TerminalEvent) -> TerminalEditorResult;
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use {
    r3vi::{
        view::{OuterViewPort},
        buffer::vec::*,
    },
    cgmath::Point2,
};

pub fn make_label(s: &str) -> OuterViewPort<dyn TerminalView> {
    let label = VecBuffer::with_data(s.chars().collect());

    let v = label.get_port()
        .to_sequence()
        .map(|c| TerminalAtom::from(c))
        .to_index()
        .map_key(
            |idx| Point2::new(*idx as i16, 0),
            |pt| if pt.y == 0 { Some(pt.x as usize) } else { None },
        );

    v
}

pub trait TerminalProjections {
    fn with_style(&self, style: TerminalStyle) -> OuterViewPort<dyn TerminalView>;
    fn with_fg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView>;
    fn with_bg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView>;
}

impl TerminalProjections for OuterViewPort<dyn TerminalView> {
    fn with_style(&self, style: TerminalStyle) -> OuterViewPort<dyn TerminalView> {
        self.map_item(
            move |_idx, a|
            a.add_style_front(style)
        )
    }

    fn with_fg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView> {
        self.with_style(TerminalStyle::fg_color(col))
    }

    fn with_bg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView> {
        self.with_style(TerminalStyle::bg_color(col))
    }
}

