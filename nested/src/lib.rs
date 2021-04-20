#![feature(trait_alias)]

pub mod core;
pub mod index;
pub mod grid;
pub mod sequence;
pub mod singleton;
pub mod terminal;
pub mod projection;
pub mod string_editor;
pub mod leveled_term_view;

/* maybe?
pub use {
    cgmath::{Vector2, Point2},
    termion::event::{Event, Key},
    crate::{
        core::{View, Observer, ObserverExt, ObserverBroadcast, ViewPort, OuterViewPort},
        index::{ImplIndexView},
        terminal::{
            TerminalView,
            TerminalAtom,
            TerminalStyle,
            TerminalEvent,
            Terminal,
            TerminalCompositor
        },
        sequence::{VecBuffer, SequenceView},
        grid::{GridOffset, GridWindowIterator},
        singleton::{SingletonView, SingletonBuffer},
        string_editor::{StringEditor, insert_view::StringInsertView},
        leveled_term_view::LeveledTermView
    }
};
 */

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}

