#![feature(trait_alias)]

pub mod core;
pub mod projection;

pub mod grid;
pub mod index;
pub mod integer;
pub mod list;
pub mod sequence;
pub mod singleton;
pub mod terminal;
pub mod vec;

pub mod tree_nav;

pub mod string_editor;

pub mod bimap;

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}
