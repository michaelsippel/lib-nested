
#![feature(trait_alias)]

// <<<<>>>><<>><><<>><<< * >>><<>><><<>><<<<>>>> \\

pub mod atom;

pub mod compositor;
pub mod ansi_parser;

pub mod terminal;
pub mod tty_application;

pub mod editors;
pub mod edit_tree;
//pub mod widgets;

// <<<<>>>><<>><><<>><<< * >>><<>><><<>><<<<>>>> \\

pub use {
    atom::{TerminalAtom, TerminalStyle},
    terminal::{Terminal, TerminalEvent},
    tty_application::TTYApplication,
    compositor::TerminalCompositor,
};

use r3vi::view::grid::*;

// <<<<>>>><<>><><<>><<< * >>><<>><><<>><<<<>>>> \\

pub trait TerminalView = GridView<Item = TerminalAtom>;

// <<<<>>>><<>><><<>><<< * >>><<>><><<>><<<<>>>> \\

use r3vi::view::OuterViewPort;

pub trait DisplaySegment {
    fn display_view(&self) -> OuterViewPort<dyn TerminalView>;
}


use nested::repr_tree::{Context, ReprTreeExt};
use std::sync::{Arc, RwLock};

impl DisplaySegment for nested::edit_tree::EditTree {
    fn display_view(&self) -> OuterViewPort<dyn TerminalView> {
        if let Some( tv_repr ) = self.disp.view
            .descend( Context::parse(&self.ctx, "TerminalView") )
        {
            if let Some(port) = 
            tv_repr
                .read().unwrap()
                .get_port::<dyn TerminalView>() {
                    port
                }
                
                else {
                make_label("# could not get ViewPort #")
            }
        } else {
            make_label("# No TTY View available #")
            .map_item(|_p,a| a.add_style_back(TerminalStyle::fg_color((220, 30, 30))))
        }
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


