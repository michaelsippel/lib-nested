#![feature(trait_alias)]

pub mod terminal;

pub mod utils;
pub mod editors;
pub mod tree;
pub mod type_system;

pub mod diagnostics;
pub mod commander;
//pub mod product;
//pub mod sum;
//pub mod list;

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}

/*
pub trait StringGen {
    fn get_string(&self) -> String;    
}

use crate::{tree::{TreeNav}, diagnostics::Diagnostics, terminal::TerminalView, core::{OuterViewPort}};
 */

use r3vi::view::OuterViewPort;
use crate::terminal::TerminalView;

pub trait PtySegment {
    fn pty_view(&self) -> OuterViewPort<dyn TerminalView>;
}

