#![feature(trait_alias)]

// general
pub mod core;
pub mod type_system;
pub mod projection;
pub mod commander;
pub mod utils;

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
pub mod char;
pub mod integer;

// display
pub mod terminal;

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}

pub trait StringGen {
    fn get_string(&self) -> String;    
}

use crate::terminal::TerminalEditor;
use crate::{tree::{TreeNav}, diagnostics::Diagnostics, terminal::TerminalView, core::{OuterViewPort}};

pub trait PtySegment {
    fn pty_view(&self) -> OuterViewPort<dyn TerminalView>;
}

pub trait Nested
    : TerminalEditor
    + TreeNav
 //   + TreeType
    + Diagnostics
    + Send
    + Sync
    + std::any::Any
{}

