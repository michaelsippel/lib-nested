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

use crate::grid::GridView;

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
    crate::{
        core::{OuterViewPort, ViewPort},
        vec::VecBuffer,
    },
    cgmath::Point2,
};

pub fn make_label(s: &str) -> OuterViewPort<dyn TerminalView> {
    let label_port = ViewPort::new();
    let _label = VecBuffer::with_data(s.chars().collect(), label_port.inner());

    let v = label_port
        .outer()
        .to_sequence()
        .map(|c| TerminalAtom::from(c))
        .to_index()
        .map_key(
            |idx| Point2::new(*idx as i16, 0),
            |pt| if pt.y == 0 { Some(pt.x as usize) } else { None },
        );

    v
}
