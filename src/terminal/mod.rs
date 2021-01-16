pub mod style;
pub mod atom;
pub mod terminal;
pub mod compositor;

pub use {
    style::{TerminalStyle},
    atom::{TerminalAtom},
    terminal::{Terminal, TerminalEvent},
    compositor::TerminalCompositor,
};

use {
    crate::{
        grid::GridView
    }
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait TerminalView = GridView<Item = Option<TerminalAtom>>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

