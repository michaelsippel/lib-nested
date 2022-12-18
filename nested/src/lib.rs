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
pub mod type_term_editor;

// display
pub mod color;
pub mod terminal;

pub fn magic_header() {
    eprintln!("<<<<>>>><<>><><<>><<<*>>><<>><><<>><<<<>>>>");
}

pub trait Commander {
    type Cmd;

    fn send_cmd(&mut self, cmd: &Self::Cmd);
}

use std::sync::{Arc, RwLock};
use crate::{
    core::context::ReprTree,
    singleton::SingletonView
};

pub trait ObjCommander {
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>);
}

//impl<Cmd: 'static, T: Commander<Cmd>> ObjCommander for T {
impl<C: Commander> ObjCommander for C
where C::Cmd: 'static
{
    fn send_cmd_obj(&mut self, cmd_obj: Arc<RwLock<ReprTree>>) {
        self.send_cmd(
            &cmd_obj.read().unwrap()
                .get_port::<dyn SingletonView<Item = C::Cmd>>().unwrap()
                .get_view().unwrap()
                .get()
        );
    }
}

pub trait StringGen {
    fn get_string(&self) -> String;    
}

use crate::terminal::TerminalEditor;
use crate::{tree::{TreeNav}, diagnostics::Diagnostics};

pub trait Nested
    : TerminalEditor
    + TreeNav
 //   + TreeType
    + Diagnostics
    + Send
    + Sync
    + std::any::Any
{}

