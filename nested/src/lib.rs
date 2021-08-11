#![feature(trait_alias)]

pub mod core;
pub mod projection;

pub mod singleton;
pub mod index;
pub mod grid;
pub mod sequence;
pub mod vec;
pub mod terminal;
pub mod integer;
pub mod list;

pub mod string_editor;
pub mod leveled_term_view;

pub mod bimap;

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}

