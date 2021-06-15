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

pub trait TerminalView = GridView<Item = TerminalAtom>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use {
    crate::{
        vec::VecBuffer,
        core::{ViewPort, OuterViewPort}
    },
    cgmath::Point2
};

pub fn make_label(s: &str) -> OuterViewPort<dyn TerminalView> {
    let label_port = ViewPort::new();
    let _label = VecBuffer::with_data(s.chars().collect(), label_port.inner());
    label_port.outer()
        .to_sequence()
        .map(|c| TerminalAtom::from(c))
        .to_index()
        .map_key(
            |idx| Point2::new(*idx as i16, 0),
            |pt| if pt.y == 0 { Some(pt.x as usize) } else { None }
        )
}

