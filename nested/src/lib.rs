#![feature(trait_alias)]

// general
pub mod core;
pub mod projection;
pub mod bimap;
pub mod modulo;
pub use modulo::modulo;

// semantics
pub mod singleton;
pub mod sequence;
pub mod index;
pub mod grid;

// implementation
pub mod vec;

// editors
pub mod product;
pub mod sum;
pub mod list;
pub mod tree;
pub mod diagnostics;

// high-level types
pub mod char_editor;
pub mod integer;
pub mod make_editor;

// display
pub mod color;
pub mod terminal;

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}

use crate::terminal::{TerminalEditor};
use crate::diagnostics::{Diagnostics};
use crate::tree::{TreeNav, TreeType};

pub trait Nested
    : TerminalEditor
    + TreeNav
//   + TreeType
    + Diagnostics
    + Send
{}

