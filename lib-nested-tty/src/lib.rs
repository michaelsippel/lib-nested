
#![feature(trait_alias)]

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub mod atom;
pub mod style;

pub mod compositor;
pub mod ansi_parser;

pub mod terminal;

//pub mod list_editor;
//pub mod widgets;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub use {
    atom::TerminalAtom,
    compositor::TerminalCompositor,
    style::TerminalStyle,
    terminal::{Terminal, TerminalEvent},
};

use r3vi::view::grid::*;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait TerminalView = GridView<Item = TerminalAtom>;

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use r3vi::view::OuterViewPort;

pub trait DisplaySegment {
    fn display_view(&self) -> OuterViewPort<dyn TerminalView>;
}


use nested::reprTree::Context;
use std::sync::{Arc, RwLock};

impl DisplaySegment for nested::editTree::NestedNode {
    fn display_view(&self) -> OuterViewPort<dyn TerminalView> {
        self.view.as_ref().unwrap()
            .read().unwrap()
            .descend( Context::parse(&self.ctx, "TerminalView") ).expect("terminal backend not supported by view")
            .read().unwrap()
            .get_port::<dyn TerminalView>().unwrap()
    }
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

use {
    r3vi::{
       buffer::vec::*,
    },
    cgmath::Point2,
};

pub fn make_label(s: &str) -> OuterViewPort<dyn TerminalView> {
    let label = VecBuffer::with_data(s.chars().collect());

    let v = label.get_port()
        .to_sequence()
        .map(|c| TerminalAtom::from(c))
        .to_index()
        .map_key(
            |idx| Point2::new(*idx as i16, 0),
            |pt| if pt.y == 0 { Some(pt.x as usize) } else { None },
        );

    v
}

//<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>

pub trait TerminalProjections {
    fn with_style(&self, style: TerminalStyle) -> OuterViewPort<dyn TerminalView>;
    fn with_fg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView>;
    fn with_bg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView>;
}

impl TerminalProjections for OuterViewPort<dyn TerminalView> {
    fn with_style(&self, style: TerminalStyle) -> OuterViewPort<dyn TerminalView> {
        self.map_item(
            move |_idx, a|
            a.add_style_front(style)
        )
    }

    fn with_fg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView> {
        self.with_style(TerminalStyle::fg_color(col))
    }

    fn with_bg_color(&self, col: (u8, u8, u8)) -> OuterViewPort<dyn TerminalView> {
        self.with_style(TerminalStyle::bg_color(col))
    }
}


