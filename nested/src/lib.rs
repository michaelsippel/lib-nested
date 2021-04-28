#![feature(trait_alias)]

pub mod core;
pub mod projection;

pub mod singleton;
pub mod index;
pub mod grid;
pub mod sequence;
pub mod terminal;
pub mod integer;

pub mod string_editor;
pub mod leveled_term_view;

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}

