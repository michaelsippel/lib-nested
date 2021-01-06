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
        core::View,
        view::{
            IndexView,
            GridView
        }
    },
    cgmath::Point2,
    std::ops::Range
};

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait TerminalView = GridView<Item = Option<TerminalAtom>>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

